use std::io;
use std::io::Write;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use crate::explorers::Explorer;
use crate::watched_fs::FSDifference;
use crate::watched_fs::WatchedFS;

pub struct JFSWatch {
    explorers: Vec<Box<dyn Explorer>>,
    verbose: bool,
    interval: Duration,
    sleep: Duration,
    cmd: Vec<String>,
    no_change_count: u32,
}

impl JFSWatch {
    pub fn new(
        explorers: Vec<Box<dyn Explorer>>,
        verbose: bool,
        interval: f32,
        sleep: f32,
        cmd: Vec<String>,
    ) -> Result<Self, String> {
        if cmd.len() == 0 {
            return Err("No command was given".to_string());
        }
        if interval <= 0.0 {
            return Err("Interval must be a positive number of seconds".to_string());
        }
        if sleep <= 0.0 {
            return Err("Sleep must be a positive number of seconds".to_string());
        }
        if explorers.len() == 0 {
            return Err("Empty path discovery list".to_string());
        }

        return Ok(JFSWatch {
            explorers,
            verbose,
            cmd,
            interval: Duration::from_secs_f32(interval),
            sleep: Duration::from_secs_f32(sleep),
            no_change_count: 0,
        });
    }

    pub fn watch(&mut self) {
        let mut prev_fs_watch = self.explore(None);
        sleep(self.interval);

        loop {
            let new_fs_watch = self.explore(Some(prev_fs_watch.len()));

            match new_fs_watch.compare(prev_fs_watch) {
                FSDifference::Unchanged => {
                    self.handle_unchanged(new_fs_watch.len());
                }
                changed => {
                    self.handle_change(changed);
                }
            }

            prev_fs_watch = new_fs_watch;
        }
    }

    fn explore(&self, prev_size: Option<usize>) -> WatchedFS {
        let mut watched_fs = WatchedFS::new(prev_size.unwrap_or(self.explorers.len()));

        for explorer in self.explorers.iter() {
            explorer.explore(&mut watched_fs);
        }

        return watched_fs;
    }

    fn handle_change(&mut self, diff: FSDifference) {
        if self.no_change_count > 0 {
            println!();
            self.no_change_count = 0;
        }

        match diff {
            FSDifference::Unchanged => unreachable!(),
            FSDifference::Modified(path) => println!("'{}' was modified", path),
            FSDifference::New(path) => println!("'{}' is new", path),
            FSDifference::Deleted(path) => println!("'{}' was deleted", path),
        }

        self.run_command();

        sleep(self.sleep);
    }

    fn handle_unchanged(&mut self, npaths: usize) {
        if self.verbose {
            if self.no_change_count == 0 {
                println!("No changes in {} paths", npaths);
            } else {
                print!("+");
                io::stdout().flush().unwrap();
            }

            self.no_change_count += 1;
        }

        sleep(self.interval);
    }

    fn run_command(&self) {
        let mut cmd = Command::new(&self.cmd[0]);
        cmd.args(&self.cmd[1..]);

        let output = cmd.output().unwrap();

        if self.verbose {
            println!(
                "---[ {} ]---\nout: {}\n---\nerr: {}\n---",
                self.cmd.join(" "),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
