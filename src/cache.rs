use std::{
    env::var_os,
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use log::debug;

fn home_dir() -> PathBuf {
    PathBuf::from(var_os("HOME").unwrap())
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

pub fn ensure_dl_dir<P: AsRef<Path>>(path: P) -> PathBuf {
    let dl_dir = cache_dir().join(path);
    debug!("download dir: {}", dl_dir.display());
    let _ = create_dir_all(&dl_dir);

    dl_dir
}
