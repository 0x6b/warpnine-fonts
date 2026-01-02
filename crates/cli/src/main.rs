use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use std::path::PathBuf;

mod calt;
mod clean;
mod condense;
mod copy_table;
mod download;
mod freeze;
mod instance;
mod ligatures;
mod merge;
mod metadata;
mod naming;
mod sans;
mod subset;

#[derive(Parser)]
#[command(name = "warpnine-fonts")]
#[command(about = "Fast font operations for Warpnine fonts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Remove build artifacts (build/ and dist/ directories)
    Clean {
        /// Build directory to clean
        #[arg(long, default_value = "build")]
        build_dir: PathBuf,
        /// Dist directory to clean
        #[arg(long, default_value = "dist")]
        dist_dir: PathBuf,
    },
    /// Download source fonts (Recursive VF, Noto CJK)
    Download {
        /// Build directory to download to
        #[arg(long, default_value = "build")]
        build_dir: PathBuf,
    },
    /// Copy GSUB table from source font to target font
    CopyGsub {
        /// Source font file
        #[arg(long)]
        from: PathBuf,
        /// Target font file
        #[arg(long)]
        to: PathBuf,
    },
    /// Remove three-backtick ligature from fonts
    RemoveLigatures {
        /// Font files to process
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
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
    /// Freeze OpenType features into fonts
    Freeze {
        /// Features to freeze (e.g., ss01,ss02,rvrn)
        #[arg(short, long, value_delimiter = ',')]
        features: Vec<String>,
        /// Auto-prepend 'rvrn' feature (Python compatibility)
        #[arg(long)]
        auto_rvrn: bool,
        /// Font files to process
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Create static instance from variable font
    Instance {
        /// Axis values in format TAG=VALUE (e.g., wght=700)
        #[arg(short, long = "axis", value_parser = parse_axis)]
        axes: Vec<(String, f32)>,
        /// Input variable font
        #[arg(required = true)]
        input: PathBuf,
        /// Output static font
        #[arg(required = true)]
        output: PathBuf,
    },
    /// Create multiple static instances from variable font (parallel batch)
    InstanceBatch {
        /// Input variable font
        #[arg(long)]
        input: PathBuf,
        /// Output directory
        #[arg(long, default_value = "dist")]
        output_dir: PathBuf,
        /// Instance definitions: NAME:TAG=VAL,TAG=VAL (can be repeated)
        #[arg(short, long = "instance", value_parser = parse_instance_def)]
        instances: Vec<(String, Vec<(String, f32)>)>,
    },
    /// Merge multiple fonts into one
    Merge {
        /// Input font files to merge
        #[arg(required = true, num_args = 2..)]
        inputs: Vec<PathBuf>,
        /// Output font file
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Merge multiple base fonts with a fallback font (parallel batch)
    MergeBatch {
        /// Base font files to merge
        #[arg(required = true)]
        base_fonts: Vec<PathBuf>,
        /// Fallback font to merge into each base font
        #[arg(short, long)]
        fallback: PathBuf,
        /// Output directory
        #[arg(short, long, default_value = "dist")]
        output_dir: PathBuf,
    },
    /// Create WarpnineSans fonts from Recursive VF
    CreateSans {
        /// Input variable font
        #[arg(long)]
        input: PathBuf,
        /// Output directory
        #[arg(long, default_value = "dist")]
        output_dir: PathBuf,
    },
    /// Create WarpnineSansCondensed fonts from Recursive VF
    CreateCondensed {
        /// Input variable font
        #[arg(long)]
        input: PathBuf,
        /// Output directory
        #[arg(long, default_value = "dist")]
        output_dir: PathBuf,
        /// Horizontal scale factor (default: 0.90)
        #[arg(long, default_value = "0.90")]
        scale: f32,
    },
    /// Set name table entries (family, style, postscript name, copyright)
    SetName {
        /// Font family name (e.g., "Warpnine Mono")
        #[arg(long)]
        family: String,
        /// Font style (e.g., "Regular", "Bold")
        #[arg(long)]
        style: String,
        /// PostScript family name (optional, defaults to family without spaces)
        #[arg(long)]
        postscript_family: Option<String>,
        /// Additional copyright text to append
        #[arg(long)]
        copyright_extra: Option<String>,
        /// Font files to process
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Fix calt/rclt feature registration across all scripts
    FixCalt {
        /// Font files to process
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
}

fn parse_axis(s: &str) -> Result<(String, f32), String> {
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid axis format '{}', expected TAG=VALUE", s));
    }
    let value: f32 = parts[1]
        .parse()
        .map_err(|_| format!("Invalid value '{}' for axis '{}'", parts[1], parts[0]))?;
    Ok((parts[0].to_string(), value))
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Clean {
            build_dir,
            dist_dir,
        } => {
            clean::clean(&build_dir, &dist_dir)?;
        }
        Commands::Download { build_dir } => {
            download::download(&build_dir)?;
        }
        Commands::CopyGsub { from, to } => {
            copy_table::copy_gsub(&from, &to)?;
        }
        Commands::RemoveLigatures { files } => {
            let results: Vec<_> = files
                .par_iter()
                .map(|path| {
                    println!("Processing {}", path.display());
                    ligatures::remove_grave_ligature(path)
                        .with_context(|| format!("Failed to process {}", path.display()))
                })
                .collect();

            let mut success = 0;
            let mut failed = 0;
            for result in results {
                match result {
                    Ok(_) => success += 1,
                    Err(e) => {
                        eprintln!("{e:?}");
                        failed += 1;
                    }
                }
            }
            println!("Remove ligatures: {success} succeeded, {failed} failed");
        }
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
        Commands::Freeze {
            features,
            auto_rvrn,
            files,
        } => {
            freeze::freeze_features(&files, &features, auto_rvrn)?;
        }
        Commands::Instance {
            axes,
            input,
            output,
        } => {
            instance::create_instance(&input, &output, &axes)?;
        }
        Commands::Merge { inputs, output } => {
            merge::merge_fonts(&inputs, &output)?;
        }
        Commands::MergeBatch {
            base_fonts,
            fallback,
            output_dir,
        } => {
            merge::merge_batch(&base_fonts, &fallback, &output_dir)?;
        }
        Commands::CreateSans { input, output_dir } => {
            sans::create_sans(&input, &output_dir)?;
        }
        Commands::CreateCondensed {
            input,
            output_dir,
            scale,
        } => {
            condense::create_condensed(&input, &output_dir, scale)?;
        }
        Commands::SetName {
            family,
            style,
            postscript_family,
            copyright_extra,
            files,
        } => {
            let font_naming = naming::FontNaming {
                family,
                style,
                postscript_family,
                copyright_extra,
            };

            let results: Vec<_> = files
                .par_iter()
                .map(|path| {
                    naming::set_name(path, &font_naming)
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
            println!("Set name: {success} succeeded, {failed} failed");
        }
        Commands::FixCalt { files } => {
            let results: Vec<_> = files
                .par_iter()
                .map(|path| {
                    calt::fix_calt_registration(path)
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
            println!("Fix calt: {success} succeeded, {failed} failed");
        }
    }

    Ok(())
}
