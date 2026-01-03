//! Font subsetting with HarfBuzz.

use std::path::Path;

use anyhow::Result;
use hb_subset::{Blob, FontFace, SubsetInput, Tag};
use log::info;

use crate::io::{read_font, write_font};

/// Variable font tables to drop during subsetting
const VF_TABLES_TO_DROP: &[&[u8; 4]] = &[
    b"HVAR", b"MVAR", b"STAT", b"avar", b"fvar", b"gvar", b"cvar",
];

/// Japanese Unicode ranges for subsetting
pub const JAPANESE_RANGES: &[(u32, u32)] = &[
    (0x3000, 0x303F), (0x3041, 0x3096), (0x3099, 0x309F), (0x30A0, 0x30FF),
    (0x4E00, 0x9FFF), (0xFF00, 0xFFEF), (0x1B100, 0x1B12F), (0x1AFF0, 0x1AFFF),
    (0x1B000, 0x1B0FF), (0x1B130, 0x1B16F), (0x3400, 0x4DBF), (0x20000, 0x2A6DF),
    (0x2A700, 0x2B739), (0x2B740, 0x2B81D), (0x2B820, 0x2CEA1), (0x2CEB0, 0x2EBE0),
    (0x30000, 0x3134A), (0x31350, 0x323AF), (0x2EBF0, 0x2EE5D), (0xF900, 0xFAFF),
    (0x2F800, 0x2FA1F),
];

/// Layout features to retain during subsetting
const LAYOUT_FEATURES: &[&[u8; 4]] = &[
    b"aalt", b"ccmp", b"dlig", b"fwid", b"hwid", b"jp78", b"jp83", b"jp90",
    b"liga", b"locl", b"nlck", b"pwid", b"vert", b"vjmo", b"vrt2",
    b"halt", b"vhal", b"kern", b"mark", b"mkmk",
];

/// Font subsetter with builder pattern.
#[derive(Default)]
pub struct Subsetter {
    unicode_ranges: Vec<(u32, u32)>,
    drop_vf_tables: bool,
    retain_glyph_names: bool,
}

impl Subsetter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure for Japanese subsetting with all standard options.
    pub fn japanese() -> Self {
        Self {
            unicode_ranges: JAPANESE_RANGES.to_vec(),
            drop_vf_tables: true,
            retain_glyph_names: true,
        }
    }

    pub fn with_unicode_ranges(mut self, ranges: impl IntoIterator<Item = (u32, u32)>) -> Self {
        self.unicode_ranges.extend(ranges);
        self
    }

    pub fn drop_vf_tables(mut self, drop: bool) -> Self {
        self.drop_vf_tables = drop;
        self
    }

    pub fn retain_glyph_names(mut self, retain: bool) -> Self {
        self.retain_glyph_names = retain;
        self
    }

    /// Subset font data and return the result.
    pub fn subset(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut input = SubsetInput::new()?;

        if self.retain_glyph_names {
            input.flags().retain_glyph_names();
        }

        {
            let mut feature_set = input.layout_feature_tag_set();
            for tag in LAYOUT_FEATURES {
                feature_set.insert(Tag::new(*tag));
            }
        }

        {
            let mut unicode_set = input.unicode_set();
            for (start, end) in &self.unicode_ranges {
                for cp in *start..=*end {
                    if let Some(c) = char::from_u32(cp) {
                        unicode_set.insert(c);
                    }
                }
            }
        }

        if self.drop_vf_tables {
            let mut drop_tables = input.drop_table_tag_set();
            for table in VF_TABLES_TO_DROP {
                drop_tables.insert(Tag::new(*table));
            }
        }

        let font = FontFace::new(Blob::from_bytes(data)?)?;
        let subset_font = input.subset_font(&font)?;
        Ok(subset_font.underlying_blob().to_vec())
    }

    /// Subset a font file to an output path.
    pub fn subset_file(&self, input: &Path, output: &Path) -> Result<()> {
        let data = read_font(input)?;
        let subset_data = self.subset(&data)?;
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
}

/// Subset a font to Japanese Unicode ranges (convenience function).
pub fn subset_japanese(input: &Path, output: &Path) -> Result<()> {
    Subsetter::japanese().subset_file(input, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_ranges_count() {
        assert_eq!(JAPANESE_RANGES.len(), 21);
    }
}
