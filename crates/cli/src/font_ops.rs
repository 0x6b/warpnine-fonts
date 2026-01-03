//! Shared font manipulation helpers.

use std::path::Path;

use anyhow::Result;
use read_fonts::types::Tag;

use crate::io::{read_font, write_font};

pub use warpnine_font_ops::{apply_family_style_names, map_name_records, rewrite_font};

/// Modify a font file in place.
pub fn modify_font_in_place(
    path: &Path,
    f: impl FnOnce(&read_fonts::FontRef, &mut write_fonts::FontBuilder) -> Result<()>,
) -> Result<()> {
    let data = read_font(path)?;
    let new_data = rewrite_font(&data, f)?;
    write_font(path, new_data)?;
    Ok(())
}

/// Copy GSUB table from source font to target font.
pub fn copy_gsub(source: &Path, target: &Path) -> Result<()> {
    let source_data = read_font(source)?;
    let target_data = read_font(target)?;

    let new_data = warpnine_font_ops::copy_table(&source_data, &target_data, Tag::new(b"GSUB"))?;
    write_font(target, new_data)?;

    println!("Copied GSUB table from {} to {}", source.display(), target.display());
    Ok(())
}
