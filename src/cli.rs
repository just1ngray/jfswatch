use clap::{ArgAction, Parser};


/// Run a command when watched files change. Files can be given as exact paths, extended glob patterns, or anchored
/// regex. The program will check for mtime, new file, or deleted file changes every `interval` seconds. If a change
/// is detected, the program will execute the specified command and sleep for `sleep` seconds before resuming standard
/// interval checks.
#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// The exact file paths to watch
    #[arg(short, long, action = ArgAction::Append)]
    exact: Vec<String>,

    /// The file paths to watch using extended glob patterns
    #[arg(short, long, action = ArgAction::Append)]
    glob: Vec<String>,

    /// The file paths to watch using anchored regex patterns
    #[arg(short, long, action = ArgAction::Append)]
    regex: Vec<String>,

    /// Seconds between each non-differing check
    #[arg(short, long, default_value_t = 0.1)]
    interval: f32,

    /// Seconds to sleep the program after the specified command has been executed. The program will not check for
    /// changes during this time. By default it uses the same value as `interval`
    #[arg(short, long)]
    sleep: Option<f32>,

    /// The command to execute when changes are detected
    #[arg(required = true)]
    cmd: Vec<String>,

    /// If set, the program will output more information about which files are being watched and how long the program
    /// takes to check for changes
    #[arg(short, long)]
    verbose: bool,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_main_jfwatch_clap_command_when_debug_assert_then_clap_authors_approve() {
        use clap::CommandFactory;
        return Cli::command().debug_assert();
    }
}
