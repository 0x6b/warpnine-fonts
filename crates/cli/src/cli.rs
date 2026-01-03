//! CLI definitions and command dispatch.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{build_all, build_condensed, build_mono, build_sans, clean, download};

#[derive(Parser)]
#[command(name = "warpnine-fonts")]
#[command(about = "Build Warpnine fonts from Recursive and Noto CJK sources")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Build directory arguments shared by build commands.
#[derive(Debug, Clone, clap::Args)]
pub struct BuildArgs {
    /// Build directory for intermediate files
    #[arg(long, default_value = "build")]
    pub build_dir: PathBuf,
    /// Distribution directory for output fonts
    #[arg(long, default_value = "dist")]
    pub dist_dir: PathBuf,
    /// Version string (YYYY-MM-DD or YYYY-MM-DD.N)
    #[arg(short, long)]
    pub version: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build all fonts (Mono, Sans, Condensed)
    Build {
        #[command(flatten)]
        args: BuildArgs,
    },
    /// Build WarpnineMono fonts only (static + variable)
    BuildMono {
        #[command(flatten)]
        args: BuildArgs,
    },
    /// Build WarpnineSans fonts only
    BuildSans {
        #[command(flatten)]
        args: BuildArgs,
    },
    /// Build WarpnineSansCondensed fonts only
    BuildCondensed {
        #[command(flatten)]
        args: BuildArgs,
    },
    /// Download source fonts (Recursive VF, Noto CJK)
    Download {
        /// Build directory to download to
        #[arg(long, default_value = "build")]
        build_dir: PathBuf,
    },
    /// Remove build artifacts (build/ and dist/ directories)
    Clean {
        /// Build directory to clean
        #[arg(long, default_value = "build")]
        build_dir: PathBuf,
        /// Dist directory to clean
        #[arg(long, default_value = "dist")]
        dist_dir: PathBuf,
    },
    /// Development and debugging commands
    #[command(subcommand, hide = true)]
    Dev(DevCommands),
}

