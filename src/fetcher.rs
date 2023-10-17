use std::{
    fs::{self, create_dir_all, remove_dir_all},
    io::{self, Write},
    path::PathBuf,
    process::exit,
};

use log::debug;

use crate::{
    config::Config,
    console::{confirm, P, S},
    error::AnyError,
};

#[derive(Debug)]
pub struct Fetcher<'a> {
    config: &'a mut Config,
    repo: String,
    repo_org: String,
    repo_name: String,
    maybe_subdir: Option<String>,
}

impl<'a> Fetcher<'a> {
    pub fn new(config: &'a mut Config, repo: &str) -> Self {
        let parts = repo.splitn(3, '/').collect::<Vec<_>>();
        if let [repo_org, repo_name, ..] = parts.as_slice() {
            let subdir = parts.get(2).copied();

            Self {
                config,
                repo: format!("{}/{}", repo_org, repo_name),
                repo_org: repo_org.to_string(),
                repo_name: repo_name.to_string(),
                maybe_subdir: subdir.map(|s| s.to_string()),
            }
        } else {
            panic!("Please enter a valid repository.")
        }
    }

    pub async fn go(&mut self, maybe_target: Option<String>, force: bool) -> Result<(), AnyError> {
        debug!("{:#?}", &self);
        let target = maybe_target.map_or(
            match &self.maybe_subdir {
                Some(subdir) => subdir.split('/').last().unwrap().to_owned(),
                _ => self.repo_name.to_owned(),
            },
            |s| s,
        );
        let target = PathBuf::from(target);

        if force {
            remove_dir_all(&target).unwrap();
        } else if target.exists() && fs::read_dir(&target).unwrap().count() > 0 {
            let confirmed = confirm(
                format!(
                    "Target directory `{}` is not empty. Remove existing files and continue?",
                    target.to_path_buf().p_display()
                )
                .bold()
                .as_str(),
            );

            if confirmed {
                remove_dir_all(&target).unwrap();
            } else {
                exit(0);
            }
        }

        let archive_path = self.dl().await?;
        self.generate_from_archive(&archive_path, &target).await?;

        Ok(())
    }

    async fn dl(&mut self) -> Result<PathBuf, AnyError> {
        let repo_hash = self.config.get_hash(&self.repo).unwrap();

        let dl_dir = self.config.from.get_dl_dir(&self.repo_org);
        let dl_filename = format!("{}-{}.tar.gz", &self.repo_name, &repo_hash);
        let output = dl_dir.join(&dl_filename);

        let archive_exists = match fs::metadata(&output) {
            Ok(meta) => meta.is_file(),
            _ => false,
        };

        if archive_exists {
            debug!("found cache: {}", output.display());
        } else {
            let archive_url = self.config.from.get_archive_url(
                format!("{}/{}", self.repo_org, self.repo_name).as_str(),
                &repo_hash,
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
        create_dir_all(target).unwrap();

        let archive_file = fs::File::open(archive_path)?;
        let gz_decoder = flate2::read::GzDecoder::new(archive_file);
        let reader = io::BufReader::new(gz_decoder);

        let mut archive = tar::Archive::new(reader);

        archive
            .entries()?
            .filter_map(|e| e.ok())
            .filter_map(|mut e| -> Option<PathBuf> {
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
            .for_each(|p| {
                print!("{}", p.display().to_string().pin());
            });

        println!(
            "{}",
            format!(
                "Done. Your project has been placed at `{}`",
                target.p_display(),
            )
            .bold()
            .pin()
        );

        Ok(())
    }
}
