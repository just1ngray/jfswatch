mod exact_explorer;

pub use exact_explorer::ExactExplorer;
use std::collections::HashMap;


pub trait Explorer {
    /// Explore the file system for file path(s) matching the pattern
    fn explore(&self, files: &mut HashMap<String, std::time::SystemTime>);
}
