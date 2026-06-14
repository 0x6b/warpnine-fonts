//! hhea table merging

use std::result;

use font_types::{FWord, UfWord};
use read_fonts::{
    FontRef, TableProvider,
    tables::{hhea, hhea::Hhea as ReadHhea},
};
use write_fonts::tables::hhea::Hhea;

use crate::{
    MergeError, Result,
    strategies::{first, max, min},
};

pub fn merge_hhea(fonts: &[FontRef], num_h_metrics: u16) -> Result<Hhea> {
    let tables: Vec<ReadHhea> = fonts
        .iter()
        .map(TableProvider::hhea)
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

    Ok(Hhea {
        ascender: FWord::new(max(&ascenders)?),
        descender: FWord::new(min(&descenders)?),
        line_gap: FWord::new(max(&line_gaps)?),
        advance_width_max: UfWord::new(max(&advance_width_maxs)?),
        min_left_side_bearing: FWord::new(min(&min_lsbs)?),
        min_right_side_bearing: FWord::new(min(&min_rsbs)?),
        x_max_extent: FWord::new(max(&x_max_extents)?),
        caret_slope_rise: first(
            &tables.iter().map(hhea::Hhea::caret_slope_rise).collect::<Vec<_>>(),
        )?,
        caret_slope_run: first(
            &tables.iter().map(hhea::Hhea::caret_slope_run).collect::<Vec<_>>(),
        )?,
        caret_offset: first(&tables.iter().map(hhea::Hhea::caret_offset).collect::<Vec<_>>())?,
        number_of_h_metrics: num_h_metrics,
    })
}
