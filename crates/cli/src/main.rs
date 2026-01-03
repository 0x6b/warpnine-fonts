use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::init;
use warpnine_fonts_cli::instance::AxisLocation;
use warpnine_fonts_cli::{
    calt::fix_calt_registration,
    commands::{build_all, build_mono, build_warpnine_mono_vf, clean, download},
    condense::create_condensed,
    font_ops::copy_gsub,
    freeze::{AutoRvrn, freeze_features},
    instance::{InstanceDef, create_instance, create_instances_batch},
    ligatures::remove_grave_ligature,
    merge::{merge_batch, merge_fonts},
    metadata::{parse_version_string, set_monospace, set_version},
    naming::{FontNaming, set_name},
    parallel::run_parallel,
    sans::create_sans,
    subset::subset_japanese,
};

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
        axes: Vec<AxisLocation>,
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
        instances: Vec<(String, Vec<AxisLocation>)>,
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
    /// Build WarpnineMono variable font from static masters
    BuildVf {
        /// Directory containing static master fonts
        #[arg(long, default_value = "dist")]
        dist_dir: PathBuf,
        /// Output variable font path
        #[arg(long, default_value = "dist/WarpnineMono-VF.ttf")]
        output: PathBuf,
    },
    /// Run full build pipeline (all fonts: Mono, Sans, Condensed)
    Build {
        /// Build directory
        #[arg(long, default_value = "build")]
        build_dir: PathBuf,
        /// Distribution directory
        #[arg(long, default_value = "dist")]
        dist_dir: PathBuf,
        /// Version string (YYYY-MM-DD or YYYY-MM-DD.N)
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Run mono-only build pipeline (WarpnineMono static + VF)
    BuildMono {
        /// Build directory
        #[arg(long, default_value = "build")]
        build_dir: PathBuf,
        /// Distribution directory
        #[arg(long, default_value = "dist")]
        dist_dir: PathBuf,
        /// Version string (YYYY-MM-DD or YYYY-MM-DD.N)
        #[arg(short, long)]
        version: Option<String>,
    },
}

fn parse_axis(s: &str) -> Result<AxisLocation, String> {
    let (tag, value_str) = s
        .split_once('=')
        .ok_or_else(|| format!("Invalid axis format '{s}', expected TAG=VALUE"))?;
    let value: f32 = value_str
        .parse()
        .map_err(|_| format!("Invalid value '{value_str}' for axis '{tag}'"))?;
    Ok(AxisLocation::new(tag, value))
}

/// Parse instance definition: NAME:TAG=VAL,TAG=VAL
fn parse_instance_def(s: &str) -> Result<(String, Vec<AxisLocation>), String> {
    let (name, axes_str) = s
        .split_once(':')
        .ok_or_else(|| format!("Expected NAME:TAG=VAL,TAG=VAL format, got '{s}'"))?;
    let axes: Result<Vec<AxisLocation>, String> = axes_str.split(',').map(parse_axis).collect();
    Ok((name.to_string(), axes?))
}

fn main() -> Result<()> {
    init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Clean { build_dir, dist_dir } => {
            clean(&build_dir, &dist_dir)?;
        }
        Commands::Download { build_dir } => {
            download(&build_dir)?;
        }
        Commands::CopyGsub { from, to } => {
            copy_gsub(&from, &to)?;
        }
        Commands::RemoveLigatures { files } => {
            run_parallel("Remove ligatures", &files, |path| {
                remove_grave_ligature(path)?;
                Ok(())
            })?;
        }
        Commands::SetMonospace { files } => {
            run_parallel("Set monospace", &files, set_monospace)?;
        }
        Commands::SetVersion { version, files } => {
            let (date, version_tag) = parse_version_string(version.as_deref())?;
            run_parallel(&format!("Set version {version_tag}"), &files, |path| {
                set_version(path, date, &version_tag)
            })?;
        }
        Commands::SubsetJapanese { input, output } => {
            subset_japanese(&input, &output)?;
        }
        Commands::Freeze { features, auto_rvrn, files } => {
            let auto_rvrn = if auto_rvrn { AutoRvrn::Enabled } else { AutoRvrn::Disabled };
            freeze_features(&files, &features, auto_rvrn)?;
        }
        Commands::Instance { axes, input, output } => {
            create_instance(&input, &output, &axes)?;
        }
        Commands::InstanceBatch { input, output_dir, instances } => {
            let defs: Vec<InstanceDef> = instances
                .into_iter()
                .map(|(name, axes)| InstanceDef { name, axes })
                .collect();
            create_instances_batch(&input, &output_dir, &defs)?;
        }
        Commands::Merge { inputs, output } => {
            merge_fonts(&inputs, &output)?;
        }
        Commands::MergeBatch { base_fonts, fallback, output_dir } => {
            merge_batch(&base_fonts, &fallback, &output_dir)?;
        }
        Commands::CreateSans { input, output_dir } => {
            create_sans(&input, &output_dir)?;
        }
        Commands::CreateCondensed { input, output_dir, scale } => {
            create_condensed(&input, &output_dir, scale)?;
        }
        Commands::SetName {
            family,
            style,
            postscript_family,
            copyright_extra,
            files,
        } => {
            let font_naming = FontNaming { family, style, postscript_family, copyright_extra };
            run_parallel("Set name", &files, |path| set_name(path, &font_naming))?;
        }
        Commands::FixCalt { files } => {
            run_parallel("Fix calt", &files, fix_calt_registration)?;
        }
        Commands::BuildVf { dist_dir, output } => {
            build_warpnine_mono_vf(&dist_dir, &output)?;
        }
        Commands::Build { build_dir, dist_dir, version } => {
            build_all(&build_dir, &dist_dir, version)?;
        }
        Commands::BuildMono { build_dir, dist_dir, version } => {
            build_mono(&build_dir, &dist_dir, version)?;
        }
    }

    Ok(())
}
