//! Build commands for Warpnine fonts.
//!
//! Contains build_all, build_mono pipeline logic (from pipeline.rs) and
//! build_warpnine_mono_vf (from build_vf.rs).

use std::{
    fs::{copy, create_dir_all, rename, write},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, Result, bail};
use font_instancer::AxisLocation;
use rayon::prelude::*;
use warpnine_font_vf_builder::{Axis, DesignSpace, Instance, Source, build_variable_font};

use crate::{
    FontVersion, MonospaceSettings, Subsetter,
    freeze::{AutoRvrn, freeze_features},
    instance::{InstanceDef, create_instances_batch},
    io::{check_results, glob_fonts, read_font, write_font},
    merge::merge_batch,
    styles::{MONO_FEATURES, MONO_STYLES, SANS_FEATURES, duotone_casl},
    warpnine::{
        condense::create_condensed,
        ligatures::remove_grave_ligature,
        naming::{FontNaming, set_name, set_names_for_pattern},
        sans::create_sans,
    },
};
use read_fonts::types::Tag;
use warpnine_font_ops::copy_table;

use super::{clean, download};

// ============================================================================
// Variable Font Building (from build_vf.rs)
// ============================================================================

/// WarpnineMono designspace configuration.
///
/// This matches the Python configuration in warpnine_fonts/config/instances.py
/// and warpnine_fonts/operations/variable.py.
pub fn warpnine_mono_designspace(dist_dir: &Path) -> DesignSpace {
    let axes = vec![
        Axis::new("wght", "Weight", 300.0, 400.0, 1000.0),
        Axis::new("ital", "Italic", 0.0, 0.0, 1.0),
    ];

    let sources: Vec<Source> = MONO_STYLES
        .iter()
        .map(|style| {
            Source::new(
                dist_dir.join(format!("WarpnineMono-{}.ttf", style.name)),
                vec![("wght", style.weight.0), ("ital", style.slant.ital())],
            )
            .with_style_name(&style.display_name())
        })
        .collect();

    let instances: Vec<Instance> = MONO_STYLES
        .iter()
        .map(|style| {
            Instance::new(
                &style.display_name(),
                vec![("wght", style.weight.0), ("ital", style.slant.ital())],
            )
        })
        .collect();

    DesignSpace::new(axes, sources).with_instances(instances)
}

/// Build WarpnineMono variable font.
pub fn build_warpnine_mono_vf(dist_dir: &Path, output: &Path) -> Result<()> {
    println!("Building WarpnineMono variable font...");

    let designspace = warpnine_mono_designspace(dist_dir);

    // Verify all source fonts exist
    for source in &designspace.sources {
        if !source.path.exists() {
            bail!("Source font not found: {}", source.path.display());
        }
    }

    println!("  Sources: {} masters", designspace.sources.len());
    println!("  Axes: wght (300-1000), ital (0-1)");

    let vf_data =
        build_variable_font(&designspace).with_context(|| "Failed to build variable font")?;

    write(output, &vf_data).with_context(|| format!("Failed to write {}", output.display()))?;

    let size_mb = vf_data.len() as f64 / 1024.0 / 1024.0;
    println!("  Output: {} ({size_mb:.2} MB)", output.display());

    Ok(())
}

// ============================================================================
// Pipeline (from pipeline.rs)
// ============================================================================

