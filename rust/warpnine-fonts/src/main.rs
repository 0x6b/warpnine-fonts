use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use std::path::PathBuf;

mod metadata;
mod subset;

#[derive(Parser)]
#[command(name = "warpnine-fonts")]
#[command(about = "Fast font metadata operations for Warpnine fonts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set monospace flags on font files
    SetMonospace {
        /// Font files to process
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Set version date on font files
    SetVersion {
        /// Version string (YYYY-MM-DD or YYYY-MM-DD.N)
        #[arg(short, long)]
        version: Option<String>,
        /// Font files to process
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Subset font to Japanese Unicode ranges
    SubsetJapanese {
        /// Input font file
        #[arg(required = true)]
        input: PathBuf,
        /// Output font file
        #[arg(required = true)]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SetMonospace { files } => {
            let results: Vec<_> = files
                .par_iter()
                .map(|path| {
                    metadata::set_monospace(path)
                        .with_context(|| format!("Failed to process {}", path.display()))
                })
                .collect();

            let mut success = 0;
            let mut failed = 0;
            for result in results {
                match result {
                    Ok(()) => success += 1,
                    Err(e) => {
                        eprintln!("{e:?}");
                        failed += 1;
                    }
                }
            }
            println!("Set monospace: {success} succeeded, {failed} failed");
        }
        Commands::SetVersion { version, files } => {
            let (date, version_tag) = metadata::parse_version_string(version.as_deref())?;

            let results: Vec<_> = files
                .par_iter()
                .map(|path| {
                    metadata::set_version(path, date, &version_tag)
                        .with_context(|| format!("Failed to process {}", path.display()))
                })
                .collect();

            let mut success = 0;
            let mut failed = 0;
            for result in results {
                match result {
                    Ok(()) => success += 1,
                    Err(e) => {
                        eprintln!("{e:?}");
                        failed += 1;
                    }
                }
            }
            println!("Set version {version_tag}: {success} succeeded, {failed} failed");
        }
        Commands::SubsetJapanese { input, output } => {
            subset::subset_japanese(&input, &output)?;
        }
    }

    Ok(())
}
