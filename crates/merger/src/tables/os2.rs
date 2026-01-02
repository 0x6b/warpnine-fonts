//! OS/2 table merging

use crate::strategies::{first, max, min};
use crate::Result;
use read_fonts::{tables::os2::Os2 as ReadOs2, FontRef, TableProvider};
use write_fonts::tables::os2::Os2;
use write_fonts::tables::os2::SelectionFlags;

pub fn merge_os2(fonts: &[FontRef]) -> Result<Option<Os2>> {
    let tables: Vec<ReadOs2> = fonts.iter().filter_map(|f| f.os2().ok()).collect();

    if tables.is_empty() {
        return Ok(None);
    }

    // Only merge if all fonts have OS/2
    if tables.len() != fonts.len() {
        return Ok(None);
    }

    let first_table = &tables[0];

    // Compute max version - merged table uses the highest version
    let max_version = tables.iter().map(|t| t.version()).max().unwrap_or(0);

    // Collect values for merging
    let x_avg_char_widths: Vec<i16> = tables.iter().map(|t| t.x_avg_char_width()).collect();
    let us_weight_classes: Vec<u16> = tables.iter().map(|t| t.us_weight_class()).collect();
    let us_width_classes: Vec<u16> = tables.iter().map(|t| t.us_width_class()).collect();
    let fs_types: Vec<u16> = tables.iter().map(|t| t.fs_type()).collect();
    let y_subscript_x_sizes: Vec<i16> = tables.iter().map(|t| t.y_subscript_x_size()).collect();
    let y_subscript_y_sizes: Vec<i16> = tables.iter().map(|t| t.y_subscript_y_size()).collect();
    let y_subscript_x_offsets: Vec<i16> = tables.iter().map(|t| t.y_subscript_x_offset()).collect();
    let y_subscript_y_offsets: Vec<i16> = tables.iter().map(|t| t.y_subscript_y_offset()).collect();
    let y_superscript_x_sizes: Vec<i16> = tables.iter().map(|t| t.y_superscript_x_size()).collect();
    let y_superscript_y_sizes: Vec<i16> = tables.iter().map(|t| t.y_superscript_y_size()).collect();
    let y_superscript_x_offsets: Vec<i16> =
        tables.iter().map(|t| t.y_superscript_x_offset()).collect();
    let y_superscript_y_offsets: Vec<i16> =
        tables.iter().map(|t| t.y_superscript_y_offset()).collect();
    let y_strikeout_sizes: Vec<i16> = tables.iter().map(|t| t.y_strikeout_size()).collect();
    let y_strikeout_positions: Vec<i16> = tables.iter().map(|t| t.y_strikeout_position()).collect();
    let s_typo_ascenders: Vec<i16> = tables.iter().map(|t| t.s_typo_ascender()).collect();
    let s_typo_descenders: Vec<i16> = tables.iter().map(|t| t.s_typo_descender()).collect();
    let s_typo_line_gaps: Vec<i16> = tables.iter().map(|t| t.s_typo_line_gap()).collect();
    let us_win_ascents: Vec<u16> = tables.iter().map(|t| t.us_win_ascent()).collect();
    let us_win_descents: Vec<u16> = tables.iter().map(|t| t.us_win_descent()).collect();
    let sx_heights: Vec<i16> = tables.iter().filter_map(|t| t.sx_height()).collect();
    let s_cap_heights: Vec<i16> = tables.iter().filter_map(|t| t.s_cap_height()).collect();

    // Merge Unicode ranges (OR)
    let ul_unicode_range1: u32 = tables
        .iter()
        .map(|t| t.ul_unicode_range_1())
        .fold(0, |a, b| a | b);
    let ul_unicode_range2: u32 = tables
        .iter()
        .map(|t| t.ul_unicode_range_2())
        .fold(0, |a, b| a | b);
    let ul_unicode_range3: u32 = tables
        .iter()
        .map(|t| t.ul_unicode_range_3())
        .fold(0, |a, b| a | b);
    let ul_unicode_range4: u32 = tables
        .iter()
        .map(|t| t.ul_unicode_range_4())
        .fold(0, |a, b| a | b);

    // Merge code page ranges (OR)
    let ul_code_page_range1: Option<u32> = tables
        .iter()
        .filter_map(|t| t.ul_code_page_range_1())
        .reduce(|a, b| a | b);
    let ul_code_page_range2: Option<u32> = tables
        .iter()
        .filter_map(|t| t.ul_code_page_range_2())
        .reduce(|a, b| a | b);

    // Merge fs_selection (AND for most bits, OR for some)
    let fs_selection = merge_fs_selection(&tables);

    // Compute char range from all fonts
    let us_first_char_index = tables
        .iter()
        .map(|t| t.us_first_char_index())
        .min()
        .unwrap_or(0);
    let us_last_char_index = tables
        .iter()
        .map(|t| t.us_last_char_index())
        .max()
        .unwrap_or(0);

    // Get panose as array
    let panose: [u8; 10] = first_table.panose_10().try_into().unwrap_or([0; 10]);

    Ok(Some(Os2 {
        x_avg_char_width: first(&x_avg_char_widths)?,
        us_weight_class: first(&us_weight_classes)?,
        us_width_class: first(&us_width_classes)?,
        fs_type: first(&fs_types)?,
        y_subscript_x_size: first(&y_subscript_x_sizes)?,
        y_subscript_y_size: first(&y_subscript_y_sizes)?,
        y_subscript_x_offset: first(&y_subscript_x_offsets)?,
        y_subscript_y_offset: first(&y_subscript_y_offsets)?,
        y_superscript_x_size: first(&y_superscript_x_sizes)?,
        y_superscript_y_size: first(&y_superscript_y_sizes)?,
        y_superscript_x_offset: first(&y_superscript_x_offsets)?,
        y_superscript_y_offset: first(&y_superscript_y_offsets)?,
        y_strikeout_size: first(&y_strikeout_sizes)?,
        y_strikeout_position: first(&y_strikeout_positions)?,
        s_family_class: first_table.s_family_class(),
        panose_10: panose,
        ul_unicode_range_1: ul_unicode_range1,
        ul_unicode_range_2: ul_unicode_range2,
        ul_unicode_range_3: ul_unicode_range3,
        ul_unicode_range_4: ul_unicode_range4,
        ach_vend_id: first_table.ach_vend_id(),
        fs_selection,
        us_first_char_index,
        us_last_char_index,
        s_typo_ascender: max(&s_typo_ascenders)?,
        s_typo_descender: min(&s_typo_descenders)?,
        s_typo_line_gap: max(&s_typo_line_gaps)?,
        us_win_ascent: max(&us_win_ascents)?,
        us_win_descent: max(&us_win_descents)?,
        // Version 1+ fields
        ul_code_page_range_1: if max_version >= 1 {
            ul_code_page_range1.or(Some(0))
        } else {
            None
        },
        ul_code_page_range_2: if max_version >= 1 {
            ul_code_page_range2.or(Some(0))
        } else {
            None
        },
        // Version 2+ fields
        sx_height: if max_version >= 2 {
            Some(if sx_heights.is_empty() {
                0
            } else {
                max(&sx_heights)?
            })
        } else {
            None
        },
        s_cap_height: if max_version >= 2 {
            Some(if s_cap_heights.is_empty() {
                0
            } else {
                max(&s_cap_heights)?
            })
        } else {
            None
        },
        us_default_char: if max_version >= 2 {
            Some(first_table.us_default_char().unwrap_or(0))
        } else {
            None
        },
        us_break_char: if max_version >= 2 {
            Some(first_table.us_break_char().unwrap_or(0x20))
        } else {
            None
        },
        us_max_context: if max_version >= 2 {
            Some(first_table.us_max_context().unwrap_or(0))
        } else {
            None
        },
        // Version 5+ fields
        us_lower_optical_point_size: if max_version >= 5 {
            first_table.us_lower_optical_point_size().or(Some(0))
        } else {
            None
        },
        us_upper_optical_point_size: if max_version >= 5 {
            first_table.us_upper_optical_point_size().or(Some(0xFFFF))
        } else {
            None
        },
    }))
}

fn merge_fs_selection(tables: &[ReadOs2]) -> SelectionFlags {
    let values: Vec<u16> = tables.iter().map(|t| t.fs_selection().bits()).collect();

    // Bit meanings:
    // 0: ITALIC - AND
    // 1: UNDERSCORE - OR
    // 2: NEGATIVE - OR
    // 3: OUTLINED - OR
    // 4: STRIKEOUT - OR
    // 5: BOLD - AND
    // 6: REGULAR - AND
    // 7: USE_TYPO_METRICS - AND
    // 8: WWS - AND
    // 9: OBLIQUE - AND

    let mut result = 0u16;

    // AND bits (must be set in all fonts)
    for bit in [0, 5, 6, 7, 8, 9] {
        let mask = 1u16 << bit;
        if values.iter().all(|v| v & mask != 0) {
            result |= mask;
        }
    }

    // OR bits (set if any font has it)
    for bit in [1, 2, 3, 4] {
        let mask = 1u16 << bit;
        if values.iter().any(|v| v & mask != 0) {
            result |= mask;
        }
    }

    write_fonts::tables::os2::SelectionFlags::from_bits_truncate(result)
}