type PipelineStep = (&'static str, fn(&PipelineContext) -> Result<()>);

const MONO_STEPS: &[PipelineStep] = &[
    ("clean", step_clean),
    ("download", step_download),
    ("extract-duotone", step_extract_duotone),
    ("remove-ligatures", step_remove_ligatures),
    ("extract-noto-weights", step_extract_noto_weights),
    ("subset-noto", step_subset_noto),
    ("merge", step_merge),
    ("set-names-mono", step_set_names_mono),
    ("freeze-static-mono", step_freeze_static_mono),
    ("backup-frozen", step_backup_frozen),
    ("build-vf", step_build_vf),
    ("copy-gsub", step_copy_gsub),
    ("restore-frozen", step_restore_frozen),
    ("set-names-vf", step_set_names_vf),
    ("set-monospace", step_set_monospace),
];

const SANS_STEPS: &[PipelineStep] = &[
    ("create-condensed", step_create_condensed),
    ("create-sans", step_create_sans),
    ("set-names-sans", step_set_names_sans),
    ("freeze-vf-and-sans", step_freeze_vf_and_sans),
];

const FINAL_STEPS: &[PipelineStep] = &[("set-version", step_set_version)];

const SANS_ONLY_STEPS: &[PipelineStep] = &[
    ("download", step_download),
    ("create-sans", step_create_sans),
    ("set-names-sans-only", step_set_names_sans_only),
    ("freeze-sans", step_freeze_sans),
    ("set-version", step_set_version),
];

const CONDENSED_ONLY_STEPS: &[PipelineStep] = &[
    ("download", step_download),
    ("create-condensed", step_create_condensed),
    ("set-names-condensed-only", step_set_names_condensed_only),
    ("freeze-condensed", step_freeze_condensed),
    ("set-version", step_set_version),
];

/// Pipeline execution context with helper methods.
pub struct PipelineContext {
    pub build_dir: PathBuf,
    pub dist_dir: PathBuf,
    pub recursive_vf: PathBuf,
    pub noto_vf: PathBuf,
    pub version: Option<String>,
}

impl PipelineContext {
    pub fn new(build_dir: PathBuf, dist_dir: PathBuf, version: Option<String>) -> Self {
        let recursive_vf = build_dir.join("Recursive_VF_1.085.ttf");
        let noto_vf = build_dir.join("NotoSansMonoCJKjp-VF.ttf");
        Self {
            build_dir,
            dist_dir,
            recursive_vf,
            noto_vf,
            version,
        }
    }

    /// Get fonts matching a pattern in the build directory.
    pub fn build_fonts(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        glob_fonts(&self.build_dir, pattern)
    }

    /// Get fonts matching a pattern in the dist directory.
    pub fn dist_fonts(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        glob_fonts(&self.dist_dir, pattern)
    }

    /// Get static mono fonts (excluding VF) in the dist directory.
    pub fn static_mono_fonts(&self) -> Result<Vec<PathBuf>> {
        Ok(self
            .dist_fonts("WarpnineMono-*.ttf")?
            .into_iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| !s.contains("-VF"))
                    .unwrap_or(false)
            })
            .collect())
    }

    /// Path to the variable font output.
    pub fn vf_output(&self) -> PathBuf {
        self.dist_dir.join("WarpnineMono-VF.ttf")
    }

    /// Path to the frozen fonts backup directory.
    pub fn frozen_backup_dir(&self) -> PathBuf {
        self.build_dir.join("frozen")
    }
}

/// Run a named pipeline step with timing
fn run_step(
    name: &str,
    step_num: usize,
    total: usize,
    ctx: &PipelineContext,
    f: impl Fn(&PipelineContext) -> Result<()>,
) -> Result<()> {
    println!("\n[{step_num}/{total}] {name}");
    let start = Instant::now();
    f(ctx)?;
    println!("  ✓ {name} ({:.2}s)", start.elapsed().as_secs_f64());
    Ok(())
}

// ============================================================================
// Pipeline Steps
// ============================================================================

fn step_clean(ctx: &PipelineContext) -> Result<()> {
    clean(&ctx.build_dir, &ctx.dist_dir)
}

fn step_download(ctx: &PipelineContext) -> Result<()> {
    download(&ctx.build_dir)
}

fn step_extract_duotone(ctx: &PipelineContext) -> Result<()> {
    println!("  Extracting 16 Duotone instances from Recursive VF...");

    let instances: Vec<InstanceDef> = MONO_STYLES
        .iter()
        .map(|style| InstanceDef {
            name: format!("RecMonoDuotone-{}", style.name),
            axes: vec![
                AxisLocation::new("MONO", 1.0),
                AxisLocation::new("CASL", duotone_casl(style.weight.0)),
                AxisLocation::new("wght", style.weight.0),
                AxisLocation::new("slnt", style.slant.slnt()),
                AxisLocation::new("CRSV", style.slant.crsv()),
            ],
        })
        .collect();

    create_instances_batch(&ctx.recursive_vf, &ctx.build_dir, &instances)
}

