use std::path::Path;

use anyhow::Result;

use crate::styles::{SANS_STYLES, Style, build_style_instances};

// Instancing already sets usWeightClass from the wght axis; naming and style
// bits are applied later by `set_ribbi_names_for_pattern`.
fn transform_sans(font_data: &[u8], _style: &Style) -> Result<Vec<u8>> {
    Ok(font_data.to_vec())
}

pub fn create_sans(input: &Path, output_dir: &Path) -> Result<()> {
    let count =
        build_style_instances(input, output_dir, SANS_STYLES, "WarpnineSans-", transform_sans)?;
    println!("Created {count} sans fonts in {}/", output_dir.display());
    Ok(())
}