/// Internal commands for development, testing, and debugging.
#[derive(Subcommand)]
pub enum DevCommands {
    /// Copy GSUB table from source font to target font
    CopyGsub {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        to: PathBuf,
    },
    /// Remove three-backtick ligature from fonts
    RemoveLigatures {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Set monospace flags on font files
    SetMonospace {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Set version date on font files
    SetVersion {
        #[arg(short, long)]
        version: Option<String>,
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Subset font to Japanese Unicode ranges
    SubsetJapanese {
        #[arg(required = true)]
        input: PathBuf,
        #[arg(required = true)]
        output: PathBuf,
    },
    /// Freeze OpenType features into fonts
    Freeze {
        #[arg(short, long, value_delimiter = ',')]
        features: Vec<String>,
        #[arg(long)]
        auto_rvrn: bool,
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Create static instance from variable font
    Instance {
        #[arg(short, long = "axis", value_parser = parse_axis)]
        axes: Vec<crate::instance::AxisLocation>,
        #[arg(required = true)]
        input: PathBuf,
        #[arg(required = true)]
        output: PathBuf,
    },
    /// Create multiple static instances from variable font
    InstanceBatch {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value = "dist")]
        output_dir: PathBuf,
        #[arg(short, long = "instance", value_parser = parse_instance_def)]
        instances: Vec<(String, Vec<crate::instance::AxisLocation>)>,
    },
    /// Merge multiple fonts into one
    Merge {
        #[arg(required = true, num_args = 2..)]
        inputs: Vec<PathBuf>,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Merge multiple base fonts with a fallback font
    MergeBatch {
        #[arg(required = true)]
        base_fonts: Vec<PathBuf>,
        #[arg(short, long)]
        fallback: PathBuf,
        #[arg(short, long, default_value = "dist")]
        output_dir: PathBuf,
    },
    /// Create WarpnineSans fonts from Recursive VF (without naming/freezing)
    CreateSans {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value = "dist")]
        output_dir: PathBuf,
    },
    /// Create WarpnineSansCondensed fonts from Recursive VF (without naming/freezing)
    CreateCondensed {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value = "dist")]
        output_dir: PathBuf,
        #[arg(long, default_value = "0.90")]
        scale: f32,
    },
    /// Set name table entries
    SetName {
        #[arg(long)]
        family: String,
        #[arg(long)]
        style: String,
        #[arg(long)]
        postscript_family: Option<String>,
        #[arg(long)]
        copyright_extra: Option<String>,
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Fix calt/rclt feature registration across all scripts
    FixCalt {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Build WarpnineMono variable font from static masters
    BuildVf {
        #[arg(long, default_value = "dist")]
        dist_dir: PathBuf,
        #[arg(long, default_value = "dist/WarpnineMono-VF.ttf")]
        output: PathBuf,
    },
}

fn parse_axis(s: &str) -> Result<crate::instance::AxisLocation, String> {
    let (tag, value_str) = s
        .split_once('=')
        .ok_or_else(|| format!("Invalid axis format '{s}', expected TAG=VALUE"))?;
    let value: f32 = value_str
        .parse()
        .map_err(|_| format!("Invalid value '{value_str}' for axis '{tag}'"))?;
    Ok(crate::instance::AxisLocation::new(tag, value))
}

fn parse_instance_def(s: &str) -> Result<(String, Vec<crate::instance::AxisLocation>), String> {
    let (name, axes_str) = s
        .split_once(':')
        .ok_or_else(|| format!("Expected NAME:TAG=VAL,TAG=VAL format, got '{s}'"))?;
    let axes: Result<Vec<_>, String> = axes_str.split(',').map(parse_axis).collect();
    Ok((name.to_string(), axes?))
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

impl DevCommands {
    pub fn run(self) -> Result<()> {
        use crate::{
            commands::build_warpnine_mono_vf,
            freeze::{AutoRvrn, freeze_features},
            instance::{InstanceDef, create_instance, create_instances_batch},
            io::{read_font, write_font, transform_font_in_place},
            merge::{merge_batch, merge_fonts},
            parallel::run_parallel,
            warpnine::{
                calt::fix_calt_registration,
                condense::create_condensed,
                ligatures::remove_grave_ligature,
                naming::{FontNaming, set_name},
                sans::create_sans,
            },
            FontVersion, MonospaceSettings, Subsetter,
        };
        use read_fonts::types::Tag;
        use warpnine_font_ops::copy_table;

        match self {
            DevCommands::CopyGsub { from, to } => {
                let source_data = read_font(&from)?;
                let target_data = read_font(&to)?;
                let new_data = copy_table(&source_data, &target_data, Tag::new(b"GSUB"))?;
                write_font(&to, new_data)?;
                println!("Copied GSUB table from {} to {}", from.display(), to.display());
            }
            DevCommands::RemoveLigatures { files } => {
                run_parallel("Remove ligatures", &files, |path| {
                    remove_grave_ligature(path)?;
                    Ok(())
                })?;
            }
            DevCommands::SetMonospace { files } => {
                run_parallel("Set monospace", &files, |path| {
                    transform_font_in_place(path, |data| MonospaceSettings::DEFAULT.apply(data))
                })?;
            }
            DevCommands::SetVersion { version, files } => {
                let font_version = FontVersion::parse(version.as_deref())?;
                let version_tag = font_version.tag.clone();
                run_parallel(&format!("Set version {version_tag}"), &files, |path| {
                    transform_font_in_place(path, |data| font_version.apply(data))
                })?;
            }
            DevCommands::SubsetJapanese { input, output } => {
                let data = read_font(&input)?;
                let subset_data = Subsetter::japanese().subset(&data)?;
                write_font(&output, subset_data)?;
                println!("Subset {} -> {}", input.display(), output.display());
            }
            DevCommands::Freeze { features, auto_rvrn, files } => {
                let auto_rvrn = if auto_rvrn { AutoRvrn::Enabled } else { AutoRvrn::Disabled };
                freeze_features(&files, &features, auto_rvrn)?;
            }
            DevCommands::Instance { axes, input, output } => {
                create_instance(&input, &output, &axes)?;
            }
            DevCommands::InstanceBatch { input, output_dir, instances } => {
                let defs: Vec<InstanceDef> = instances
                    .into_iter()
                    .map(|(name, axes)| InstanceDef::new(name, axes))
                    .collect();
                create_instances_batch(&input, &output_dir, &defs)?;
            }
            DevCommands::Merge { inputs, output } => {
                merge_fonts(&inputs, &output)?;
            }
            DevCommands::MergeBatch { base_fonts, fallback, output_dir } => {
                merge_batch(&base_fonts, &fallback, &output_dir)?;
            }
            DevCommands::CreateSans { input, output_dir } => {
                create_sans(&input, &output_dir)?;
            }
            DevCommands::CreateCondensed { input, output_dir, scale } => {
                create_condensed(&input, &output_dir, scale)?;
            }
            DevCommands::SetName {
                family,
                style,
                postscript_family,
                copyright_extra,
                files,
            } => {
                let naming = FontNaming { family, style, postscript_family, copyright_extra };
                run_parallel("Set name", &files, |path| set_name(path, &naming))?;
            }
            DevCommands::FixCalt { files } => {
                run_parallel("Fix calt", &files, fix_calt_registration)?;
            }
            DevCommands::BuildVf { dist_dir, output } => {
                build_warpnine_mono_vf(&dist_dir, &output)?;
            }
        }
        Ok(())
    }
}
