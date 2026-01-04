//! Variable font building.

use std::{fs::write, path::Path};

use anyhow::{Context, Result, bail};
use warpnine_font_vf_builder::{Axis, DesignSpace, Instance, Source, build_variable_font};

use crate::styles::MONO_STYLES;

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
                vec![("wght", style.weight.value()), ("ital", style.slant.ital())],
            )
            .with_style_name(&style.display_name())
        })
        .collect();

    let instances: Vec<Instance> = MONO_STYLES
        .iter()
        .map(|style| {
            Instance::new(
                &style.display_name(),
                vec![("wght", style.weight.value()), ("ital", style.slant.ital())],
            )
        })
        .collect();

    DesignSpace::new(axes, sources).with_instances(instances)
}

pub fn build_warpnine_mono_vf(dist_dir: &Path, output: &Path) -> Result<()> {
    println!("Building WarpnineMono variable font...");

    let designspace = warpnine_mono_designspace(dist_dir);

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
