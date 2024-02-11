use std::collections::HashMap;
use std::time::SystemTime;


/// A type to track the differences between two WatchedFS structs.
pub enum FSDifference {
    Unchanged,
    Modified(String),
    New(String),
    Deleted(String)
}


pub struct WatchedFS {
    paths: HashMap<String, SystemTime>
}

impl WatchedFS {
    /// After exploring and finding the existing path `abspath` last modified at `mtime`, mark the path as found
    pub fn found(&mut self, path: String, mtime: SystemTime) {
        self.paths.insert(path, mtime);
    }

    /// How many paths have been found
    pub fn len(&self) -> usize {
        return self.paths.len();
    }

    /// Compares the current state of the file system against a previous state. Returns an enum indicating the
    /// first detected difference, if any
    pub fn compare(&self, mut prev_fs: WatchedFS) -> FSDifference {
        // ensure that all paths in the current filesystem existed in the previous filesystem
        for (path, mtime) in &self.paths {
            if let Some((owned_path, prev_mtime)) = prev_fs.paths.remove_entry(path) {
                // path existed, but now we must check the mtime
                if mtime != &prev_mtime {
                    return FSDifference::Modified(owned_path);
                }
            }
            else {
                // path did not exist in the previous filesystem
                return FSDifference::New(path.to_owned());
            }
        }

        // if the path still exists in the previous filesystem paths, then it does not exist in self's
        for (path, _mtime) in prev_fs.paths {
            return FSDifference::Deleted(path);
        }

        return FSDifference::Unchanged;
    }
}
