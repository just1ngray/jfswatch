mod exact_explorer;

pub use exact_explorer::ExactExplorer;

use crate::watched_fs::WatchedFS;


pub trait Explorer {
    /// Construct an instance of a particular file system explorer given the cli argument
    fn from_cli_arg(arg: &str) -> Self;

    /// Explore the file system for file path(s) matching the pattern
    fn explore(&self, watched_fs: &mut WatchedFS);
}
