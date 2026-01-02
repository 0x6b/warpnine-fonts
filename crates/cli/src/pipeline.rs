//! Full build pipeline for Warpnine fonts.
//!
//! Rust equivalent of the Python pipeline in warpnine_fonts/pipeline/runner.py

use crate::build_vf::build_warpnine_mono_vf;
use crate::condense::create_condensed;
use crate::copy_table::copy_gsub;
use crate::freeze::freeze_features;
use crate::instance::{create_instances_batch, InstanceDef};
use crate::ligatures::remove_grave_ligature;
use crate::merge::merge_batch;
use crate::metadata::{parse_version_string, set_monospace, set_version};
use crate::naming::{set_name, FontNaming};
use crate::sans::create_sans;
use crate::subset::subset_japanese;
use crate::{clean, download};

use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Duotone instance definitions (16 styles: 8 weights × 2 italics)
const DUOTONE_INSTANCES: &[(&str, &[(&str, f32)])] = &[
    ("Light", &[("MONO", 1.0), ("CASL", 0.0), ("wght", 300.0), ("slnt", 0.0), ("CRSV", 0.5)]),
    ("LightItalic", &[("MONO", 1.0), ("CASL", 0.0), ("wght", 300.0), ("slnt", -15.0), ("CRSV", 1.0)]),
    ("Regular", &[("MONO", 1.0), ("CASL", 0.0), ("wght", 400.0), ("slnt", 0.0), ("CRSV", 0.5)]),
    ("Italic", &[("MONO", 1.0), ("CASL", 0.0), ("wght", 400.0), ("slnt", -15.0), ("CRSV", 1.0)]),
    ("Medium", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 500.0), ("slnt", 0.0), ("CRSV", 0.5)]),
    ("MediumItalic", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 500.0), ("slnt", -15.0), ("CRSV", 1.0)]),
    ("SemiBold", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 600.0), ("slnt", 0.0), ("CRSV", 0.5)]),
    ("SemiBoldItalic", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 600.0), ("slnt", -15.0), ("CRSV", 1.0)]),
    ("Bold", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 700.0), ("slnt", 0.0), ("CRSV", 0.5)]),
    ("BoldItalic", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 700.0), ("slnt", -15.0), ("CRSV", 1.0)]),
    ("ExtraBold", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 800.0), ("slnt", 0.0), ("CRSV", 0.5)]),
    ("ExtraBoldItalic", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 800.0), ("slnt", -15.0), ("CRSV", 1.0)]),
    ("Black", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 900.0), ("slnt", 0.0), ("CRSV", 0.5)]),
    ("BlackItalic", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 900.0), ("slnt", -15.0), ("CRSV", 1.0)]),
    ("ExtraBlack", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 1000.0), ("slnt", 0.0), ("CRSV", 0.5)]),
    ("ExtraBlackItalic", &[("MONO", 1.0), ("CASL", 1.0), ("wght", 1000.0), ("slnt", -15.0), ("CRSV", 1.0)]),
];

/// Features to freeze in static mono fonts
const MONO_FEATURES: &[&str] = &[
    "dlig", "ss01", "ss02", "ss03", "ss04", "ss05", "ss06", "ss07", "ss08", 
    "ss10", "ss11", "ss12", "pnum", "liga",
];

/// Pipeline execution context
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
}

/// Run a named pipeline step with timing
fn run_step(name: &str, step_num: usize, total: usize, ctx: &PipelineContext, f: impl Fn(&PipelineContext) -> Result<()>) -> Result<()> {
    println!("\n[{}/{}] {}", step_num, total, name);
    let start = Instant::now();
    f(ctx)?;
    println!("  ✓ {} ({:.2}s)", name, start.elapsed().as_secs_f64());
    Ok(())
}

// ============================================================================
// Pipeline Steps
// ============================================================================

fn step_clean(ctx: &PipelineContext) -> Result<()> {
    clean::clean(&ctx.build_dir, &ctx.dist_dir)
}

fn step_download(ctx: &PipelineContext) -> Result<()> {
    download::download(&ctx.build_dir)
}

fn step_extract_duotone(ctx: &PipelineContext) -> Result<()> {
    println!("  Extracting 16 Duotone instances from Recursive VF...");
    
    let instances: Vec<InstanceDef> = DUOTONE_INSTANCES
        .iter()
        .map(|(name, axes)| InstanceDef {
            name: format!("RecMonoDuotone-{}", name),
            axes: axes.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        })
        .collect();
    
    create_instances_batch(&ctx.recursive_vf, &ctx.build_dir, &instances)
}

