#[macro_use]
extern crate log;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use flexi_logger::{AdaptiveFormat, Logger};

mod cli;
mod explorers;
mod jfswatch;
mod test_utils;
mod watched_fs;

use crate::explorers::*;
use crate::jfswatch::JFSWatch;

fn main() {
    Logger::try_with_env_or_str("info")
        .unwrap()
        .adaptive_format_for_stdout(AdaptiveFormat::Detailed)
        .log_to_stdout()
        .start()
        .unwrap();

    let parsed = cli::Cli::parse();
    trace!("Parsed CLI args: {:?}", parsed);

    if let Some(shell) = parsed.autocomplete {
        let mut cmd = cli::Cli::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut std::io::stdout());
        return;
    }

    if parsed.cmd.len() == 0 {
        let mut cmd = cli::Cli::command();
        cmd.error(
            clap::error::ErrorKind::ValueValidation,
            "A command must be specified. Use -h for more help",
        )
        .exit();
    }

    let mut explorers: Vec<Box<dyn Explorer>> =
        Vec::with_capacity(parsed.exact.len() + parsed.glob.len());
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
