use clap::{Parser, ArgAction};

/// Some docs about the command itself
#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// TODO - document the arg
    #[arg(short, long, action = ArgAction::Append)]
    file: Vec<String>
}