fn step_remove_ligatures(ctx: &PipelineContext) -> Result<()> {
    let fonts: Vec<PathBuf> = glob_fonts(&ctx.build_dir, "RecMonoDuotone-*.ttf")?;
    println!("  Removing triple-backtick ligature from {} fonts...", fonts.len());
    
    let results: Vec<_> = fonts.par_iter()
        .map(|path| remove_grave_ligature(path))
        .collect();
    
    check_results(&results, "remove ligatures")
}

fn step_extract_noto_weights(ctx: &PipelineContext) -> Result<()> {
    println!("  Extracting Regular (400) and Bold (700) from Noto CJK VF...");
    
    let instances = vec![
        InstanceDef { name: "Noto-400".to_string(), axes: vec![("wght".to_string(), 400.0)] },
        InstanceDef { name: "Noto-700".to_string(), axes: vec![("wght".to_string(), 700.0)] },
    ];
    
    create_instances_batch(&ctx.noto_vf, &ctx.build_dir, &instances)
}

fn step_subset_noto(ctx: &PipelineContext) -> Result<()> {
    println!("  Subsetting Noto fonts to Japanese Unicode ranges...");
    
    for weight in ["400", "700"] {
        let input = ctx.build_dir.join(format!("Noto-{}.ttf", weight));
        let output = ctx.build_dir.join(format!("Noto-{}-subset.ttf", weight));
        subset_japanese(&input, &output)?;
    }
    Ok(())
}

fn step_merge(ctx: &PipelineContext) -> Result<()> {
    println!("  Merging Duotone + Noto CJK into WarpnineMono...");
    
    // Get all Duotone fonts
    let duotone_fonts: Vec<PathBuf> = glob_fonts(&ctx.build_dir, "RecMonoDuotone-*.ttf")?;
    
    // Use Noto-400-subset as fallback for all
    let fallback = ctx.build_dir.join("Noto-400-subset.ttf");
    
    // Merge each Duotone with Noto fallback
    std::fs::create_dir_all(&ctx.dist_dir)?;
    merge_batch(&duotone_fonts, &fallback, &ctx.dist_dir)?;
    
    // Rename merged files from RecMonoDuotone-* to WarpnineMono-*
    for font in glob_fonts(&ctx.dist_dir, "RecMonoDuotone-*.ttf")? {
        let new_name = font.file_name()
            .unwrap()
            .to_string_lossy()
            .replace("RecMonoDuotone-", "WarpnineMono-");
        let new_path = ctx.dist_dir.join(new_name);
        std::fs::rename(&font, &new_path)?;
    }
    
    Ok(())
}

fn step_set_names_mono(ctx: &PipelineContext) -> Result<()> {
    let fonts: Vec<PathBuf> = glob_fonts(&ctx.dist_dir, "WarpnineMono-*.ttf")?
        .into_iter()
        .filter(|p| !p.file_name().unwrap().to_string_lossy().contains("-VF"))
        .collect();

    println!("  Setting names for {} static mono fonts...", fonts.len());

    let results: Vec<_> = fonts
        .par_iter()
        .map(|path| {
            let style = path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .replace("WarpnineMono-", "");
            let naming = FontNaming {
                family: "Warpnine Mono".to_string(),
                style,
                postscript_family: Some("WarpnineMono".to_string()),
                copyright_extra: Some(
                    "Warpnine Mono is based on Recursive Mono Duotone and Noto Sans Mono CJK JP."
                        .to_string(),
                ),
            };
            set_name(path, &naming)
        })
        .collect();

    check_results(&results, "set names")
}

fn step_freeze_static_mono(ctx: &PipelineContext) -> Result<()> {
    let fonts: Vec<PathBuf> = glob_fonts(&ctx.dist_dir, "WarpnineMono-*.ttf")?
        .into_iter()
        .filter(|p| !p.file_name().unwrap().to_string_lossy().contains("-VF"))
        .collect();

    println!("  Freezing features in {} static mono fonts...", fonts.len());

    let features: Vec<String> = MONO_FEATURES.iter().map(|s| s.to_string()).collect();
    freeze_features(&fonts, &features, true)
}

