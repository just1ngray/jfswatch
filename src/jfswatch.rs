use std::time::Duration;
use std::thread::sleep;

use crate::explorers::Explorer;
use crate::watched_fs::WatchedFS;
use crate::watched_fs::FSDifference;


pub struct JFSWatch {
    explorers: Vec<Box<dyn Explorer>>,
    verbose: bool,
    interval: Duration,
    sleep: Duration,
    cmd: Vec<String>
}

impl JFSWatch {
    pub fn new(explorers: Vec<Box<dyn Explorer>>, verbose: bool, interval: f32, sleep: f32, cmd: Vec<String>) -> Self {
        return JFSWatch {
            explorers, verbose, cmd,
            interval: Duration::from_secs_f32(interval),
            sleep: Duration::from_secs_f32(sleep),
        }
    }

    pub fn watch(&self) {
        let mut prev_fs_watch = self.explore(None);
        sleep(self.interval);

        loop {
            let new_fs_watch = self.explore(Some(prev_fs_watch.len()));

            let diff = new_fs_watch.compare(prev_fs_watch);
            let mut delay = self.sleep;
            match diff {
                FSDifference::Unchanged => {
                    delay = self.interval;
                },
                FSDifference::Modified(path) => {},
                FSDifference::New(path) => {},
                FSDifference::Deleted(path) => {},
            }

            prev_fs_watch = new_fs_watch;
            sleep(delay);
        }
    }

    fn explore(&self, prev_size: Option<usize>) -> WatchedFS {
        let mut watched_fs = WatchedFS::new(prev_size.unwrap_or(self.explorers.len()));

        for explorer in self.explorers.iter() {
            explorer.explore(&mut watched_fs);
        }

        return watched_fs;
    }
}
