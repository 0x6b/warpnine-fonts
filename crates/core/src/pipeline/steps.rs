//! Pipeline step definitions.

use std::fs::{copy, create_dir_all, rename};

use anyhow::Result;
use rayon::prelude::*;
use read_fonts::types::Tag;
use warpnine_font_ops::copy_table;

use super::{PipelineContext, clean::clean, download::download, vf::build_warpnine_mono_vf};
use crate::{
    MonospaceSettings, Subsetter,
    freeze_batch::{AutoRvrn, freeze_features},
    instance::{AxisLocation, InstanceDef, create_instances_batch},
    io::{check_results, glob_fonts, read_font, write_font},
    merge::merge_batch,
    styles::{FeatureTag, MONO_FEATURES, MONO_STYLES, SANS_FEATURES, duotone_casl},
    warpnine::{
        condense::create_condensed,
        ligatures::remove_grave_ligature,
        naming::{FontNaming, set_name, set_names_for_pattern},
        sans::create_sans,
    },
};

pub type PipelineStep = (&'static str, fn(&PipelineContext) -> Result<()>);

pub const MONO_STEPS: &[PipelineStep] = &[
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

pub const SANS_STEPS: &[PipelineStep] = &[
    ("create-condensed", step_create_condensed),
    ("create-sans", step_create_sans),
    ("set-names-sans", step_set_names_sans),
    ("freeze-vf-and-sans", step_freeze_vf_and_sans),
];

pub const FINAL_STEPS: &[PipelineStep] = &[("set-version", step_set_version)];

pub const SANS_ONLY_STEPS: &[PipelineStep] = &[
    ("download", step_download),
    ("create-sans", step_create_sans),
    ("set-names-sans-only", step_set_names_sans_only),
    ("freeze-sans", step_freeze_sans),
    ("set-version", step_set_version),
];

pub const CONDENSED_ONLY_STEPS: &[PipelineStep] = &[
    ("download", step_download),
    ("create-condensed", step_create_condensed),
    ("set-names-condensed-only", step_set_names_condensed_only),
    ("freeze-condensed", step_freeze_condensed),
    ("set-version", step_set_version),
];

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
                AxisLocation::new("CASL", duotone_casl(style.weight.value())),
                AxisLocation::new("wght", style.weight.value()),
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

/// Freeze features in fonts matching a pattern.
/// Returns Ok(()) if pattern matches no fonts (silent skip).
fn freeze_matching(
    ctx: &PipelineContext,
    pattern: &str,
    features: &[FeatureTag],
    label: &str,
) -> Result<()> {
    let fonts = ctx.dist_fonts(pattern)?;
    if !fonts.is_empty() {
        println!("  Freezing features in {} {} fonts...", fonts.len(), label);
        freeze_features(&fonts, features, AutoRvrn::Enabled)?;
    }
    Ok(())
}

fn step_freeze_sans(ctx: &PipelineContext) -> Result<()> {
    freeze_matching(ctx, "WarpnineSans-*.ttf", SANS_FEATURES, "Sans")
}

fn step_freeze_condensed(ctx: &PipelineContext) -> Result<()> {
    freeze_matching(ctx, "WarpnineSansCondensed-*.ttf", SANS_FEATURES, "Condensed")
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

    freeze_matching(ctx, "WarpnineSans-*.ttf", SANS_FEATURES, "Sans")?;
    freeze_matching(ctx, "WarpnineSansCondensed-*.ttf", SANS_FEATURES, "Condensed")?;

    Ok(())
}

fn step_set_version(ctx: &PipelineContext) -> Result<()> {
    let fonts = ctx.dist_fonts("*.ttf")?;
    println!("  Setting version on {} fonts...", fonts.len());

    let results: Vec<_> = fonts
        .par_iter()
        .map(|path| {
            let data = read_font(path)?;
            let new_data = ctx.version.apply(&data)?;
            write_font(path, new_data)?;
            Ok(())
        })
        .collect();

    check_results(&results, "set version")
}
