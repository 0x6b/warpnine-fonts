//! Font subsetting with HarfBuzz.

use std::path::Path;

use anyhow::Result;
use log::info;

use crate::io::{read_font, write_font};

pub use warpnine_font_subsetter::{JAPANESE_RANGES, LAYOUT_FEATURES, Subsetter, VF_TABLES_TO_DROP};

/// Subset a font file to an output path using the given subsetter.
pub fn subset_file(subsetter: &Subsetter, input: &Path, output: &Path) -> Result<()> {
    let data = read_font(input)?;
    let subset_data = subsetter.subset(&data)?;
    write_font(output, &subset_data)?;

    let input_size = data.len() as f64 / 1024.0 / 1024.0;
    let output_size = subset_data.len() as f64 / 1024.0 / 1024.0;

    info!(
        "Subset {} -> {} ({input_size:.2} MB -> {output_size:.2} MB, {:.1}% reduction)",
        input.file_name().unwrap_or_default().to_string_lossy(),
        output.file_name().unwrap_or_default().to_string_lossy(),
        (1.0 - output_size / input_size) * 100.0
    );

    Ok(())
}

/// Subset a font to Japanese Unicode ranges (convenience function).
pub fn subset_japanese(input: &Path, output: &Path) -> Result<()> {
    subset_file(&Subsetter::japanese(), input, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_ranges_count() {
        assert_eq!(JAPANESE_RANGES.len(), 21);
    }
}
