use std::{
    fs::{read, write},
    path::Path,
};

use anyhow::{Context, Result};
use hb_subset::{Blob, FontFace, SubsetInput, Tag};
use log::info;

/// Variable font tables to drop during subsetting (matches Python pipeline)
const VF_TABLES_TO_DROP: &[&[u8; 4]] = &[
    b"HVAR", // Horizontal metrics variations
    b"MVAR", // Metrics variations
    b"STAT", // Style attributes
    b"avar", // Axis variations
    b"fvar", // Font variations
    b"gvar", // Glyph variations
    b"cvar", // CVT variations
];

/// Japanese Unicode ranges for subsetting
pub const JAPANESE_RANGES: &[(u32, u32)] = &[
    // Basic Japanese
    (0x3000, 0x303F), // CJK Symbols and Punctuation
    (0x3041, 0x3096), // Hiragana (basic)
    (0x3099, 0x309F), // Hiragana (combining marks)
    (0x30A0, 0x30FF), // Katakana
    (0x4E00, 0x9FFF), // CJK Unified Ideographs
    (0xFF00, 0xFFEF), // Halfwidth and Fullwidth Forms
    // Extended Japanese
    (0x1B100, 0x1B12F), // Kana Extended-A
    (0x1AFF0, 0x1AFFF), // Kana Extended-B
    (0x1B000, 0x1B0FF), // Kana Supplement
    (0x1B130, 0x1B16F), // Small Kana Extension
    (0x3400, 0x4DBF),   // CJK Unified Ideographs Extension A
    (0x20000, 0x2A6DF), // CJK Unified Ideographs Extension B
    (0x2A700, 0x2B739), // CJK Unified Ideographs Extension C
    (0x2B740, 0x2B81D), // CJK Unified Ideographs Extension D
    (0x2B820, 0x2CEA1), // CJK Unified Ideographs Extension E
    (0x2CEB0, 0x2EBE0), // CJK Unified Ideographs Extension F
    (0x30000, 0x3134A), // CJK Unified Ideographs Extension G
    (0x31350, 0x323AF), // CJK Unified Ideographs Extension H
    (0x2EBF0, 0x2EE5D), // CJK Unified Ideographs Extension I
    (0xF900, 0xFAFF),   // CJK Compatibility Ideographs
    (0x2F800, 0x2FA1F), // CJK Compatibility Ideographs Supplement
];

/// Layout features to retain (matches pyftsubset --layout-features=*)
/// These are all features present in Noto Sans Mono CJK
const LAYOUT_FEATURES: &[&[u8; 4]] = &[
    // GSUB features
    b"aalt", // Access All Alternates
    b"ccmp", // Glyph Composition/Decomposition
    b"dlig", // Discretionary Ligatures (important!)
    b"fwid", // Full Widths
    b"hwid", // Half Widths
    b"jp78", // JIS78 Forms
    b"jp83", // JIS83 Forms
    b"jp90", // JIS90 Forms
    b"liga", // Standard Ligatures
    b"locl", // Localized Forms
    b"nlck", // NLC Kanji Forms
    b"pwid", // Proportional Widths
    b"vert", // Vertical Writing
    b"vjmo", // Vertical Jamo
    b"vrt2", // Vertical Alternates and Rotation
    // GPOS features
    b"halt", // Alternate Half Widths
    b"vhal", // Alternate Vertical Half Metrics
    b"kern", // Kerning
    b"mark", // Mark Positioning
    b"mkmk", // Mark to Mark Positioning
];

/// Subset a font to Japanese Unicode ranges using HarfBuzz
pub fn subset_japanese(input: &Path, output: &Path) -> Result<()> {
    let data = read(input).with_context(|| format!("Failed to read {}", input.display()))?;

    let mut subset_input = SubsetInput::new()?;

    // Retain glyph names (matches pyftsubset --glyph-names)
    subset_input.flags().retain_glyph_names();

    // Retain all layout features (matches pyftsubset --layout-features=*)
    // This ensures glyphs used by dlig, liga, etc. are kept
    {
        let mut feature_set = subset_input.layout_feature_tag_set();
        for tag in LAYOUT_FEATURES {
            feature_set.insert(Tag::new(*tag));
        }
    }

    // Add all Japanese Unicode ranges
    {
        let mut unicode_set = subset_input.unicode_set();
        for (start, end) in JAPANESE_RANGES {
            for cp in *start..=*end {
                if let Some(c) = char::from_u32(cp) {
                    unicode_set.insert(c);
                }
            }
        }
    }

    // Drop VF tables to reduce file size (convert VF to static font)
    {
        let mut drop_tables = subset_input.drop_table_tag_set();
        for table in VF_TABLES_TO_DROP {
            drop_tables.insert(Tag::new(*table));
        }
    }

    let font = FontFace::new(Blob::from_bytes(&data)?)?;
    let subset_font = subset_input.subset_font(&font)?;
    let blob = &*subset_font.underlying_blob();

    write(output, blob).with_context(|| format!("Failed to write {}", output.display()))?;

    let input_size = data.len() as f64 / 1024.0 / 1024.0;
    let output_size = blob.len() as f64 / 1024.0 / 1024.0;

    info!(
        "Subset {} -> {} ({input_size:.2} MB -> {output_size:.2} MB, {:.1}% reduction)",
        input.file_name().unwrap_or_default().to_string_lossy(),
        output.file_name().unwrap_or_default().to_string_lossy(),
        (1.0 - output_size / input_size) * 100.0
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_ranges_count() {
        assert_eq!(JAPANESE_RANGES.len(), 21);
    }
}