fn step_remove_ligatures(ctx: &PipelineContext) -> Result<()> {
    let fonts = ctx.build_fonts("RecMonoDuotone-*.ttf")?;
    println!("  Removing triple-backtick ligature from {} fonts...", fonts.len());

    let results: Vec<_> = fonts.par_iter().map(|path| remove_grave_ligature(path)).collect();
    check_results(&results, "remove ligatures")
}

fn step_extract_noto_weights(ctx: &PipelineContext) -> Result<()> {
    println!("  Extracting Regular (400) and Bold (700) from Noto CJK VF...");

    let instances = vec![
        InstanceDef {
            name: "Noto-400".to_string(),
            axes: vec![AxisLocation::new("wght", 400.0)],
        },
        InstanceDef {
            name: "Noto-700".to_string(),
            axes: vec![AxisLocation::new("wght", 700.0)],
        },
    ];

    create_instances_batch(&ctx.noto_vf, &ctx.build_dir, &instances)
}

fn step_subset_noto(ctx: &PipelineContext) -> Result<()> {
    println!("  Subsetting Noto fonts to Japanese Unicode ranges...");

    for weight in ["400", "700"] {
        let input = ctx.build_dir.join(format!("Noto-{weight}.ttf"));
        let output = ctx.build_dir.join(format!("Noto-{weight}-subset.ttf"));
        let data = read_font(&input)?;
        let subset_data = Subsetter::japanese().subset(&data)?;
        write_font(&output, subset_data)?;
    }
    Ok(())
}

fn step_merge(ctx: &PipelineContext) -> Result<()> {
    println!("  Merging Duotone + Noto CJK into WarpnineMono...");

    let duotone_fonts = ctx.build_fonts("RecMonoDuotone-*.ttf")?;
    let fallback = ctx.build_dir.join("Noto-400-subset.ttf");

    create_dir_all(&ctx.dist_dir)?;
    merge_batch(&duotone_fonts, &fallback, &ctx.dist_dir)?;

    // Rename merged files from RecMonoDuotone-* to WarpnineMono-*
    for font in ctx.dist_fonts("RecMonoDuotone-*.ttf")? {
        let new_name = font
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", font.display()))?
            .replace("RecMonoDuotone-", "WarpnineMono-");
        let new_path = ctx.dist_dir.join(new_name);
        rename(&font, &new_path)?;
    }

    Ok(())
}

fn step_set_names_mono(ctx: &PipelineContext) -> Result<()> {
    const MONO_COPYRIGHT: &str =
        "Warpnine Mono is based on Recursive Mono Duotone and Noto Sans Mono CJK JP.";

    let fonts = ctx.static_mono_fonts()?;
    println!("  Setting names for {} static mono fonts...", fonts.len());

    let results: Vec<_> = fonts
        .par_iter()
        .map(|path| {
            let style = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.strip_prefix("WarpnineMono-").unwrap_or(s))
                .unwrap_or_default()
                .to_string();

            let naming = FontNaming {
                family: "Warpnine Mono".to_string(),
                style,
                postscript_family: Some("WarpnineMono".to_string()),
                copyright_extra: Some(MONO_COPYRIGHT.to_string()),
            };

            set_name(path, &naming)
        })
        .collect();

    check_results(&results, "set names (mono)")
}

fn step_freeze_static_mono(ctx: &PipelineContext) -> Result<()> {
    let fonts = ctx.static_mono_fonts()?;
    println!("  Freezing features in {} static mono fonts...", fonts.len());
    freeze_features(&fonts, MONO_FEATURES, AutoRvrn::Enabled)
}

