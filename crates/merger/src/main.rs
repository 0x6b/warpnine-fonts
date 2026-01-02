use std::{
    fs::{read, write},
    path::PathBuf,
};

use clap::Parser;
use warpnine_font_merger::{Merger, Options, Result};

#[derive(Parser)]
#[command(name = "font-merger")]
#[command(about = "Merge multiple fonts into one", long_about = None)]
struct Cli {
    /// Input font files to merge
    #[arg(required = true)]
    input_files: Vec<PathBuf>,

    /// Output font file
    #[arg(short, long, default_value = "merged.ttf")]
    output: PathBuf,

    /// Comma-separated list of tables to drop
    #[arg(long, value_delimiter = ',')]
    drop_tables: Vec<String>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("Merging {} fonts into {}", cli.input_files.len(), cli.output.display());
    }

    let font_data: Vec<Vec<u8>> = cli
        .input_files
        .iter()
        .map(|path| {
            if cli.verbose {
                eprintln!("  Reading: {}", path.display());
            }
            read(path).map_err(warpnine_font_merger::MergeError::Io)
        })
        .collect::<Result<Vec<_>>>()?;

    let font_refs: Vec<&[u8]> = font_data.iter().map(Vec::as_slice).collect();

    let options = Options::new().drop_tables(cli.drop_tables).verbose(cli.verbose);

    let merger = Merger::new(options);
    let merged = merger.merge(&font_refs)?;

    write(&cli.output, &merged)?;

    if cli.verbose {
        eprintln!("Wrote {} bytes to {}", merged.len(), cli.output.display());
    } else {
        println!("{}", cli.output.display());
    }

    Ok(())
}
