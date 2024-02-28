use std::path::PathBuf;

use crate::explorers::Explorer;
use crate::watched_fs::WatchedFS;

/// A file system explorer that looks for a simple path on the file system.
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
        watched_fs.find(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir_in;

    use crate::test_utils::utils::make_files;

    use super::*;

    #[test]
    fn given_non_existing_path_when_explore_then_watched_unchanged() {
        let tmp = tempdir_in(".").unwrap();
        let basedir = tmp.path().to_owned();
        let path = basedir.join("i-dont-exist");

        let mut watched = WatchedFS::new(10);
        let explorer = ExactExplorer { path };

        explorer.explore(&mut watched);

        assert_eq!(watched.len(), 0);
    }

    #[test]
    fn given_existing_path_when_explore_then_watched_includes_that_path() {
        let tmp = tempdir_in(".").unwrap();
        let basedir = tmp.path().to_owned();
        let path = make_files(&basedir, vec!["file.txt"])[0].to_owned();

        let mut watched = WatchedFS::new(10);
        let explorer = ExactExplorer { path };

        explorer.explore(&mut watched);

        assert_eq!(watched.len(), 1);
    }
}
