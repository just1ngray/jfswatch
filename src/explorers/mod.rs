mod exact_explorer;

pub use std::collections::HashMap;
pub use std::time::SystemTime;
pub use exact_explorer::ExactExplorer;


pub trait Explorer {
    /// Explore the file system for file path(s) matching the pattern
    fn explore(&self, files: &mut HashMap<String, SystemTime>);
}
