use std::{collections::HashMap, fs, path::PathBuf};

use log::debug;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    cache::{self, ensure_config_file},
    error::AnyError,
    git_utils,
};

#[derive(Deserialize, Serialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum From {
    #[serde(rename = "gh")]
    GH,
}

impl From {
    pub fn get_archive_url(&self, repo: &str, hash: &str) -> String {
        match self {
            Self::GH => format!("https://codeload.github.com/{}/tar.gz/{}", repo, hash,),
        }
    }

    pub fn get_host_url(&self) -> String {
        match self {
            Self::GH => String::from("https://github.com"),
        }
    }

    pub fn get_dl_dir(&self, repo_org: &str) -> PathBuf {
        let dirname = match self {
            Self::GH => String::from("github.com"),
        };

        cache::ensure_dl_dir(format!("{}/{}", dirname, repo_org))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub local: HashMap<From, HashMap<String, String>>,
    pub from: From,
}

impl Config {
    pub fn new() -> Self {
        let config_path = ensure_config_file();
        let file_contents = fs::read_to_string(&config_path).unwrap();
        serde_json::from_str(&file_contents).unwrap()
    }

    pub fn update_hash(&mut self, repo: &str) -> Result<(), AnyError> {
        let new_hash =
            git_utils::get_head(format!("{}/{}", self.from.get_host_url(), repo).as_str())?
                .to_string();
        if !self.get_hash(repo).is_some_and(|hash| hash.eq(&new_hash)) {
            let repo_cache = self.local.entry(self.from).or_insert(HashMap::new());
            repo_cache.insert(repo.to_string(), new_hash);
            let _ = self.apply();
        }

        Ok(())
    }

    pub fn get_hash(&self, repo: &str) -> Option<String> {
        self.local
            .get(&self.from)
            .and_then(|repos| repos.get(repo).cloned())
    }

    fn apply(&self) -> Result<(), AnyError> {
        let config_path = ensure_config_file();
        debug!("write to {}", config_path.display());
        fs::write(&config_path, serde_json::to_string_pretty(&self)?)?;

        Ok(())
    }
}
