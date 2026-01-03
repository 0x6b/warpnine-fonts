//! Font subsetting wrapper around hb-subset with builder pattern.
//!
//! This crate provides a high-level interface for subsetting fonts using HarfBuzz's
//! hb-subset library. It operates purely on byte slices with no file I/O dependencies.
//!
//! # Example
//!
//! ```no_run
//! use warpnine_font_subsetter::{Subsetter, subset_japanese};
//!
//! // Using the builder pattern
//! let font_data: &[u8] = &[];
//! let subset = Subsetter::new()
//!     .with_unicode_ranges([(0x3000, 0x303F)])
//!     .drop_vf_tables(true)
//!     .subset(font_data);
//!
//! // Using the Japanese preset
//! let subset = Subsetter::japanese().subset(font_data);
//!
//! // Using the convenience function
//! let subset = subset_japanese(font_data);
//! ```

use anyhow::Result;
use hb_subset::{Blob, FontFace, SubsetInput, Tag};

/// Variable font tables to drop during subsetting.
///
/// These tables are specific to variable fonts and can be safely removed
/// when creating a static font subset.
pub const VF_TABLES_TO_DROP: &[&[u8; 4]] = &[
    b"HVAR", b"MVAR", b"STAT", b"avar", b"fvar", b"gvar", b"cvar",
];

/// Japanese Unicode ranges for subsetting.
///
/// Includes:
/// - CJK Symbols and Punctuation (U+3000-U+303F)
/// - Hiragana (U+3041-U+3096, U+3099-U+309F)
/// - Katakana (U+30A0-U+30FF)
/// - CJK Unified Ideographs (U+4E00-U+9FFF)
/// - Halfwidth and Fullwidth Forms (U+FF00-U+FFEF)
/// - Kana Extended/Supplement blocks
/// - CJK Extension blocks (A-I)
/// - CJK Compatibility Ideographs
pub const JAPANESE_RANGES: &[(u32, u32)] = &[
    (0x3000, 0x303F),
    (0x3041, 0x3096),
    (0x3099, 0x309F),
    (0x30A0, 0x30FF),
    (0x4E00, 0x9FFF),
    (0xFF00, 0xFFEF),
    (0x1B100, 0x1B12F),
    (0x1AFF0, 0x1AFFF),
    (0x1B000, 0x1B0FF),
    (0x1B130, 0x1B16F),
    (0x3400, 0x4DBF),
    (0x20000, 0x2A6DF),
    (0x2A700, 0x2B739),
    (0x2B740, 0x2B81D),
    (0x2B820, 0x2CEA1),
    (0x2CEB0, 0x2EBE0),
    (0x30000, 0x3134A),
    (0x31350, 0x323AF),
    (0x2EBF0, 0x2EE5D),
    (0xF900, 0xFAFF),
    (0x2F800, 0x2FA1F),
];

/// Layout features to retain during subsetting.
///
/// These OpenType features are commonly used for proper text rendering
/// and should be preserved in the subset font.
pub const LAYOUT_FEATURES: &[&[u8; 4]] = &[
    b"aalt", b"ccmp", b"dlig", b"fwid", b"hwid", b"jp78", b"jp83", b"jp90", b"liga", b"locl",
    b"nlck", b"pwid", b"vert", b"vjmo", b"vrt2", b"halt", b"vhal", b"kern", b"mark", b"mkmk",
];

/// Font subsetter with builder pattern.
///
/// Provides a flexible way to configure font subsetting options before
/// performing the subset operation.
#[derive(Default)]
pub struct Subsetter {
    unicode_ranges: Vec<(u32, u32)>,
    drop_vf_tables: bool,
    retain_glyph_names: bool,
    layout_features: Vec<[u8; 4]>,
}

impl Subsetter {
    /// Creates a new subsetter with default settings.
    ///
    /// Default settings use the standard [`LAYOUT_FEATURES`] and do not
    /// drop variable font tables or retain glyph names.
    pub fn new() -> Self {
        Self {
            layout_features: LAYOUT_FEATURES.iter().map(|f| **f).collect(),
            ..Default::default()
        }
    }

