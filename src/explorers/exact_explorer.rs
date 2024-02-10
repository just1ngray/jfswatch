use std::fs;
use std::path::PathBuf;
use crate::explorers::{Explorer, HashMap, SystemTime};


pub struct ExactExplorer {
    path: PathBuf,
}


impl Explorer for ExactExplorer {
    fn from_cli_arg(arg: &str) -> Self {
        let p = PathBuf::from(arg);
        return Self { path: p };
    }

    fn explore(&self, files: &mut HashMap<String, SystemTime>) {
        if let Ok(metadata) = fs::metadata(&self.path) {
            let mtime = metadata.modified().expect("mtime is not supported on your platform");
            files.insert(self.path.to_string_lossy().to_string(), mtime);
        }
    }
}
