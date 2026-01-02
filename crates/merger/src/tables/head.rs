//! head table merging

use std::result;

use read_fonts::{tables::head::Head as ReadHead, FontRef, TableProvider};
use write_fonts::tables::head::Head;

use crate::{
    strategies::{equal, max, merge_bits, min},
    MergeError, Result,
};

/// Bit map for merging head.flags
/// true = OR, false = AND, None = take first
const HEAD_FLAGS_BIT_MAP: [Option<bool>; 16] = [
    Some(true), // 0: baseline at y=0
    Some(true), // 1: left sidebearing at x=0
    None,       // 2: instructions depend on point size
    Some(true), // 3: force ppem to integer
    Some(true), // 4: instructions alter advance width
    None,       // 5-10: reserved
    None,
    None,
    None,
    None,
    None,
    Some(true), // 11: lossless font data
    Some(true), // 12: font converted
    Some(true), // 13: optimized for ClearType
    Some(true), // 14: last resort font
    None,       // 15: reserved
];

/// Bit map for merging head.macStyle
const MAC_STYLE_BIT_MAP: [Option<bool>; 16] = [
    Some(false), // 0: bold - AND
    Some(false), // 1: italic - AND
    Some(true),  // 2: underline - OR
    Some(true),  // 3: outline - OR
    Some(true),  // 4: shadow - OR
    Some(false), // 5: condensed - AND
    Some(false), // 6: extended - AND
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

pub fn merge_head(fonts: &[FontRef]) -> Result<Head> {
    let tables: Vec<ReadHead> = fonts
        .iter()
        .map(|f| f.head())
        .collect::<result::Result<Vec<_>, _>>()?;

    if tables.is_empty() {
        return Err(MergeError::NoFonts);
    }

    // Collect field values
    let units_per_em: Vec<u16> = tables.iter().map(|t| t.units_per_em()).collect();
    let font_revisions: Vec<i32> = tables.iter().map(|t| t.font_revision().to_bits()).collect();
    let flags: Vec<u16> = tables.iter().map(|t| t.flags().bits()).collect();
    let mac_styles: Vec<u16> = tables.iter().map(|t| t.mac_style().bits()).collect();
    let x_mins: Vec<i16> = tables.iter().map(|t| t.x_min()).collect();
    let y_mins: Vec<i16> = tables.iter().map(|t| t.y_min()).collect();
    let x_maxs: Vec<i16> = tables.iter().map(|t| t.x_max()).collect();
    let y_maxs: Vec<i16> = tables.iter().map(|t| t.y_max()).collect();
    let lowest_rec_ppems: Vec<u16> = tables.iter().map(|t| t.lowest_rec_ppem()).collect();

    // Apply merge strategies
    let units_per_em = equal(&units_per_em, "head", "unitsPerEm")?;
    let font_revision = max(&font_revisions)?;
    let flags = merge_bits(&flags, &HEAD_FLAGS_BIT_MAP)?;
    let mac_style = merge_bits(&mac_styles, &MAC_STYLE_BIT_MAP)?;
    let x_min = min(&x_mins)?;
    let y_min = min(&y_mins)?;
    let x_max = max(&x_maxs)?;
    let y_max = max(&y_maxs)?;
    let lowest_rec_ppem = max(&lowest_rec_ppems)?;

    // Take other values from first font
    let first = &tables[0];

    Ok(Head {
        font_revision: font_types::Fixed::from_bits(font_revision),
        checksum_adjustment: 0, // Will be recomputed on write
        magic_number: 0x5F0F3CF5,
        flags: write_fonts::tables::head::Flags::from_bits_truncate(flags),
        units_per_em,
        created: first.created(),
        modified: first.modified(),
        x_min,
        y_min,
        x_max,
        y_max,
        mac_style: write_fonts::tables::head::MacStyle::from_bits_truncate(mac_style),
        lowest_rec_ppem,
        font_direction_hint: first.font_direction_hint(),
        index_to_loc_format: first.index_to_loc_format(),
    })
}
