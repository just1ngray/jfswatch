use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::fmt::Display;
use std::time::SystemTime;

/// A type to track the differences between two WatchedFS structs.
#[derive(Debug, PartialEq)]
pub enum FSDifference {
    Unchanged,
    Modified { path: String, mtime: SystemTime },
    New { path: String, mtime: SystemTime },
    Deleted { path: String },
}

/// A data structure to manage the watched paths on the filesystem and their last modified time
#[derive(Debug, PartialEq, Clone)]
pub struct WatchedFS {
    paths: HashMap<String, SystemTime>,
}

impl WatchedFS {
    /// Creates a new WatchedFS with a given capacity
    /// Note: the capacity will expand automatically as needed
    pub fn new(size: usize) -> Self {
        let map = HashMap::with_capacity(size);
        return WatchedFS { paths: map };
    }

    /// Returns an iterator over the paths and their last modified time
    #[allow(dead_code)]
    pub fn paths(&self) -> Keys<'_, String, SystemTime> {
        return self.paths.keys();
    }

    /// Record that a given `path` exists, and was last modified at `mtime`
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
                    return FSDifference::Modified {
                        path: owned_path,
                        mtime: *mtime,
                    };
                }
            } else {
                // path did not exist in the previous filesystem
                return FSDifference::New {
                    path: path.clone(),
                    mtime: *mtime,
                };
            }
        }

        // if the path still exists in the previous filesystem paths, then it does not exist in self's
        for (path, _mtime) in prev_fs.paths {
            return FSDifference::Deleted { path };
        }

        return FSDifference::Unchanged;
    }
}

impl Display for WatchedFS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.len() == 0 {
            return write!(f, "");
        }

        return writeln!(
            f,
            "{}",
            self.paths
                .keys()
                .map(|path| path.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        );
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::time::Duration;

    use super::*;

    #[test]
    fn given_watched_fs_when_path_found_then_is_added_to_paths() {
        let mut watched = WatchedFS {
            paths: HashMap::new(),
        };
        let mock_path = "mock/path".to_string();
        let mock_time = SystemTime::now();
        watched.found(mock_path.clone(), mock_time.clone());
        assert_eq!(
            watched.paths,
            HashMap::from([(mock_path.clone(), mock_time)])
        );
        assert_eq!(watched.paths().collect::<Vec<&String>>(), vec![&mock_path]);
    }

    #[test]
    fn given_watched_fs_when_len_then_returns_current_size() {
        let mut watched = WatchedFS {
            paths: HashMap::new(),
        };
        assert_eq!(watched.len(), 0);
        assert_eq!(watched.paths().len(), 0);
        assert_eq!(
            watched
                .paths()
                .map(|p| p.clone())
                .collect::<HashSet<String>>(),
            HashSet::new()
        );

        watched.found("path/a".to_string(), SystemTime::now());
        assert_eq!(watched.len(), 1);
        assert_eq!(watched.paths().len(), 1);
        assert_eq!(
            watched
                .paths()
                .map(|p| p.clone())
                .collect::<HashSet<String>>(),
            HashSet::from(["path/a".to_string()])
        );

        watched.found("path/b".to_string(), SystemTime::now());
        assert_eq!(watched.len(), 2);
        assert_eq!(watched.paths().len(), 2);
        assert_eq!(
            watched
                .paths()
                .map(|p| p.clone())
                .collect::<HashSet<String>>(),
            HashSet::from(["path/a".to_string(), "path/b".to_string()])
        );

        watched.found("path/a".to_string(), SystemTime::now());
        assert_eq!(watched.len(), 2);
        assert_eq!(watched.paths().len(), 2);
        assert_eq!(
            watched
                .paths()
                .map(|p| p.clone())
                .collect::<HashSet<String>>(),
            HashSet::from(["path/a".to_string(), "path/b".to_string()])
        );
    }

    #[test]
    fn given_empty_fs_when_compared_against_another_empty_then_is_unchanged() {
        let a = WatchedFS {
            paths: HashMap::new(),
        };
        let b = WatchedFS {
            paths: HashMap::new(),
        };
        assert_eq!(a.compare(b), FSDifference::Unchanged);
        assert_eq!(a.len(), 0);
    }

    #[test]
    fn given_non_empty_fs_when_compared_against_itself_then_is_unchanged() {
        let mut watched = WatchedFS {
            paths: HashMap::new(),
        };
        watched.found("/some/path".to_string(), SystemTime::now());

        let watched_cloned = watched.clone();
        assert_eq!(watched.compare(watched_cloned), FSDifference::Unchanged);
        assert_eq!(watched.len(), 1);
    }

    #[test]
    fn given_modified_fs_when_compared_then_returns_modified_with_path() {
        let path = "/this/will/be/modified".to_string();
        let mtime_initial = SystemTime::now() - Duration::new(10, 0); // 10s ago
        let mtime_now = SystemTime::now();

        let prev_watched = WatchedFS {
            paths: HashMap::from([(path.clone(), mtime_initial)]),
        };
        let curr_watched = WatchedFS {
            paths: HashMap::from([(path.clone(), mtime_now.clone())]),
        };

        assert_eq!(
            curr_watched.compare(prev_watched),
            FSDifference::Modified {
                path: path,
                mtime: mtime_now
            }
        );
        assert_eq!(curr_watched.len(), 1);
    }

    #[test]
    fn given_new_file_when_compared_then_returns_new_path() {
        let new_path = "new/path".to_string();
        let prev_watched = WatchedFS {
            paths: HashMap::new(),
        };
        let mtime = SystemTime::now();
        let curr_watched = WatchedFS {
            paths: HashMap::from([(new_path.clone(), mtime.clone())]),
        };

        assert_eq!(
            curr_watched.compare(prev_watched),
            FSDifference::New {
                path: new_path,
                mtime: mtime
            }
        );
        assert_eq!(curr_watched.len(), 1);
    }

    #[test]
    fn given_deleted_file_when_compared_then_returns_deleted_path() {
        let deleted_path = "deleted/path".to_string();
        let prev_watched = WatchedFS {
            paths: HashMap::from([(deleted_path.clone(), SystemTime::now())]),
        };
        let curr_watched = WatchedFS {
            paths: HashMap::new(),
        };

        assert_eq!(
            curr_watched.compare(prev_watched),
            FSDifference::Deleted { path: deleted_path }
        );
        assert_eq!(curr_watched.len(), 0);
    }

    #[test]
    fn given_watched_fs_when_displayed_then_shows_all_paths() {
        let mut watched = WatchedFS::new(3);
        watched.found("path/a".to_string(), SystemTime::now());
        watched.found("path/b".to_string(), SystemTime::now());
        watched.found("path/c".to_string(), SystemTime::now());

        let displayed = format!("{}", watched);

        assert!(displayed.contains("path/a"));
        assert!(displayed.contains("path/b"));
        assert!(displayed.contains("path/c"));
    }

    #[test]
    fn given_empty_watched_fs_when_displayed_then_is_empty_string() {
        let watched = WatchedFS::new(0);

        let displayed = format!("{}", watched);

        assert_eq!(displayed, "".to_string());
    }
}