fn step_backup_frozen(ctx: &PipelineContext) -> Result<()> {
    let backup_dir = ctx.build_dir.join("frozen");
    std::fs::create_dir_all(&backup_dir)?;
    
    let fonts: Vec<PathBuf> = glob_fonts(&ctx.dist_dir, "WarpnineMono-*.ttf")?
        .into_iter()
        .filter(|p| !p.file_name().unwrap().to_string_lossy().contains("-VF"))
        .collect();
    
    println!("  Backing up {} frozen static fonts...", fonts.len());
    
    for font in &fonts {
        let dest = backup_dir.join(font.file_name().unwrap());
        std::fs::copy(font, dest)?;
    }
    Ok(())
}

fn step_build_vf(ctx: &PipelineContext) -> Result<()> {
    let output = ctx.dist_dir.join("WarpnineMono-VF.ttf");
    build_warpnine_mono_vf(&ctx.dist_dir, &output)
}

fn step_copy_gsub(ctx: &PipelineContext) -> Result<()> {
    let vf = ctx.dist_dir.join("WarpnineMono-VF.ttf");
    println!("  Copying GSUB from Recursive VF to WarpnineMono VF...");
    copy_gsub(&ctx.recursive_vf, &vf)
}

fn step_restore_frozen(ctx: &PipelineContext) -> Result<()> {
    let backup_dir = ctx.build_dir.join("frozen");
    
    if !backup_dir.exists() {
        println!("  No backup directory found, skipping restore");
        return Ok(());
    }
    
    let backups: Vec<PathBuf> = glob_fonts(&backup_dir, "WarpnineMono-*.ttf")?;
    println!("  Restoring {} frozen static fonts...", backups.len());
    
    for backup in &backups {
        let dest = ctx.dist_dir.join(backup.file_name().unwrap());
        std::fs::copy(backup, dest)?;
    }
    Ok(())
}

fn step_set_monospace(ctx: &PipelineContext) -> Result<()> {
    let fonts: Vec<PathBuf> = glob_fonts(&ctx.dist_dir, "WarpnineMono-*.ttf")?;
    println!("  Setting monospace flags on {} fonts...", fonts.len());
    
    let results: Vec<_> = fonts.par_iter()
        .map(|path| set_monospace(path))
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
    // Sans fonts
    let sans_fonts: Vec<PathBuf> = glob_fonts(&ctx.dist_dir, "WarpnineSans-*.ttf")?;
    if !sans_fonts.is_empty() {
        println!("  Setting names for {} Sans fonts...", sans_fonts.len());
        let results: Vec<_> = sans_fonts
            .par_iter()
            .map(|path| {
                let style = path
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .replace("WarpnineSans-", "");
                let naming = FontNaming {
                    family: "Warpnine Sans".to_string(),
                    style,
                    postscript_family: Some("WarpnineSans".to_string()),
                    copyright_extra: Some("Warpnine Sans is based on Recursive.".to_string()),
                };
                set_name(path, &naming)
            })
            .collect();
        check_results(&results, "set names (Sans)")?;
    }

    // Condensed fonts
    let condensed_fonts: Vec<PathBuf> = glob_fonts(&ctx.dist_dir, "WarpnineSansCondensed-*.ttf")?;
    if !condensed_fonts.is_empty() {
        println!(
            "  Setting names for {} Condensed fonts...",
            condensed_fonts.len()
        );
        let results: Vec<_> = condensed_fonts
            .par_iter()
            .map(|path| {
                let style = path
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .replace("WarpnineSansCondensed-", "");
                let naming = FontNaming {
                    family: "Warpnine Sans Condensed".to_string(),
                    style,
                    postscript_family: Some("WarpnineSansCondensed".to_string()),
                    copyright_extra: Some("Warpnine Sans Condensed is based on Recursive.".to_string()),
                };
                set_name(path, &naming)
            })
            .collect();
        check_results(&results, "set names (Condensed)")?;
    }

    Ok(())
}

fn step_set_names_vf(ctx: &PipelineContext) -> Result<()> {
    let vf = ctx.dist_dir.join("WarpnineMono-VF.ttf");
    if !vf.exists() {
        println!("  VF not found, skipping name setting");
        return Ok(());
    }

    println!("  Setting names for VF...");
    let naming = FontNaming {
        family: "Warpnine Mono".to_string(),
        style: "Regular".to_string(), // VF default style
        postscript_family: Some("WarpnineMono".to_string()),
        copyright_extra: Some(
            "Warpnine Mono is based on Recursive Mono Duotone and Noto Sans Mono CJK JP."
                .to_string(),
        ),
    };
    set_name(&vf, &naming)
}

