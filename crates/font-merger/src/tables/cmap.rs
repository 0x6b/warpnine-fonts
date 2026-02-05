//! cmap table merging

use std::collections::HashMap;

use indexmap::{IndexMap, map::Entry};
use read_fonts::{
    FontRef, TableProvider,
    tables::cmap::{Cmap as ReadCmap, CmapSubtable, PlatformId},
};
use write_fonts::tables::cmap::{
    Cmap, Cmap12, CmapSubtable as WriteCmapSubtable, EncodingRecord, PlatformId as WritePlatformId,
    SequentialMapGroup,
};

use crate::{
    Result,
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

    // Build the cmap using format 12 only to avoid format 4 overflow with large character sets
    let mut mappings: Vec<(u32, u32)> = codepoint_to_glyph
        .iter()
        .filter_map(|(cp, name)| {
            let mega_gid = glyph_order.mega_id(name)?;
            Some((cp.to_u32(), mega_gid.to_u32()))
        })
        .collect();

    mappings.sort_by_key(|(cp, _)| *cp);

    let cmap = build_cmap_format12(&mappings);

    Ok((cmap, duplicate_info))
}

/// Build a cmap table using only format 12 subtables.
///
/// This avoids the format 4 overflow issue that occurs with large character sets
/// (format 4 uses u16 for segment counts and can overflow with >32k entries).
fn build_cmap_format12(mappings: &[(u32, u32)]) -> Cmap {
    // Build sequential map groups by finding contiguous runs
    let groups = build_sequential_groups(mappings);

    let cmap12 = Cmap12 { language: 0, groups };

    // Create encoding records for Unicode platform (required for cross-platform support)
    // Platform 0 (Unicode), Encoding 4 (Unicode full repertoire)
    // Platform 3 (Windows), Encoding 10 (Unicode full repertoire)
    let encoding_records = vec![
        EncodingRecord::new(
            WritePlatformId::Unicode,
            4, // Full Unicode
            WriteCmapSubtable::Format12(cmap12.clone()),
        ),
        EncodingRecord::new(
            WritePlatformId::Windows,
            10, // Full Unicode
            WriteCmapSubtable::Format12(cmap12),
        ),
    ];

    Cmap::new(encoding_records)
}

/// Build sequential map groups from sorted (codepoint, glyph_id) pairs.
///
/// Groups consecutive codepoints that map to consecutive glyph IDs.
fn build_sequential_groups(mappings: &[(u32, u32)]) -> Vec<SequentialMapGroup> {
    if mappings.is_empty() {
        return Vec::new();
    }

    let mut groups = Vec::new();
    let mut group_start_cp = mappings[0].0;
    let mut group_start_gid = mappings[0].1;
    let mut prev_cp = group_start_cp;
    let mut prev_gid = group_start_gid;

    for &(cp, gid) in &mappings[1..] {
        // Check if this continues the current group (consecutive codepoint AND glyph ID)
        if cp == prev_cp + 1 && gid == prev_gid + 1 {
            prev_cp = cp;
            prev_gid = gid;
        } else {
            // End the current group and start a new one
            groups.push(SequentialMapGroup::new(group_start_cp, prev_cp, group_start_gid));
            group_start_cp = cp;
            group_start_gid = gid;
            prev_cp = cp;
            prev_gid = gid;
        }
    }

    // Don't forget the last group
    groups.push(SequentialMapGroup::new(group_start_cp, prev_cp, group_start_gid));

    groups
}

fn find_best_subtable<'a>(cmap: &'a ReadCmap<'a>) -> Option<CmapSubtable<'a>> {
    // Priority: Format 12 (full Unicode) > Format 4 (BMP) > others
    let records = cmap.encoding_records();

    // Try to find format 12 first (Unicode full)
    for record in records {
        if (record.platform_id() == PlatformId::Unicode
            || (record.platform_id() == PlatformId::Windows && record.encoding_id() == 10))
            && let Ok(subtable) = record.subtable(cmap.offset_data())
            && matches!(subtable, CmapSubtable::Format12(_))
        {
            return Some(subtable);
        }
    }

    // Fall back to format 4 (BMP)
    for record in records {
        if (record.platform_id() == PlatformId::Unicode
            || (record.platform_id() == PlatformId::Windows && record.encoding_id() == 1))
            && let Ok(subtable) = record.subtable(cmap.offset_data())
            && matches!(subtable, CmapSubtable::Format4(_))
        {
            return Some(subtable);
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
