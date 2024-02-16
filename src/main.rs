mod cli;
mod explorers;
mod jfswatch;
mod watched_fs;

use crate::explorers::*;
use crate::jfswatch::JFSWatch;

fn main() {
    let parsed = <cli::Cli as clap::Parser>::parse();
    println!("{:?}", parsed);

    if parsed.regex.len() > 0 {
        unimplemented!("Regex and glob paths are not supported yet");
    }

    let mut explorers: Vec<Box<dyn Explorer>> =
        Vec::with_capacity(parsed.exact.len() + parsed.glob.len() + parsed.regex.len());
    explorers.extend(
        parsed
            .exact
            .iter()
            .map(|arg| -> Box<dyn Explorer> { Box::new(ExactExplorer::from_cli_arg(arg)) }),
    );
    explorers.extend(
        parsed
            .glob
            .iter()
            .map(|arg| -> Box<dyn Explorer> { Box::new(GlobExplorer::from_cli_arg(arg)) }),
    );

    let jfs_result = JFSWatch::new(
        explorers,
        parsed.verbose,
        parsed.interval,
        parsed.sleep.unwrap_or(parsed.interval),
        parsed.cmd,
    );

    match jfs_result {
        Ok(mut jfs) => jfs.watch(),
        Err(error) => {
            let mut cmd = <cli::Cli as clap::CommandFactory>::command();
            cmd.error(clap::error::ErrorKind::ValueValidation, error)
                .exit();
        }
    }
}