fn step_backup_frozen(ctx: &PipelineContext) -> Result<()> {
    let backup_dir = ctx.frozen_backup_dir();
    create_dir_all(&backup_dir)?;

    let fonts = ctx.static_mono_fonts()?;
    println!("  Backing up {} frozen static fonts...", fonts.len());

    for font in &fonts {
        let file_name = font
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", font.display()))?;
        copy(font, backup_dir.join(file_name))?;
    }
    Ok(())
}

fn step_build_vf(ctx: &PipelineContext) -> Result<()> {
    build_warpnine_mono_vf(&ctx.dist_dir, &ctx.vf_output())
}

fn step_copy_gsub(ctx: &PipelineContext) -> Result<()> {
    println!("  Copying GSUB from Recursive VF to WarpnineMono VF...");
    let source = &ctx.recursive_vf;
    let target = &ctx.vf_output();
    let source_data = read_font(source)?;
    let target_data = read_font(target)?;
    let new_data = copy_table(&source_data, &target_data, Tag::new(b"GSUB"))?;
    write_font(target, new_data)?;
    println!("Copied GSUB table from {} to {}", source.display(), target.display());
    Ok(())
}

fn step_restore_frozen(ctx: &PipelineContext) -> Result<()> {
    let backup_dir = ctx.frozen_backup_dir();

    if !backup_dir.exists() {
        println!("  No backup directory found, skipping restore");
        return Ok(());
    }

    let backups = glob_fonts(&backup_dir, "WarpnineMono-*.ttf")?;
    println!("  Restoring {} frozen static fonts...", backups.len());

    for backup in &backups {
        let file_name = backup
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", backup.display()))?;
        copy(backup, ctx.dist_dir.join(file_name))?;
    }
    Ok(())
}

fn step_set_monospace(ctx: &PipelineContext) -> Result<()> {
    let fonts = ctx.dist_fonts("WarpnineMono-*.ttf")?;
    println!("  Setting monospace flags on {} fonts...", fonts.len());

    let results: Vec<_> = fonts
        .par_iter()
        .map(|path| {
            let data = read_font(path)?;
            let new_data = MonospaceSettings::DEFAULT.apply(&data)?;
            write_font(path, new_data)?;
            Ok(())
        })
        .collect();

    check_results(&results, "set monospace")
}

fn step_create_condensed(ctx: &PipelineContext) -> Result<()> {
    create_condensed(&ctx.recursive_vf, &ctx.dist_dir, 0.90)
}

fn step_create_sans(ctx: &PipelineContext) -> Result<()> {
    create_sans(&ctx.recursive_vf, &ctx.dist_dir)
}

fn step_set_names_sans(ctx: &PipelineContext) -> Result<()> {
    set_names_for_pattern(
        &ctx.dist_dir,
        "WarpnineSans-*.ttf",
        "Warpnine Sans",
        "WarpnineSans",
        "Warpnine Sans is based on Recursive.",
        "WarpnineSans-",
    )?;

    set_names_for_pattern(
        &ctx.dist_dir,
        "WarpnineSansCondensed-*.ttf",
        "Warpnine Sans Condensed",
        "WarpnineSansCondensed",
        "Warpnine Sans Condensed is based on Recursive.",
        "WarpnineSansCondensed-",
    )?;

    Ok(())
}

fn step_set_names_sans_only(ctx: &PipelineContext) -> Result<()> {
    set_names_for_pattern(
        &ctx.dist_dir,
        "WarpnineSans-*.ttf",
        "Warpnine Sans",
        "WarpnineSans",
        "Warpnine Sans is based on Recursive.",
        "WarpnineSans-",
    )?;
    Ok(())
}

fn step_set_names_condensed_only(ctx: &PipelineContext) -> Result<()> {
    set_names_for_pattern(
        &ctx.dist_dir,
        "WarpnineSansCondensed-*.ttf",
        "Warpnine Sans Condensed",
        "WarpnineSansCondensed",
        "Warpnine Sans Condensed is based on Recursive.",
        "WarpnineSansCondensed-",
    )?;
    Ok(())
}

