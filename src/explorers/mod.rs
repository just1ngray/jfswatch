mod exact_explorer;
mod glob_explorer;

pub use exact_explorer::ExactExplorer;
pub use glob_explorer::GlobExplorer;

use crate::watched_fs::WatchedFS;

pub trait Explorer {
    /// Construct an instance of a particular file system explorer given the cli argument
    fn from_cli_arg(arg: &str) -> Self
    where
        Self: Sized;

    /// Explore the file system for file path(s) matching the pattern
    fn explore(&self, watched_fs: &mut WatchedFS);
}
