//! Build variable font from static masters.

use anyhow::{Context, Result};
use std::path::Path;
use warpnine_font_vf_builder::{Axis, DesignSpace, Instance, Source, build_variable_font};

/// WarpnineMono designspace configuration.
///
/// This matches the Python configuration in warpnine_fonts/config/instances.py
/// and warpnine_fonts/operations/variable.py.
pub fn warpnine_mono_designspace(dist_dir: &Path) -> DesignSpace {
    let axes = vec![
        Axis::new("wght", "Weight", 300.0, 400.0, 1000.0),
        Axis::new("ital", "Italic", 0.0, 0.0, 1.0),
    ];

    // 16 static masters: 8 weights × 2 styles
    let sources = vec![
        // Light
        Source::new(
            dist_dir.join("WarpnineMono-Light.ttf"),
            vec![("wght", 300.0), ("ital", 0.0)],
        )
        .with_style_name("Light"),
        Source::new(
            dist_dir.join("WarpnineMono-LightItalic.ttf"),
            vec![("wght", 300.0), ("ital", 1.0)],
        )
        .with_style_name("Light Italic"),
        // Regular (default)
        Source::new(
            dist_dir.join("WarpnineMono-Regular.ttf"),
            vec![("wght", 400.0), ("ital", 0.0)],
        )
        .with_style_name("Regular"),
        Source::new(
            dist_dir.join("WarpnineMono-Italic.ttf"),
            vec![("wght", 400.0), ("ital", 1.0)],
        )
        .with_style_name("Italic"),
        // Medium
        Source::new(
            dist_dir.join("WarpnineMono-Medium.ttf"),
            vec![("wght", 500.0), ("ital", 0.0)],
        )
        .with_style_name("Medium"),
        Source::new(
            dist_dir.join("WarpnineMono-MediumItalic.ttf"),
            vec![("wght", 500.0), ("ital", 1.0)],
        )
        .with_style_name("Medium Italic"),
        // SemiBold
        Source::new(
            dist_dir.join("WarpnineMono-SemiBold.ttf"),
            vec![("wght", 600.0), ("ital", 0.0)],
        )
        .with_style_name("SemiBold"),
        Source::new(
            dist_dir.join("WarpnineMono-SemiBoldItalic.ttf"),
            vec![("wght", 600.0), ("ital", 1.0)],
        )
        .with_style_name("SemiBold Italic"),
        // Bold
        Source::new(
            dist_dir.join("WarpnineMono-Bold.ttf"),
            vec![("wght", 700.0), ("ital", 0.0)],
        )
        .with_style_name("Bold"),
        Source::new(
            dist_dir.join("WarpnineMono-BoldItalic.ttf"),
            vec![("wght", 700.0), ("ital", 1.0)],
        )
        .with_style_name("Bold Italic"),
        // ExtraBold
        Source::new(
            dist_dir.join("WarpnineMono-ExtraBold.ttf"),
            vec![("wght", 800.0), ("ital", 0.0)],
        )
        .with_style_name("ExtraBold"),
        Source::new(
            dist_dir.join("WarpnineMono-ExtraBoldItalic.ttf"),
            vec![("wght", 800.0), ("ital", 1.0)],
        )
        .with_style_name("ExtraBold Italic"),
        // Black
        Source::new(
            dist_dir.join("WarpnineMono-Black.ttf"),
            vec![("wght", 900.0), ("ital", 0.0)],
        )
        .with_style_name("Black"),
        Source::new(
            dist_dir.join("WarpnineMono-BlackItalic.ttf"),
            vec![("wght", 900.0), ("ital", 1.0)],
        )
        .with_style_name("Black Italic"),
        // ExtraBlack
        Source::new(
            dist_dir.join("WarpnineMono-ExtraBlack.ttf"),
            vec![("wght", 1000.0), ("ital", 0.0)],
        )
        .with_style_name("ExtraBlack"),
        Source::new(
            dist_dir.join("WarpnineMono-ExtraBlackItalic.ttf"),
            vec![("wght", 1000.0), ("ital", 1.0)],
        )
        .with_style_name("ExtraBlack Italic"),
    ];

    // Named instances matching the static fonts
    let instances = vec![
        Instance::new("Light", vec![("wght", 300.0), ("ital", 0.0)]),
        Instance::new("Light Italic", vec![("wght", 300.0), ("ital", 1.0)]),
        Instance::new("Regular", vec![("wght", 400.0), ("ital", 0.0)]),
        Instance::new("Italic", vec![("wght", 400.0), ("ital", 1.0)]),
        Instance::new("Medium", vec![("wght", 500.0), ("ital", 0.0)]),
        Instance::new("Medium Italic", vec![("wght", 500.0), ("ital", 1.0)]),
        Instance::new("SemiBold", vec![("wght", 600.0), ("ital", 0.0)]),
        Instance::new("SemiBold Italic", vec![("wght", 600.0), ("ital", 1.0)]),
        Instance::new("Bold", vec![("wght", 700.0), ("ital", 0.0)]),
        Instance::new("Bold Italic", vec![("wght", 700.0), ("ital", 1.0)]),
        Instance::new("ExtraBold", vec![("wght", 800.0), ("ital", 0.0)]),
        Instance::new("ExtraBold Italic", vec![("wght", 800.0), ("ital", 1.0)]),
        Instance::new("Black", vec![("wght", 900.0), ("ital", 0.0)]),
        Instance::new("Black Italic", vec![("wght", 900.0), ("ital", 1.0)]),
        Instance::new("ExtraBlack", vec![("wght", 1000.0), ("ital", 0.0)]),
        Instance::new("ExtraBlack Italic", vec![("wght", 1000.0), ("ital", 1.0)]),
    ];

    DesignSpace::new(axes, sources).with_instances(instances)
}

/// Build WarpnineMono variable font.
pub fn build_warpnine_mono_vf(dist_dir: &Path, output: &Path) -> Result<()> {
    println!("Building WarpnineMono variable font...");

    let designspace = warpnine_mono_designspace(dist_dir);

    // Verify all source fonts exist
    for source in &designspace.sources {
        if !source.path.exists() {
            anyhow::bail!("Source font not found: {}", source.path.display());
        }
    }

    println!("  Sources: {} masters", designspace.sources.len());
    println!("  Axes: wght (300-1000), ital (0-1)");

    let vf_data =
        build_variable_font(&designspace).with_context(|| "Failed to build variable font")?;

    std::fs::write(output, &vf_data)
        .with_context(|| format!("Failed to write {}", output.display()))?;

    let size_mb = vf_data.len() as f64 / 1024.0 / 1024.0;
    println!("  Output: {} ({:.2} MB)", output.display(), size_mb);

    Ok(())
}
