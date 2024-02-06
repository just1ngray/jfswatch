mod cli;

use clap::Parser;

fn main() {
    let parsed = cli::Cli::parse();
    println!("{:?}", parsed);
}
