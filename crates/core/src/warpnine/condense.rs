//! Condensed font generation.

use std::path::Path;

use anyhow::Result;
use warpnine_font_condense::apply_horizontal_scale;
use warpnine_font_ops::apply_family_style_names;

use crate::styles::{SANS_STYLES, build_style_instances};

const WIDTH_CLASS_CONDENSED: u16 = 3;

pub fn create_condensed(input: &Path, output_dir: &Path, scale: f32) -> Result<()> {
    let count = build_style_instances(
        input,
        output_dir,
        SANS_STYLES,
        "WarpnineSansCondensed-",
        |font_data, style| {
            println!("  Applying {:.0}% horizontal scale", scale * 100.0);
            let scaled_data = apply_horizontal_scale(
                font_data,
                scale,
                Some(WIDTH_CLASS_CONDENSED),
                Some(style.weight.value() as u16),
            )?;
            apply_family_style_names(&scaled_data, "Warpnine Sans Condensed", style.name)
        },
    )?;

    println!("Created {count} condensed fonts in {}/", output_dir.display());
    Ok(())
}
