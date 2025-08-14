use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    pub(crate) command: Command,
    #[arg(
        short = 'i',
        long = "input",
        help = "Input file",
        group = "metadata",
        global = true
    )]
    pub(crate) input_file: Option<PathBuf>,
    #[arg(
        short = 'd',
        long = "decompress",
        help = "Decompress input as gzip compressed data",
        global = true
    )]
    pub(crate) decompress: bool,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Show information about input")]
    Info,
    #[command(about = "Show GRZ metadata")]
    GrzMetadata,
    #[command(about = "Scramble input data")]
    Scramble,
}