fn step_freeze_sans(ctx: &PipelineContext) -> Result<()> {
    let sans_fonts = ctx.dist_fonts("WarpnineSans-*.ttf")?;
    if !sans_fonts.is_empty() {
        println!("  Freezing features in {} Sans fonts...", sans_fonts.len());
        freeze_features(&sans_fonts, SANS_FEATURES, AutoRvrn::Enabled)?;
    }
    Ok(())
}

fn step_freeze_condensed(ctx: &PipelineContext) -> Result<()> {
    let condensed_fonts = ctx.dist_fonts("WarpnineSansCondensed-*.ttf")?;
    if !condensed_fonts.is_empty() {
        println!("  Freezing features in {} Condensed fonts...", condensed_fonts.len());
        freeze_features(&condensed_fonts, SANS_FEATURES, AutoRvrn::Enabled)?;
    }
    Ok(())
}

fn step_set_names_vf(ctx: &PipelineContext) -> Result<()> {
    const MONO_COPYRIGHT: &str =
        "Warpnine Mono is based on Recursive Mono Duotone and Noto Sans Mono CJK JP.";

    let count = set_names_for_pattern(
        &ctx.dist_dir,
        "WarpnineMono-VF.ttf",
        "Warpnine Mono",
        "WarpnineMono",
        MONO_COPYRIGHT,
        "WarpnineMono-",
    )?;

    if count == 0 {
        println!("  VF not found, skipping name setting");
    }

    Ok(())
}

fn step_freeze_vf_and_sans(ctx: &PipelineContext) -> Result<()> {
    let vf = ctx.vf_output();
    if vf.exists() {
        println!("  Freezing features in VF...");
        freeze_features(&[vf], MONO_FEATURES, AutoRvrn::Enabled)?;
    }

    let sans_fonts = ctx.dist_fonts("WarpnineSans-*.ttf")?;
    if !sans_fonts.is_empty() {
        println!("  Freezing features in {} Sans fonts...", sans_fonts.len());
        freeze_features(&sans_fonts, SANS_FEATURES, AutoRvrn::Enabled)?;
    }

    let condensed_fonts = ctx.dist_fonts("WarpnineSansCondensed-*.ttf")?;
    if !condensed_fonts.is_empty() {
        println!("  Freezing features in {} Condensed fonts...", condensed_fonts.len());
        freeze_features(&condensed_fonts, SANS_FEATURES, AutoRvrn::Enabled)?;
    }

    Ok(())
}

fn step_set_version(ctx: &PipelineContext) -> Result<()> {
    let fonts = ctx.dist_fonts("*.ttf")?;
    println!("  Setting version on {} fonts...", fonts.len());

    let version = FontVersion::parse(ctx.version.as_deref())?;

    let results: Vec<_> = fonts
        .par_iter()
        .map(|path| {
            let data = read_font(path)?;
            let new_data = version.apply(&data)?;
            write_font(path, new_data)?;
            Ok(())
        })
        .collect();

    check_results(&results, "set version")
}

fn run_steps(
    steps: &[PipelineStep],
    ctx: &PipelineContext,
    offset: usize,
    total: usize,
) -> Result<()> {
    for (i, (name, step_fn)) in steps.iter().enumerate() {
        run_step(name, offset + i + 1, total, ctx, step_fn)?;
    }
    Ok(())
}

// ============================================================================
// Public Pipeline Functions
// ============================================================================

