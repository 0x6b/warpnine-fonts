//! post table merging

use std::result;

use read_fonts::{TableProvider, tables::post::Post as ReadPost};
use write_fonts::tables::post::Post;

use crate::{MergeError, Result, context::MergeContext, strategies::first};

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
    let underline_positions: Vec<i16> =
        tables.iter().map(|t| t.underline_position().to_i16()).collect();
    let underline_thicknesses: Vec<i16> =
        tables.iter().map(|t| t.underline_thickness().to_i16()).collect();
    let is_fixed_pitches: Vec<u32> = tables.iter().map(|t| t.is_fixed_pitch()).collect();

    // Build a version 2.0 post table that preserves glyph names from the merged glyph order.
    // This is important for GSUB substitution mappings to work correctly after merging,
    // as tools like feature freezers need to look up glyphs by name.
    let glyph_names: Vec<&str> = ctx.mega().iter().map(|n| n.as_str()).collect();
    let mut post = Post::new_v2(glyph_names);

    // Copy over metric values from the first font
    post.italic_angle = font_types::Fixed::from_bits(first(&italic_angles)?);
    post.underline_position = font_types::FWord::new(first(&underline_positions)?);
    post.underline_thickness = font_types::FWord::new(first(&underline_thicknesses)?);
    post.is_fixed_pitch = first(&is_fixed_pitches)?;
    post.min_mem_type42 = first(&tables.iter().map(|t| t.min_mem_type42()).collect::<Vec<_>>())?;
    post.max_mem_type42 = first(&tables.iter().map(|t| t.max_mem_type42()).collect::<Vec<_>>())?;
    post.min_mem_type1 = first(&tables.iter().map(|t| t.min_mem_type1()).collect::<Vec<_>>())?;
    post.max_mem_type1 = first(&tables.iter().map(|t| t.max_mem_type1()).collect::<Vec<_>>())?;

    Ok(post)
}
