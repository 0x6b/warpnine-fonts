//! post table merging

use crate::context::MergeContext;

use crate::strategies::first;
use crate::{MergeError, Result};
use font_types::Version16Dot16;
use read_fonts::{tables::post::Post as ReadPost, TableProvider};
use std::result;
use write_fonts::tables::post::Post;

/// Version 3.0 - no glyph names stored
const POST_VERSION_3: Version16Dot16 = Version16Dot16::new(3, 0);

pub fn merge_post(ctx: &MergeContext) -> Result<Post> {
    let tables: Vec<ReadPost> = ctx
        .fonts()
        .iter()
        .map(|f| f.post())
        .collect::<result::Result<Vec<_>, _>>()?;

    if tables.is_empty() {
        return Err(MergeError::NoFonts);
    }

    let italic_angles: Vec<i32> = tables.iter().map(|t| t.italic_angle().to_bits()).collect();
    let underline_positions: Vec<i16> = tables
        .iter()
        .map(|t| t.underline_position().to_i16())
        .collect();
    let underline_thicknesses: Vec<i16> = tables
        .iter()
        .map(|t| t.underline_thickness().to_i16())
        .collect();
    let is_fixed_pitches: Vec<u32> = tables.iter().map(|t| t.is_fixed_pitch()).collect();

    let num_glyphs = ctx.mega().len() as u16;

    // Use version 3.0 which doesn't require glyph names.
    // Version 2.0 would require building glyphNameIndex for all merged glyphs,
    // which is complex when fonts have different post versions.
    Ok(Post {
        version: POST_VERSION_3,
        num_glyphs: Some(num_glyphs),
        glyph_name_index: None,
        string_data: None,
        italic_angle: font_types::Fixed::from_bits(first(&italic_angles)?),
        underline_position: font_types::FWord::new(first(&underline_positions)?),
        underline_thickness: font_types::FWord::new(first(&underline_thicknesses)?),
        is_fixed_pitch: first(&is_fixed_pitches)?,
        min_mem_type42: first(
            &tables
                .iter()
                .map(|t| t.min_mem_type42())
                .collect::<Vec<_>>(),
        )?,
        max_mem_type42: first(
            &tables
                .iter()
                .map(|t| t.max_mem_type42())
                .collect::<Vec<_>>(),
        )?,
        min_mem_type1: first(&tables.iter().map(|t| t.min_mem_type1()).collect::<Vec<_>>())?,
        max_mem_type1: first(&tables.iter().map(|t| t.max_mem_type1()).collect::<Vec<_>>())?,
    })
}
