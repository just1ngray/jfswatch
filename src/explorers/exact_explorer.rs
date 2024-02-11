use std::fs;
use std::path::PathBuf;

use crate::explorers::Explorer;
use crate::watched_fs::WatchedFS;

#[derive(Debug)]
pub struct ExactExplorer {
    path: PathBuf,
}

impl Explorer for ExactExplorer {
    fn from_cli_arg(arg: &str) -> Self {
        let p = PathBuf::from(arg);
        return Self { path: p };
    }

    fn explore(&self, watched_fs: &mut WatchedFS) {
        if let Ok(metadata) = fs::metadata(&self.path) {
            let mtime = metadata
                .modified()
                .expect("mtime is not supported on your platform");
            watched_fs.found(self.path.to_string_lossy().to_string(), mtime);
        }
    }
}
