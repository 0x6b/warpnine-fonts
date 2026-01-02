//! vhea table merging

use crate::strategies::{first, max, min};
use crate::Result;
use read_fonts::{tables::vhea::Vhea as ReadVhea, FontRef, TableProvider};
use write_fonts::tables::vhea::Vhea;

pub fn merge_vhea(fonts: &[FontRef], num_v_metrics: u16) -> Result<Option<Vhea>> {
    let tables: Vec<ReadVhea> = fonts.iter().filter_map(|f| f.vhea().ok()).collect();

    if tables.is_empty() {
        return Ok(None);
    }

    // Only merge if all fonts have vhea
    if tables.len() != fonts.len() {
        return Ok(None);
    }

    let ascenders: Vec<i16> = tables.iter().map(|t| t.ascender().to_i16()).collect();
    let descenders: Vec<i16> = tables.iter().map(|t| t.descender().to_i16()).collect();
    let line_gaps: Vec<i16> = tables.iter().map(|t| t.line_gap().to_i16()).collect();
    let advance_height_maxs: Vec<u16> = tables
        .iter()
        .map(|t| t.advance_height_max().to_u16())
        .collect();
    let min_tsbs: Vec<i16> = tables
        .iter()
        .map(|t| t.min_top_side_bearing().to_i16())
        .collect();
    let min_bsbs: Vec<i16> = tables
        .iter()
        .map(|t| t.min_bottom_side_bearing().to_i16())
        .collect();
    let y_max_extents: Vec<i16> = tables.iter().map(|t| t.y_max_extent().to_i16()).collect();

    let _first_table = &tables[0];

    Ok(Some(Vhea {
        ascender: font_types::FWord::new(max(&ascenders)?),
        descender: font_types::FWord::new(min(&descenders)?),
        line_gap: font_types::FWord::new(max(&line_gaps)?),
        advance_height_max: font_types::UfWord::new(max(&advance_height_maxs)?),
        min_top_side_bearing: font_types::FWord::new(min(&min_tsbs)?),
        min_bottom_side_bearing: font_types::FWord::new(min(&min_bsbs)?),
        y_max_extent: font_types::FWord::new(max(&y_max_extents)?),
        caret_slope_rise: first(
            &tables
                .iter()
                .map(|t| t.caret_slope_rise())
                .collect::<Vec<_>>(),
        )?,
        caret_slope_run: first(
            &tables
                .iter()
                .map(|t| t.caret_slope_run())
                .collect::<Vec<_>>(),
        )?,
        caret_offset: first(&tables.iter().map(|t| t.caret_offset()).collect::<Vec<_>>())?,
        number_of_long_ver_metrics: num_v_metrics,
    }))
}
