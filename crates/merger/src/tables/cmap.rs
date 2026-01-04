//! cmap table merging

use std::collections::HashMap;

use indexmap::{IndexMap, map::Entry};
use read_fonts::{
    FontRef, TableProvider,
    tables::cmap::{Cmap as ReadCmap, CmapSubtable, PlatformId},
    types,
};
use write_fonts::tables::cmap::Cmap;

use crate::{
    MergeError, Result,
    context::GlyphOrder,
    glyph_order::GlyphName,
    types::{Codepoint, GlyphId},
};

/// Information about duplicate glyphs (same codepoint, different glyphs)
#[derive(Debug, Default)]
pub struct DuplicateGlyphInfo {
    /// Per-font mapping of original glyph name to disambiguated name
    pub per_font: Vec<HashMap<GlyphName, GlyphName>>,
}

/// Merge cmap tables from multiple fonts
///
/// Returns the merged cmap and information about duplicate glyphs
pub fn merge_cmap(
    fonts: &[FontRef],
    glyph_order: &GlyphOrder,
) -> Result<(Cmap, DuplicateGlyphInfo)> {
    let mut codepoint_to_glyph: IndexMap<Codepoint, GlyphName> = IndexMap::new();
    let mut duplicate_info = DuplicateGlyphInfo { per_font: vec![HashMap::new(); fonts.len()] };

    for (font_idx, font) in fonts.iter().enumerate() {
        let cmap = font.cmap()?;
        let mapping = glyph_order.font_mapping(font_idx);

        // Find best cmap subtable (prefer Unicode BMP, then full Unicode)
        if let Some(subtable) = find_best_subtable(&cmap) {
            for (codepoint, glyph_id) in iter_cmap_subtable(&subtable) {
                if let Some(name) = mapping.get(&glyph_id) {
                    match codepoint_to_glyph.entry(codepoint) {
                        Entry::Vacant(slot) => {
                            slot.insert(name.clone());
                        }
                        Entry::Occupied(slot) => {
                            let existing = slot.get();
                            if existing != name {
                                duplicate_info.per_font[font_idx]
                                    .insert(name.clone(), existing.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Build the cmap
    let mut mappings: Vec<(char, types::GlyphId)> = codepoint_to_glyph
        .iter()
        .filter_map(|(cp, name)| {
            let mega_gid = glyph_order.mega_id(name)?;
            let ch = cp.to_char()?;
            Some((ch, read_fonts::types::GlyphId::new(mega_gid.to_u32())))
        })
        .collect();

    mappings.sort_by_key(|(ch, _)| *ch);

    let cmap = Cmap::from_mappings(mappings).map_err(|_| MergeError::CmapBuildError)?;

    Ok((cmap, duplicate_info))
}

fn find_best_subtable<'a>(cmap: &'a ReadCmap<'a>) -> Option<CmapSubtable<'a>> {
    // Priority: Format 12 (full Unicode) > Format 4 (BMP) > others
    let records = cmap.encoding_records();

    // Try to find format 12 first (Unicode full)
    for record in records {
        if record.platform_id() == PlatformId::Unicode
            || (record.platform_id() == PlatformId::Windows && record.encoding_id() == 10)
        {
            if let Ok(subtable) = record.subtable(cmap.offset_data()) {
                if matches!(subtable, CmapSubtable::Format12(_)) {
                    return Some(subtable);
                }
            }
        }
    }

    // Fall back to format 4 (BMP)
    for record in records {
        if record.platform_id() == PlatformId::Unicode
            || (record.platform_id() == PlatformId::Windows && record.encoding_id() == 1)
        {
            if let Ok(subtable) = record.subtable(cmap.offset_data()) {
                if matches!(subtable, CmapSubtable::Format4(_)) {
                    return Some(subtable);
                }
            }
        }
    }

    // Take any subtable
    records.iter().find_map(|r| r.subtable(cmap.offset_data()).ok())
}

fn iter_cmap_subtable(subtable: &CmapSubtable) -> Vec<(Codepoint, GlyphId)> {
    let mut mappings = Vec::new();

    match subtable {
        CmapSubtable::Format4(f4) => {
            // Iterate through segments
            let end_codes = f4.end_code();
            let start_codes = f4.start_code();
            let id_deltas = f4.id_delta();
            let id_range_offsets = f4.id_range_offsets();
            let glyph_id_array = f4.glyph_id_array();

            let seg_count = f4.seg_count_x2() as usize / 2;
            for seg in 0..seg_count {
                let end_code = end_codes.get(seg).map(|v| v.get()).unwrap_or(0xFFFF);
                let start_code = start_codes.get(seg).map(|v| v.get()).unwrap_or(0);
                let id_delta = id_deltas.get(seg).map(|v| v.get()).unwrap_or(0);
                let id_range_offset = id_range_offsets.get(seg).map(|v| v.get()).unwrap_or(0);

                if start_code == 0xFFFF {
                    continue;
                }

                for cp in start_code..=end_code {
                    let gid = if id_range_offset == 0 {
                        ((cp as i32 + id_delta as i32) & 0xFFFF) as u16
                    } else {
                        let glyph_idx = (id_range_offset as usize / 2) + (cp - start_code) as usize
                            - (seg_count - seg);
                        if let Some(gid) = glyph_id_array.get(glyph_idx) {
                            let gid = gid.get();
                            if gid != 0 {
                                ((gid as i32 + id_delta as i32) & 0xFFFF) as u16
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    };

                    if gid != 0 {
                        mappings.push((Codepoint::new(cp as u32), GlyphId::new(gid)));
                    }
                }
            }
        }
        CmapSubtable::Format12(f12) => {
            for group in f12.groups() {
                let start = group.start_char_code();
                let end = group.end_char_code();
                let mut gid = group.start_glyph_id();
                for cp in start..=end {
                    if gid != 0 {
                        mappings.push((Codepoint::new(cp), GlyphId::new(gid as u16)));
                    }
                    gid += 1;
                }
            }
        }
        CmapSubtable::Format6(f6) => {
            let first = f6.first_code() as u32;
            for (i, gid) in f6.glyph_id_array().iter().enumerate() {
                let gid = gid.get();
                if gid != 0 {
                    mappings.push((Codepoint::new(first + i as u32), GlyphId::new(gid)));
                }
            }
        }
        _ => {}
    }

    mappings
}