    /// Creates a subsetter pre-configured for Japanese font subsetting.
    ///
    /// This preset:
    /// - Uses [`JAPANESE_RANGES`] for Unicode coverage
    /// - Drops variable font tables
    /// - Retains glyph names
    /// - Uses standard [`LAYOUT_FEATURES`]
    pub fn japanese() -> Self {
        Self {
            unicode_ranges: JAPANESE_RANGES.to_vec(),
            drop_vf_tables: true,
            retain_glyph_names: true,
            layout_features: LAYOUT_FEATURES.iter().map(|f| **f).collect(),
        }
    }

    /// Adds Unicode ranges to include in the subset.
    ///
    /// Each range is a tuple of (start, end) Unicode code points, inclusive.
    pub fn with_unicode_ranges(mut self, ranges: impl IntoIterator<Item = (u32, u32)>) -> Self {
        self.unicode_ranges.extend(ranges);
        self
    }

    /// Sets whether to drop variable font tables.
    ///
    /// When `true`, tables like `fvar`, `gvar`, `avar`, etc. are removed,
    /// converting a variable font to a static font.
    pub fn drop_vf_tables(mut self, drop: bool) -> Self {
        self.drop_vf_tables = drop;
        self
    }

    /// Sets whether to retain glyph names in the subset.
    ///
    /// Glyph names can be useful for debugging but increase file size.
    pub fn retain_glyph_names(mut self, retain: bool) -> Self {
        self.retain_glyph_names = retain;
        self
    }

    /// Sets the layout features to retain in the subset.
    ///
    /// Replaces any previously configured layout features.
    pub fn with_layout_features(
        mut self,
        features: impl IntoIterator<Item = [u8; 4]>,
    ) -> Self {
        self.layout_features = features.into_iter().collect();
        self
    }

    /// Subsets the font data and returns the result.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw font file data
    ///
    /// # Returns
    ///
    /// The subset font data as a byte vector, or an error if subsetting fails.
    pub fn subset(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut input = SubsetInput::new()?;

        if self.retain_glyph_names {
            input.flags().retain_glyph_names();
        }

        {
            let mut feature_set = input.layout_feature_tag_set();
            for tag in &self.layout_features {
                feature_set.insert(Tag::new(tag));
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
}

/// Subsets font data for Japanese text (convenience function).
///
/// This is equivalent to calling `Subsetter::japanese().subset(data)`.
///
/// # Arguments
///
/// * `data` - The raw font file data
///
/// # Returns
///
/// The subset font data as a byte vector, or an error if subsetting fails.
pub fn subset_japanese(data: &[u8]) -> Result<Vec<u8>> {
    Subsetter::japanese().subset(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_ranges_count() {
        assert_eq!(JAPANESE_RANGES.len(), 21);
    }

    #[test]
    fn test_layout_features_count() {
        assert_eq!(LAYOUT_FEATURES.len(), 20);
    }

    #[test]
    fn test_vf_tables_count() {
        assert_eq!(VF_TABLES_TO_DROP.len(), 7);
    }

    #[test]
    fn test_builder_chain() {
        let subsetter = Subsetter::new()
            .with_unicode_ranges([(0x0000, 0x007F)])
            .drop_vf_tables(true)
            .retain_glyph_names(true)
            .with_layout_features([*b"kern", *b"liga"]);

        assert!(subsetter.drop_vf_tables);
        assert!(subsetter.retain_glyph_names);
        assert_eq!(subsetter.unicode_ranges.len(), 1);
        assert_eq!(subsetter.layout_features.len(), 2);
    }

    #[test]
    fn test_japanese_preset() {
        let subsetter = Subsetter::japanese();
        assert!(subsetter.drop_vf_tables);
        assert!(subsetter.retain_glyph_names);
        assert_eq!(subsetter.unicode_ranges.len(), 21);
    }
}
