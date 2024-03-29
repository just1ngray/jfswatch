use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use crate::explorers::Explorer;
use crate::watched_fs::FSDifference;
use crate::watched_fs::WatchedFS;

/// The format for writing DateTime<Local>'s
const LOCAL_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

/// Executes the specified command
fn run_command(command: String) {
    let shell = std::env::var("SHELL").unwrap_or("sh".to_string());

    info!("$ {}", command);
    info!("\n{}", "-".repeat(80));

    let status = Command::new(&shell)
        .args(["-c", &command])
        .stderr(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit())
        .status();

    info!("\n{}", "-".repeat(80));

    match status {
        Ok(status) => {
            info!("... Exited with status: {}", status);
        }
        Err(error) => {
            error!("... Error running command: {}", error);
        }
    }
}

/// Main data structure to maintain the state of the JFSWatch application
pub struct JFSWatch {
    /// How to discover paths on the file system
    explorers: Vec<Box<dyn Explorer>>,

    /// How long to wait between non-changing checks before exploring again
    interval: Duration,

    /// How long to wait after running the command before exploring again
    sleep: Duration,

    /// The command to run when an explored path changes
    cmd: Vec<String>,

    /// For substituting variables into the command
    substitution_pattern: regex::Regex,
}

impl JFSWatch {
    pub fn new(
        explorers: Vec<Box<dyn Explorer>>,
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
            cmd,
            interval: Duration::from_secs_f32(interval),
            sleep: Duration::from_secs_f32(sleep),
            substitution_pattern: regex::Regex::new(r".?\$(\{(diff|path|mtime)\}|diff|path|mtime)")
                .unwrap(),
        });
    }

    /// The main loop for checking the file system and running the specified command (blocking call)
    pub fn watch(&mut self) {
        let mut prev_fs_watch = self.explore(None);
        info!("Found {} initial paths", prev_fs_watch.len());
        debug!("Initial paths:\n{}", prev_fs_watch);

        sleep(self.interval);

        loop {
            let new_fs_watch = self.explore(Some(prev_fs_watch.len()));

            match new_fs_watch.compare(prev_fs_watch) {
                FSDifference::Unchanged => {
                    debug!("No changes in {} paths", new_fs_watch.len());
                    sleep(self.interval);
                }
                changed => {
                    match changed {
                        FSDifference::Modified {
                            ref path,
                            ref mtime,
                        } => {
                            info!(
                                "'{}' was modified at {}",
                                path,
                                mtime.format(LOCAL_DATE_FORMAT)
                            )
                        }
                        FSDifference::New {
                            ref path,
                            ref mtime,
                        } => info!(
                            "'{}' is new since {}",
                            path,
                            mtime.format(LOCAL_DATE_FORMAT)
                        ),
                        FSDifference::Deleted { ref path } => info!("'{}' was deleted", path),
                        FSDifference::Unchanged => unreachable!(),
                    }
                    trace!("Updated paths:\n{}", new_fs_watch);
                    let command = self.get_command(&changed).unwrap();
                    run_command(command);
                    sleep(self.sleep);
                }
            }

            prev_fs_watch = new_fs_watch;
        }
    }

    /// Explores the file system for paths and finds their modified times
    fn explore(&self, prev_size: Option<usize>) -> WatchedFS {
        let mut watched_fs = WatchedFS::new(prev_size.unwrap_or(self.explorers.len()));

        for explorer in self.explorers.iter() {
            explorer.explore(&mut watched_fs);
        }

        return watched_fs;
    }

    /// Returns the command to run, if a command should run. Substitutes variables where available:
    /// - $path | ${path}:   the path that changed
    /// - $diff | ${diff}:   new | modified | deleted
    /// - $mtime | ${mtime}: the modified time of the path (note this will not be available for deleted diffs)
    fn get_command(&self, diff: &FSDifference) -> Option<String> {
        if let FSDifference::Unchanged = diff {
            return None;
        }

        let mut command = self.cmd.join(" ");

        command = self
            .substitution_pattern
            .replace_all(&command, |caps: &regex::Captures| {
                let first_char = caps.get(0).unwrap().as_str().chars().next().unwrap();

                // escaped case - do not substitute
                if first_char == '\\' {
                    return caps.get(0).unwrap().as_str()[1..].to_string();
                }

                let replacement = match caps
                    .get(0)
                    .unwrap()
                    .as_str()
                    .trim_matches(['{', '}', ' ', '$'])
                {
                    "diff" => match diff {
                        FSDifference::Modified { .. } => "modified".to_string(),
                        FSDifference::New { .. } => "new".to_string(),
                        FSDifference::Deleted { .. } => "deleted".to_string(),
                        FSDifference::Unchanged => unreachable!(),
                    },
                    "path" => match diff {
                        FSDifference::Modified { path, .. } => path.to_string(),
                        FSDifference::New { path, .. } => path.to_string(),
                        FSDifference::Deleted { path } => path.to_string(),
                        FSDifference::Unchanged => unreachable!(),
                    },
                    "mtime" => {
                        match diff {
                            FSDifference::Modified { mtime, .. } => {
                                mtime.format(LOCAL_DATE_FORMAT).to_string()
                            }
                            FSDifference::New { mtime, .. } => {
                                mtime.format(LOCAL_DATE_FORMAT).to_string()
                            }
                            FSDifference::Deleted { .. } => {
                                // no mtime for deleted (use what was previously there)
                                caps.get(0).unwrap().as_str()[1..].to_string()
                            }
                            FSDifference::Unchanged => unreachable!(),
                        }
                    }
                    _ => panic!("Unknown substitution target on {:?}", caps),
                };

                return format!("{}{}", first_char, replacement);
            })
            .to_string();

        return Some(command);
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
        let interval = 0.1;
        let sleep = 0.1;
        let cmd = vec!["echo".to_string(), "hello".to_string()];

        let jfswatch = JFSWatch::new(explorers, interval, sleep, cmd);
        assert!(jfswatch.is_ok());
    }

    #[test]
    fn given_no_command_when_new_then_err() {
        let explorers: Vec<Box<dyn Explorer>> = vec![Box::new(ExactExplorer::from_cli_arg("path"))];
        let interval = 0.1;
        let sleep = 0.1;
        let cmd = vec![];

        let jfswatch = JFSWatch::new(explorers, interval, sleep, cmd);
        assert!(jfswatch.is_err());
    }

    #[rstest]
    #[case(0.0)]
    #[case(-1.0)]
    fn given_non_positive_interval_when_new_then_err(#[case] interval: f32) {
        let explorers: Vec<Box<dyn Explorer>> = vec![Box::new(ExactExplorer::from_cli_arg("path"))];
        let sleep = 0.1;
        let cmd = vec!["echo".to_string(), "hello".to_string()];

        let jfswatch = JFSWatch::new(explorers, interval, sleep, cmd);
        assert!(jfswatch.is_err());
    }

    #[rstest]
    #[case(0.0)]
    #[case(-1.0)]
    fn given_non_positive_sleep_when_new_then_err(#[case] sleep: f32) {
        let explorers: Vec<Box<dyn Explorer>> = vec![Box::new(ExactExplorer::from_cli_arg("path"))];
        let interval = 0.1;
        let cmd = vec!["echo".to_string(), "hello".to_string()];

        let jfswatch = JFSWatch::new(explorers, interval, sleep, cmd);
        assert!(jfswatch.is_err());
    }

    #[test]
    fn given_no_explorers_when_new_then_err() {
        let explorers = vec![];
        let interval = 0.1;
        let sleep = 0.1;
        let cmd = vec!["echo".to_string(), "hello".to_string()];

        let jfswatch = JFSWatch::new(explorers, interval, sleep, cmd);
        assert!(jfswatch.is_err());
    }

    fn jfswatch_with_command(command: Vec<&str>) -> JFSWatch {
        let explorers: Vec<Box<dyn Explorer>> = vec![Box::new(ExactExplorer::from_cli_arg("path"))];
        let interval = 0.1;
        let sleep = 0.1;
        let cmd = command.iter().map(|s| s.to_string()).collect();
        let jfswatch = JFSWatch::new(explorers, interval, sleep, cmd).unwrap();
        return jfswatch;
    }

    #[test]
    fn given_unchanged_diff_when_get_command_then_none() {
        let jfswatch = jfswatch_with_command(vec!["doesn't", "matter"]);
        let diff = FSDifference::Unchanged;

        match jfswatch.get_command(&diff) {
            Some(_) => panic!("Expected None"),
            None => {}
        }
    }

    #[test]
    fn given_new_diff_when_get_command_then_substitutes_all() {
        let jfswatch =
            jfswatch_with_command(vec!["echo", "$diff", "$path was", "created at $mtime"]);
        let mtime = chrono::Local::now();
        let diff = FSDifference::New {
            path: "mock/path".to_string(),
            mtime: mtime,
        };
        let command = jfswatch.get_command(&diff).unwrap();

        assert_eq!(
            command,
            format!(
                "echo new mock/path was created at {}",
                mtime.format(LOCAL_DATE_FORMAT)
            )
        );
    }

    #[test]
    fn given_modified_diff_when_get_command_then_substitutes_all() {
        let jfswatch =
            jfswatch_with_command(vec!["echo", "{ diff: $diff, path: $path, mtime: $mtime }"]);
        let mtime = chrono::Local::now();
        let diff = FSDifference::Modified {
            path: "mock/path".to_string(),
            mtime: mtime,
        };
        let command = jfswatch.get_command(&diff).unwrap();

        assert_eq!(
            command,
            format!(
                "echo {{ diff: modified, path: mock/path, mtime: {} }}",
                mtime.format(LOCAL_DATE_FORMAT)
            )
        );
    }

    #[test]
    fn given_deleted_diff_when_get_command_then_substitutes_all() {
        let jfswatch = jfswatch_with_command(vec![
            "echo",
            "{ diff: $diff }",
            "path: $path\nmtime: $mtime",
        ]);
        let diff = FSDifference::Deleted {
            path: "mock/path".to_string(),
        };
        let command = jfswatch.get_command(&diff).unwrap();

        assert_eq!(
            command,
            format!("echo {{ diff: deleted }} path: mock/path\nmtime: $mtime")
        );
    }

    #[rstest]
    #[case(FSDifference::New { path: "mock/path".to_string(), mtime: chrono::Local::now() })]
    #[case(FSDifference::Modified { path: "mock/path".to_string(), mtime: chrono::Local::now() })]
    #[case(FSDifference::Deleted { path: "mock/path".to_string() })]
    fn given_any_diff_when_get_command_then_ignores_escaped_variables(#[case] diff: FSDifference) {
        let jfswatch = jfswatch_with_command(vec!["echo $path \\$path \\${path} ${path}"]);
        let command = jfswatch.get_command(&diff).unwrap();

        assert_eq!(command, "echo mock/path $path ${path} mock/path");
    }
}
