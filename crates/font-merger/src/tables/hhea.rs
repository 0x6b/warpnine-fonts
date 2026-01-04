//! hhea table merging

use std::result;

use read_fonts::{FontRef, TableProvider, tables::hhea::Hhea as ReadHhea};
use write_fonts::tables::hhea::Hhea;

use crate::{
    MergeError, Result,
    strategies::{first, max, min},
};

pub fn merge_hhea(fonts: &[FontRef], num_h_metrics: u16) -> Result<Hhea> {
    let tables: Vec<ReadHhea> = fonts
        .iter()
        .map(|f| f.hhea())
        .collect::<result::Result<Vec<_>, _>>()?;

    if tables.is_empty() {
        return Err(MergeError::NoFonts);
    }

    let ascenders: Vec<i16> = tables.iter().map(|t| t.ascender().to_i16()).collect();
    let descenders: Vec<i16> = tables.iter().map(|t| t.descender().to_i16()).collect();
    let line_gaps: Vec<i16> = tables.iter().map(|t| t.line_gap().to_i16()).collect();
    let advance_width_maxs: Vec<u16> =
        tables.iter().map(|t| t.advance_width_max().to_u16()).collect();
    let min_lsbs: Vec<i16> = tables.iter().map(|t| t.min_left_side_bearing().to_i16()).collect();
    let min_rsbs: Vec<i16> = tables.iter().map(|t| t.min_right_side_bearing().to_i16()).collect();
    let x_max_extents: Vec<i16> = tables.iter().map(|t| t.x_max_extent().to_i16()).collect();

    let _first_table = &tables[0];

    Ok(Hhea {
        ascender: font_types::FWord::new(max(&ascenders)?),
        descender: font_types::FWord::new(min(&descenders)?),
        line_gap: font_types::FWord::new(max(&line_gaps)?),
        advance_width_max: font_types::UfWord::new(max(&advance_width_maxs)?),
        min_left_side_bearing: font_types::FWord::new(min(&min_lsbs)?),
        min_right_side_bearing: font_types::FWord::new(min(&min_rsbs)?),
        x_max_extent: font_types::FWord::new(max(&x_max_extents)?),
        caret_slope_rise: first(&tables.iter().map(|t| t.caret_slope_rise()).collect::<Vec<_>>())?,
        caret_slope_run: first(&tables.iter().map(|t| t.caret_slope_run()).collect::<Vec<_>>())?,
        caret_offset: first(&tables.iter().map(|t| t.caret_offset()).collect::<Vec<_>>())?,
        number_of_h_metrics: num_h_metrics,
    })
}
