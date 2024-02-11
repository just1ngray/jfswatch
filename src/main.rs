mod cli;
mod explorers;
mod jfswatch;
mod watched_fs;

use crate::explorers::*;
use crate::jfswatch::JFSWatch;

fn main() {
    let parsed = <cli::Cli as clap::Parser>::parse();
    println!("{:?}", parsed);

    let explorers: Vec<Box<dyn Explorer>> = parsed
        .exact
        .iter()
        .map(|exact| -> Box<dyn explorers::Explorer> {
            Box::new(ExactExplorer::from_cli_arg(exact))
        })
        .collect();

    let jfs = JFSWatch::new(
        explorers,
        parsed.verbose,
        parsed.interval,
        parsed.sleep.unwrap_or(parsed.interval),
        parsed.cmd,
    );
    jfs.watch();
}