/// Run the full build pipeline (all fonts)
pub fn build_all(build_dir: &Path, dist_dir: &Path, version: Option<String>) -> Result<()> {
    let ctx = PipelineContext::new(build_dir.to_path_buf(), dist_dir.to_path_buf(), version);
    let start = Instant::now();

    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("Warpnine Fonts Build Pipeline (Rust)");
    println!("═══════════════════════════════════════════════════════════════════════════════");

    let total = MONO_STEPS.len() + SANS_STEPS.len() + FINAL_STEPS.len();

    run_steps(MONO_STEPS, &ctx, 0, total)?;
    run_steps(SANS_STEPS, &ctx, MONO_STEPS.len(), total)?;
    run_steps(FINAL_STEPS, &ctx, MONO_STEPS.len() + SANS_STEPS.len(), total)?;

    println!("\n═══════════════════════════════════════════════════════════════════════════════");
    println!("✨ Build complete in {:.2}s", start.elapsed().as_secs_f64());
    println!("   Output: {}", ctx.dist_dir.display());

    let mono_count = ctx.dist_fonts("WarpnineMono-*.ttf")?.len();
    let sans_count = ctx.dist_fonts("WarpnineSans-*.ttf")?.len();
    let condensed_count = ctx.dist_fonts("WarpnineSansCondensed-*.ttf")?.len();

    println!("   Fonts: {mono_count} Mono, {sans_count} Sans, {condensed_count} Condensed");
    println!("═══════════════════════════════════════════════════════════════════════════════");

    Ok(())
}

/// Run mono-only build pipeline
pub fn build_mono(build_dir: &Path, dist_dir: &Path, version: Option<String>) -> Result<()> {
    let ctx = PipelineContext::new(build_dir.to_path_buf(), dist_dir.to_path_buf(), version);
    let start = Instant::now();

    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("Warpnine Mono Build Pipeline (Rust)");
    println!("═══════════════════════════════════════════════════════════════════════════════");

    let total = MONO_STEPS.len() + FINAL_STEPS.len();

    run_steps(MONO_STEPS, &ctx, 0, total)?;
    run_steps(FINAL_STEPS, &ctx, MONO_STEPS.len(), total)?;

    println!("\n═══════════════════════════════════════════════════════════════════════════════");
    println!("✨ Mono build complete in {:.2}s", start.elapsed().as_secs_f64());
    println!("   Output: {}", ctx.dist_dir.display());

    let mono_count = ctx.dist_fonts("WarpnineMono-*.ttf")?.len();
    println!("   Fonts: {mono_count} Mono");
    println!("═══════════════════════════════════════════════════════════════════════════════");

    Ok(())
}

/// Run sans-only build pipeline
pub fn build_sans(build_dir: &Path, dist_dir: &Path, version: Option<String>) -> Result<()> {
    let ctx = PipelineContext::new(build_dir.to_path_buf(), dist_dir.to_path_buf(), version);
    let start = Instant::now();

    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("Warpnine Sans Build Pipeline (Rust)");
    println!("═══════════════════════════════════════════════════════════════════════════════");

    let total = SANS_ONLY_STEPS.len();

    run_steps(SANS_ONLY_STEPS, &ctx, 0, total)?;

    println!("\n═══════════════════════════════════════════════════════════════════════════════");
    println!("✨ Sans build complete in {:.2}s", start.elapsed().as_secs_f64());
    println!("   Output: {}", ctx.dist_dir.display());

    let sans_count = ctx.dist_fonts("WarpnineSans-*.ttf")?.len();
    println!("   Fonts: {sans_count} Sans");
    println!("═══════════════════════════════════════════════════════════════════════════════");

    Ok(())
}

/// Run condensed-only build pipeline
pub fn build_condensed(build_dir: &Path, dist_dir: &Path, version: Option<String>) -> Result<()> {
    let ctx = PipelineContext::new(build_dir.to_path_buf(), dist_dir.to_path_buf(), version);
    let start = Instant::now();

    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("Warpnine Sans Condensed Build Pipeline (Rust)");
    println!("═══════════════════════════════════════════════════════════════════════════════");

    let total = CONDENSED_ONLY_STEPS.len();

    run_steps(CONDENSED_ONLY_STEPS, &ctx, 0, total)?;

    println!("\n═══════════════════════════════════════════════════════════════════════════════");
    println!("✨ Condensed build complete in {:.2}s", start.elapsed().as_secs_f64());
    println!("   Output: {}", ctx.dist_dir.display());

    let condensed_count = ctx.dist_fonts("WarpnineSansCondensed-*.ttf")?.len();
    println!("   Fonts: {condensed_count} Condensed");
    println!("═══════════════════════════════════════════════════════════════════════════════");

    Ok(())
}
