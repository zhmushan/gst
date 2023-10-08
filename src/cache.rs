use std::{
    env::var_os,
    fs::{self, create_dir_all},
    io::Write,
    path::{Path, PathBuf},
};

use log::debug;

fn home_dir() -> PathBuf {
    let home_env_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    PathBuf::from(var_os(home_env_var).unwrap())
}

fn cache_dir() -> PathBuf {
    let root = if cfg!(target_os = "macos") {
        home_dir().join("Library/Caches")
    } else if let Some(path) = var_os("XDG_CACHE_HOME") {
        PathBuf::from(path)
    } else {
        home_dir().join(".cache")
    };

    root.join("zhmushan.gst")
}

fn ensure_cache_dir() -> PathBuf {
    let cache_dir = cache_dir();
    let _ = create_dir_all(&cache_dir);

    cache_dir
}

pub fn ensure_dl_dir<P: AsRef<Path>>(path: P) -> PathBuf {
    let dl_dir = ensure_cache_dir().join(path);
    debug!("download dir: {}", dl_dir.display());
    let _ = create_dir_all(&dl_dir);

    dl_dir
}

pub fn ensure_config_file() -> PathBuf {
    let config_path = ensure_cache_dir().join("config.json");
    if fs::metadata(&config_path).is_err() {
        let mut file = fs::File::create(&config_path).unwrap();
        let _ = file.write_all(
            r#"{
    "remote": {
        "gh": "https://github.com"
    },
    "local": {
        "gh": {}
    },
    "from": "gh"
}"#
            .as_bytes(),
        );
    }

    config_path
}
