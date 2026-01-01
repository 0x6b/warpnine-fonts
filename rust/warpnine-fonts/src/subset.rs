use anyhow::{Context, Result};
use hb_subset::{Blob, FontFace, SubsetInput};
use std::fs;
use std::path::Path;

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

/// Subset a font to Japanese Unicode ranges using HarfBuzz
pub fn subset_japanese(input: &Path, output: &Path) -> Result<()> {
    let data = fs::read(input).with_context(|| format!("Failed to read {}", input.display()))?;

    let mut subset_input = SubsetInput::new()?;

    // Add all Japanese Unicode ranges
    let mut unicode_set = subset_input.unicode_set();
    for (start, end) in JAPANESE_RANGES {
        for cp in *start..=*end {
            if let Some(c) = char::from_u32(cp) {
                unicode_set.insert(c);
            }
        }
    }

    let font = FontFace::new(Blob::from_bytes(&data)?)?;
    let subset_font = subset_input.subset_font(&font)?;
    let blob = &*subset_font.underlying_blob();

    fs::write(output, blob.to_vec())
        .with_context(|| format!("Failed to write {}", output.display()))?;

    let input_size = data.len() as f64 / 1024.0 / 1024.0;
    let output_size = blob.len() as f64 / 1024.0 / 1024.0;

    println!(
        "Subset {} -> {} ({:.2} MB -> {:.2} MB, {:.1}% reduction)",
        input.file_name().unwrap_or_default().to_string_lossy(),
        output.file_name().unwrap_or_default().to_string_lossy(),
        input_size,
        output_size,
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
