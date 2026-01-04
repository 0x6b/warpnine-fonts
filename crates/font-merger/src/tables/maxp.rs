//! maxp table merging

use std::result;

use font_types::Version16Dot16;
use read_fonts::{FontRef, TableProvider, tables::maxp::Maxp as ReadMaxp};
use write_fonts::tables::maxp::Maxp;

use crate::{
    MergeError, Result,
    strategies::{first, max},
};

pub fn merge_maxp(fonts: &[FontRef], total_glyphs: u16) -> Result<Maxp> {
    let tables: Vec<ReadMaxp> = fonts
        .iter()
        .map(|f| f.maxp())
        .collect::<result::Result<Vec<_>, _>>()?;

    if tables.is_empty() {
        return Err(MergeError::NoFonts);
    }

    // Determine version from first font
    let is_version_1 = tables[0].version() == Version16Dot16::VERSION_1_0;

    if is_version_1 {
        // TrueType outlines - has more fields
        let max_points: Vec<u16> = tables.iter().map(|t| t.max_points().unwrap_or(0)).collect();
        let max_contours: Vec<u16> = tables.iter().map(|t| t.max_contours().unwrap_or(0)).collect();
        let max_composite_points: Vec<u16> =
            tables.iter().map(|t| t.max_composite_points().unwrap_or(0)).collect();
        let max_composite_contours: Vec<u16> = tables
            .iter()
            .map(|t| t.max_composite_contours().unwrap_or(0))
            .collect();
        let max_zones: Vec<u16> = tables.iter().map(|t| t.max_zones().unwrap_or(1)).collect();
        let max_twilight_points: Vec<u16> =
            tables.iter().map(|t| t.max_twilight_points().unwrap_or(0)).collect();
        let max_storage: Vec<u16> = tables.iter().map(|t| t.max_storage().unwrap_or(0)).collect();
        let max_function_defs: Vec<u16> =
            tables.iter().map(|t| t.max_function_defs().unwrap_or(0)).collect();
        let max_instruction_defs: Vec<u16> =
            tables.iter().map(|t| t.max_instruction_defs().unwrap_or(0)).collect();
        let max_stack_elements: Vec<u16> =
            tables.iter().map(|t| t.max_stack_elements().unwrap_or(0)).collect();
        let max_size_of_instructions: Vec<u16> = tables
            .iter()
            .map(|t| t.max_size_of_instructions().unwrap_or(0))
            .collect();
        let max_component_elements: Vec<u16> = tables
            .iter()
            .map(|t| t.max_component_elements().unwrap_or(0))
            .collect();
        let max_component_depth: Vec<u16> =
            tables.iter().map(|t| t.max_component_depth().unwrap_or(0)).collect();

        Ok(Maxp {
            num_glyphs: total_glyphs,
            max_points: Some(max(&max_points)?),
            max_contours: Some(max(&max_contours)?),
            max_composite_points: Some(max(&max_composite_points)?),
            max_composite_contours: Some(max(&max_composite_contours)?),
            max_zones: Some(max(&max_zones)?),
            max_twilight_points: Some(max(&max_twilight_points)?),
            max_storage: Some(first(&max_storage)?),
            max_function_defs: Some(first(&max_function_defs)?),
            max_instruction_defs: Some(first(&max_instruction_defs)?),
            max_stack_elements: Some(max(&max_stack_elements)?),
            max_size_of_instructions: Some(first(&max_size_of_instructions)?),
            max_component_elements: Some(max(&max_component_elements)?),
            max_component_depth: Some(max(&max_component_depth)?),
        })
    } else {
        // CFF outlines - version 0.5
        Ok(Maxp {
            num_glyphs: total_glyphs,
            max_points: None,
            max_contours: None,
            max_composite_points: None,
            max_composite_contours: None,
            max_zones: None,
            max_twilight_points: None,
            max_storage: None,
            max_function_defs: None,
            max_instruction_defs: None,
            max_stack_elements: None,
            max_size_of_instructions: None,
            max_component_elements: None,
            max_component_depth: None,
        })
    }
}
