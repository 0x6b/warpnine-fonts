use anyhow::Result;
use clap::Parser;
use env_logger::init;
use warpnine_fonts_cli::cli::Cli;

fn main() -> Result<()> {
    init();
    Cli::parse().command.run()
}
