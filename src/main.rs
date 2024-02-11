mod cli;
mod explorers;
mod watched_fs;


fn main() {
    let parsed = <cli::Cli as clap::Parser>::parse();
    println!("{:?}", parsed);
}
