use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub(crate) command: Command,
    #[arg(
        short = 'd',
        long = "decompress",
        help = "decompress input as gzip compressed data"
    )]
    pub(crate) decompress: bool,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Show information about input")]
    Info,
    #[command(about = "Scramble input data")]
    Scramble,
}
