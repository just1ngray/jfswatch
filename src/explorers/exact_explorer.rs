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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_non_existing_path_when_explore_then_watched_unchanged() {
        let mut watched = WatchedFS::new(10);
        let path = std::env::temp_dir().join("i-dont-exist");
        let explorer = ExactExplorer { path };

        explorer.explore(&mut watched);

        assert_eq!(watched.len(), 0);
    }

    #[test]
    fn given_existing_path_when_explore_then_watched_includes_that_path() {
        let mut watched = WatchedFS::new(10);
        let path = std::env::temp_dir().join("file.txt");
        std::fs::write(&path, "contents").unwrap();
        let explorer = ExactExplorer { path };

        explorer.explore(&mut watched);

        assert_eq!(watched.len(), 1);
    }
}
