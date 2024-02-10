mod cli;


fn main() {
    let parsed = <cli::Cli as clap::Parser>::parse();
    println!("{:?}", parsed);
}
