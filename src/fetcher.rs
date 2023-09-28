use std::{
    fmt, fs,
    io::{self, Write},
    path::PathBuf,
};

use git2::Oid;
use log::debug;

use crate::{cache, error::AnyError, git_utils};

const GH: &str = "github.com";

pub enum Site {
    GH,
}

impl Site {
    fn get_archive_url(&self, repo_id: &str, hash: &Oid) -> String {
        match self {
            Self::GH => format!(
                "https://codeload.{}/{}/tar.gz/{}",
                GH,
                repo_id,
                hash.to_string()
            ),
        }
    }

    fn get_host_url(&self) -> String {
        match self {
            Self::GH => format!("https://{}", GH),
        }
    }

    fn get_dl_dirname(&self) -> String {
        match self {
            Self::GH => GH.to_string(),
        }
    }
}

pub struct Fetcher {
    site: Site,

    repo_org: String,
    repo_name: String,

    repo_head_hash: Oid,

    maybe_subdir: Option<String>,
}

impl Fetcher {
    pub fn new(site: Site, repo_id: &str) -> Self {
        let parts = repo_id.splitn(3, '/').collect::<Vec<_>>();
        if let [repo_org, repo_name, ..] = parts.as_slice() {
            let subdir = parts.get(2).copied();

            let repo_head_hash = git_utils::get_head(
                format!("{}/{}/{}", site.get_host_url(), repo_org, repo_name).as_str(),
            )
            .unwrap();

            Self {
                site,
                repo_org: repo_org.to_string(),
                repo_name: repo_name.to_string(),
                repo_head_hash,
                maybe_subdir: subdir.map(|s| s.to_string()),
            }
        } else {
            panic!("Please enter a valid repository.")
        }
    }

    pub async fn go(&self, maybe_target: Option<String>) -> Result<(), AnyError> {
        debug!("{}", &self);
        let archive_path = self.dl().await?;
        let target = maybe_target.map_or(
            match &self.maybe_subdir {
                Some(subdir) => subdir.split('/').last().unwrap().to_owned(),
                _ => self.repo_name.to_owned(),
            },
            |s| s,
        );
        let target = PathBuf::from(target);
        self.generate_from_archive(&archive_path, &target).await?;

        Ok(())
    }

    async fn dl(&self) -> Result<PathBuf, AnyError> {
        let output =
            cache::ensure_dl_dir(format!("{}/{}", self.site.get_dl_dirname(), &self.repo_org))
                .join(format!(
                    "{}-{}.tar.gz",
                    &self.repo_name, &self.repo_head_hash
                ));

        let archive_exists = match fs::metadata(&output) {
            Ok(meta) => meta.is_file(),
            _ => false,
        };

        if archive_exists {
            debug!("found cache: {}", output.display());
        } else {
            let archive_url = self.site.get_archive_url(
                format!("{}/{}", self.repo_org, self.repo_name).as_str(),
                &self.repo_head_hash,
            );
            debug!("download {} to {}", archive_url, output.display());

            let mut file = fs::File::create(&output)?;

            let response = reqwest::get(archive_url).await?;
            let content = response.bytes().await?;

            let _ = file.write_all(&content);
        }

        Ok(output)
    }

    async fn generate_from_archive(
        &self,
        archive_path: &PathBuf,
        target: &PathBuf,
    ) -> Result<(), AnyError> {
        let archive_file = fs::File::open(archive_path)?;
        let gz_decoder = flate2::read::GzDecoder::new(archive_file);
        let reader = io::BufReader::new(gz_decoder);

        let mut archive = tar::Archive::new(reader);

        archive
            .entries()?
            .filter_map(|e| e.ok())
            .map(|mut e| -> Option<PathBuf> {
                let maybe_adopted_path =
                    e.path().unwrap().components().skip(1).collect::<PathBuf>();
                let maybe_adopted_path = if let Some(subdir) = &self.maybe_subdir {
                    maybe_adopted_path
                        .strip_prefix(subdir)
                        .map(|p| p.to_path_buf())
                        .ok()
                } else {
                    Some(maybe_adopted_path)
                };

                if let Some(adopted_path) = maybe_adopted_path {
                    let mut target_path = PathBuf::from(target);
                    target_path.push(adopted_path);
                    let _ = e.unpack(&target_path);

                    Some(target_path)
                } else {
                    None
                }
            })
            .filter_map(|p| p)
            .for_each(|p| debug!("> {}", p.display()));

        Ok(())
    }
}

impl fmt::Display for Fetcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = format!(
            r#"
site: {}
repo: {}
hash: {}"#,
            self.site.get_host_url(),
            format!("{}/{}", self.repo_org, self.repo_name),
            self.repo_head_hash,
        );

        if let Some(subdir) = &self.maybe_subdir {
            res.push_str(format!("\nsubdir: {}", subdir).as_str());
        };

        writeln!(f, "{}", res)
    }
}
