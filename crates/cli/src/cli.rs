//! CLI definitions and command dispatch.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use warpnine_core::{build_all, build_condensed, build_mono, build_sans, pipeline::{clean, download}};

#[derive(Parser)]
#[command(name = "warpnine-fonts")]
#[command(about = "Build Warpnine fonts from Recursive and Noto CJK sources")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, clap::Args)]
pub struct BuildArgs {
    #[arg(long, default_value = "build")]
    pub build_dir: PathBuf,
    #[arg(long, default_value = "dist")]
    pub dist_dir: PathBuf,
    #[arg(short, long)]
    pub version: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    Build {
        #[command(flatten)]
        args: BuildArgs,
    },
    BuildMono {
        #[command(flatten)]
        args: BuildArgs,
    },
    BuildSans {
        #[command(flatten)]
        args: BuildArgs,
    },
    BuildCondensed {
        #[command(flatten)]
        args: BuildArgs,
    },
    Download {
        #[arg(long, default_value = "build")]
        build_dir: PathBuf,
    },
    Clean {
        #[arg(long, default_value = "build")]
        build_dir: PathBuf,
        #[arg(long, default_value = "dist")]
        dist_dir: PathBuf,
    },
    #[command(subcommand, hide = true)]
    Dev(crate::dev::DevCommands),
}

impl Commands {
    pub fn run(self) -> Result<()> {
        match self {
            Commands::Build { args } => {
                build_all(&args.build_dir, &args.dist_dir, args.version)?;
            }
            Commands::BuildMono { args } => {
                build_mono(&args.build_dir, &args.dist_dir, args.version)?;
            }
            Commands::BuildSans { args } => {
                build_sans(&args.build_dir, &args.dist_dir, args.version)?;
            }
            Commands::BuildCondensed { args } => {
                build_condensed(&args.build_dir, &args.dist_dir, args.version)?;
            }
            Commands::Download { build_dir } => {
                download(&build_dir)?;
            }
            Commands::Clean { build_dir, dist_dir } => {
                clean(&build_dir, &dist_dir)?;
            }
            Commands::Dev(dev) => dev.run()?,
        }
        Ok(())
    }
}