fn step_freeze_vf_and_sans(ctx: &PipelineContext) -> Result<()> {
    // Freeze VF
    let vf = ctx.dist_dir.join("WarpnineMono-VF.ttf");
    if vf.exists() {
        println!("  Freezing features in VF...");
        let features: Vec<String> = MONO_FEATURES.iter().map(|s| s.to_string()).collect();
        freeze_features(&[vf], &features, true)?;
    }
    
    // Freeze Sans fonts
    let sans_fonts: Vec<PathBuf> = glob_fonts(&ctx.dist_dir, "WarpnineSans-*.ttf")?;
    if !sans_fonts.is_empty() {
        println!("  Freezing features in {} Sans fonts...", sans_fonts.len());
        let features: Vec<String> = MONO_FEATURES.iter().map(|s| s.to_string()).collect();
        freeze_features(&sans_fonts, &features, true)?;
    }
    
    // Freeze Condensed fonts  
    let condensed_fonts: Vec<PathBuf> = glob_fonts(&ctx.dist_dir, "WarpnineSansCondensed-*.ttf")?;
    if !condensed_fonts.is_empty() {
        println!("  Freezing features in {} Condensed fonts...", condensed_fonts.len());
        let features: Vec<String> = MONO_FEATURES.iter().map(|s| s.to_string()).collect();
        freeze_features(&condensed_fonts, &features, true)?;
    }
    
    Ok(())
}

fn step_set_version(ctx: &PipelineContext) -> Result<()> {
    let fonts: Vec<PathBuf> = glob_fonts(&ctx.dist_dir, "*.ttf")?;
    println!("  Setting version on {} fonts...", fonts.len());
    
    let (date, version_tag) = parse_version_string(ctx.version.as_deref())?;
    
    let results: Vec<_> = fonts.par_iter()
        .map(|path| set_version(path, date, &version_tag))
        .collect();
    
    check_results(&results, "set version")
}

// ============================================================================
// Helper Functions
// ============================================================================

fn glob_fonts(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let glob_pattern = dir.join(pattern);
    let paths: Vec<PathBuf> = glob::glob(glob_pattern.to_str().unwrap())
        .with_context(|| format!("Invalid glob pattern: {}", pattern))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(paths)
}

fn check_results<T>(results: &[Result<T>], operation: &str) -> Result<()> {
    let failed: Vec<_> = results.iter().filter(|r| r.is_err()).collect();
    if !failed.is_empty() {
        anyhow::bail!("{} failed for {} files", operation, failed.len());
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
    
    let steps: Vec<(&str, fn(&PipelineContext) -> Result<()>)> = vec![
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
        ("create-condensed", step_create_condensed),
        ("create-sans", step_create_sans),
        ("set-names-sans", step_set_names_sans),
        ("freeze-vf-and-sans", step_freeze_vf_and_sans),
        ("set-version", step_set_version),
    ];
    
    let total = steps.len();
    for (i, (name, step_fn)) in steps.iter().enumerate() {
        run_step(name, i + 1, total, &ctx, step_fn)?;
    }
    
    println!("\n═══════════════════════════════════════════════════════════════════════════════");
    println!("✨ Build complete in {:.2}s", start.elapsed().as_secs_f64());
    println!("   Output: {}", ctx.dist_dir.display());
    
    // Print summary
    let mono_count = glob_fonts(&ctx.dist_dir, "WarpnineMono-*.ttf")?.len();
    let sans_count = glob_fonts(&ctx.dist_dir, "WarpnineSans-*.ttf")?.len();
    let condensed_count = glob_fonts(&ctx.dist_dir, "WarpnineSansCondensed-*.ttf")?.len();
    
    println!("   Fonts: {} Mono, {} Sans, {} Condensed", mono_count, sans_count, condensed_count);
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
    
    let steps: Vec<(&str, fn(&PipelineContext) -> Result<()>)> = vec![
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
        ("set-version", step_set_version),
    ];
    
    let total = steps.len();
    for (i, (name, step_fn)) in steps.iter().enumerate() {
        run_step(name, i + 1, total, &ctx, step_fn)?;
    }
    
    println!("\n═══════════════════════════════════════════════════════════════════════════════");
    println!("✨ Mono build complete in {:.2}s", start.elapsed().as_secs_f64());
    println!("   Output: {}", ctx.dist_dir.display());
    
    let mono_count = glob_fonts(&ctx.dist_dir, "WarpnineMono-*.ttf")?.len();
    println!("   Fonts: {} Mono", mono_count);
    println!("═══════════════════════════════════════════════════════════════════════════════");
    
    Ok(())
}
