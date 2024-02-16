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
        self.print_formatted_watched_fs(&prev_fs_watch);

        sleep(self.interval);

        loop {
            let new_fs_watch = self.explore(Some(prev_fs_watch.len()));

            match new_fs_watch.compare(prev_fs_watch) {
                FSDifference::Unchanged => {
                    self.handle_unchanged(new_fs_watch.len());
                }
                changed => {
                    self.handle_change(changed);
                    self.print_formatted_watched_fs(&new_fs_watch);
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
            eprintln!();
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
                eprintln!("No changes in {} paths", npaths);
            } else {
                eprint!("+");
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
            eprintln!(
                "---[ {} ]---\nout: {}\n---\nerr: {}\n---[ rc={} ]---\n",
                self.cmd.join(" "),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
                output.status.code().unwrap()
            );
        } else {
            println!(
                "$ {} => exited {}",
                self.cmd.join(" "),
                output.status.code().unwrap()
            );
        }
    }

    fn print_formatted_watched_fs(&self, fs_watch: &WatchedFS) {
        if !self.verbose {
            return;
        }

        eprintln!("--[ List of watched paths ]---\n{fs_watch}---");
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::ExactExplorer;

    #[test]
    fn given_all_valid_args_when_new_then_ok() {
        let explorers: Vec<Box<dyn Explorer>> = vec![Box::new(ExactExplorer::from_cli_arg("path"))];
        let verbose = false;
        let interval = 0.1;
        let sleep = 0.1;
        let cmd = vec!["echo".to_string(), "hello".to_string()];

        let jfswatch = JFSWatch::new(explorers, verbose, interval, sleep, cmd);
        assert!(jfswatch.is_ok());
    }

    #[test]
    fn given_no_command_when_new_then_err() {
        let explorers: Vec<Box<dyn Explorer>> = vec![Box::new(ExactExplorer::from_cli_arg("path"))];
        let verbose = false;
        let interval = 0.1;
        let sleep = 0.1;
        let cmd = vec![];

        let jfswatch = JFSWatch::new(explorers, verbose, interval, sleep, cmd);
        assert!(jfswatch.is_err());
    }

    #[rstest]
    #[case(0.0)]
    #[case(-1.0)]
    fn given_non_positive_interval_when_new_then_err(#[case] interval: f32) {
        let explorers: Vec<Box<dyn Explorer>> = vec![Box::new(ExactExplorer::from_cli_arg("path"))];
        let verbose = false;
        let sleep = 0.1;
        let cmd = vec![];

        let jfswatch = JFSWatch::new(explorers, verbose, interval, sleep, cmd);
        assert!(jfswatch.is_err());
    }

    #[rstest]
    #[case(0.0)]
    #[case(-1.0)]
    fn given_non_positive_sleep_when_new_then_err(#[case] sleep: f32) {
        let explorers: Vec<Box<dyn Explorer>> = vec![Box::new(ExactExplorer::from_cli_arg("path"))];
        let verbose = false;
        let interval = 0.1;
        let cmd = vec![];

        let jfswatch = JFSWatch::new(explorers, verbose, interval, sleep, cmd);
        assert!(jfswatch.is_err());
    }

    #[test]
    fn given_no_explorers_when_new_then_err() {
        let explorers = vec![];
        let verbose = false;
        let interval = 0.1;
        let sleep = 0.1;
        let cmd = vec![];

        let jfswatch = JFSWatch::new(explorers, verbose, interval, sleep, cmd);
        assert!(jfswatch.is_err());
    }
}
