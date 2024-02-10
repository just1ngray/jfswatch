mod exact_explorer;

pub use std::collections::HashMap;
pub use std::time::SystemTime;
pub use exact_explorer::ExactExplorer;


pub trait Explorer {
    /// Construct an instance of a particular file system explorer given the cli argument
    fn from_cli_arg(arg: &str) -> Self;

    /// Explore the file system for file path(s) matching the pattern
    fn explore(&self, files: &mut HashMap<String, SystemTime>);
}
