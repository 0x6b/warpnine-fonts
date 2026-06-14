//! Variable font building.

use std::{fs::write, path::Path};

use anyhow::{Context, Result, bail};
use warpnine_font_vf_builder::{Axis, DesignSpace, Instance, Source, build_variable_font};

use crate::styles::{MONO_STYLES, SANS_STYLES, Style};

/// Build a `wght` + `ital` designspace for a family whose static masters live
/// in `dist_dir` as `{file_prefix}{style}.ttf`.
fn family_designspace(
    dist_dir: &Path,
    styles: &[Style],
    file_prefix: &str,
    wght_max: f32,
) -> DesignSpace {
    let axes = vec![
        Axis::new("wght", "Weight", 300.0, 400.0, wght_max),
        Axis::new("ital", "Italic", 0.0, 0.0, 1.0),
    ];

    let sources: Vec<Source> = styles
        .iter()
        .map(|style| {
            Source::new(
                dist_dir.join(format!("{file_prefix}{}.ttf", style.name)),
                vec![("wght", style.weight.value()), ("ital", style.slant.ital())],
            )
            .with_style_name(&style.display_name())
        })
        .collect();

    let instances: Vec<Instance> = styles
        .iter()
        .map(|style| {
            // PostScript name matches the static instance's name ID 6, e.g.
            // "WarpnineMono-BoldItalic" (file_prefix already ends with '-').
            Instance::new(
                &style.display_name(),
                vec![("wght", style.weight.value()), ("ital", style.slant.ital())],
            )
            .with_postscript_name(&format!("{file_prefix}{}", style.name))
        })
        .collect();

    DesignSpace::new(axes, sources).with_instances(instances)
}

/// Build a variable font from `designspace`, writing the result to `output`.
fn build_family_vf(
    designspace: &DesignSpace,
    output: &Path,
    label: &str,
    wght_max: f32,
) -> Result<()> {
    println!("Building {label} variable font...");

    for source in &designspace.sources {
        if !source.path.exists() {
            bail!("Source font not found: {}", source.path.display());
        }
    }

    println!("  Sources: {} masters", designspace.sources.len());
    println!("  Axes: wght (300-{wght_max:.0}), ital (0-1)");

    let vf_data =
        build_variable_font(designspace).with_context(|| "Failed to build variable font")?;

    write(output, &vf_data).with_context(|| format!("Failed to write {}", output.display()))?;

    let size_mb = vf_data.len() as f64 / 1024.0 / 1024.0;
    println!("  Output: {} ({size_mb:.2} MB)", output.display());

    Ok(())
}

pub fn warpnine_mono_designspace(dist_dir: &Path) -> DesignSpace {
    family_designspace(dist_dir, MONO_STYLES, "WarpnineMono-", 1000.0)
}

pub fn build_warpnine_mono_vf(dist_dir: &Path, output: &Path) -> Result<()> {
    let designspace = warpnine_mono_designspace(dist_dir);
    build_family_vf(&designspace, output, "WarpnineMono", 1000.0)
}

pub fn build_warpnine_sans_vf(dist_dir: &Path, output: &Path) -> Result<()> {
    let designspace = family_designspace(dist_dir, SANS_STYLES, "WarpnineSans-", 900.0);
    build_family_vf(&designspace, output, "WarpnineSans", 900.0)
}

pub fn build_warpnine_condensed_vf(dist_dir: &Path, output: &Path) -> Result<()> {
    let designspace = family_designspace(dist_dir, SANS_STYLES, "WarpnineSansCondensed-", 900.0);
    build_family_vf(&designspace, output, "WarpnineSansCondensed", 900.0)
}
