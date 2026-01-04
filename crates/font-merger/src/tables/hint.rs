//! TrueType hinting table handling
//!
//! TrueType fonts use several tables for hinting:
//!
//! - `fpgm` (font program): Global hinting functions shared across all glyphs
//! - `prep` (control value program): Setup code run before rendering each glyph
//! - `cvt ` (control value table): Control values used by hinting instructions
//!
//! Additionally, per-glyph hinting instructions are stored in the `glyf` table.
//!
//! # Current Behavior
//!
//! When merging fonts:
//!
//! 1. **Global hint tables (`fpgm`, `prep`, `cvt`)**: Copied from the first font only. This is
//!    because different fonts typically have incompatible hinting programs that cannot be safely
//!    merged.
//!
//! 2. **Per-glyph instructions**: Preserved from each font's glyphs as they are copied to the
//!    merged font. These instructions are stored in the `glyf` table.
//!
//! # Implications
//!
//! - Glyphs from the first font will render with their original hinting quality
//! - Glyphs from subsequent fonts may not hint correctly if they rely on `fpgm` functions or `cvt`
//!   values that differ from the first font
//! - For best results, merge fonts that share similar hinting strategies
//!
//! # Future Improvements
//!
//! A more sophisticated implementation could:
//!
//! 1. Analyze hint programs to detect compatibility
//! 2. Rename function numbers to avoid conflicts
//! 3. Concatenate `fpgm` tables with adjusted references
//! 4. Merge `cvt` tables with remapped indices
//! 5. Update per-glyph instructions to use new function/cvt indices
//!
//! However, this is extremely complex and may introduce subtle rendering bugs.

use log::{debug, warn};
use read_fonts::{FontRef, types::Tag};

/// Check if fonts have compatible hinting
///
/// Currently returns true if only one font has hinting tables, or if all
/// fonts share the same hinting data. Returns false (with a warning) if
/// fonts have different hinting programs.
pub fn check_hint_compatibility(fonts: &[FontRef]) -> bool {
    let fpgm_tag = Tag::new(b"fpgm");
    let prep_tag = Tag::new(b"prep");
    let cvt_tag = Tag::new(b"cvt ");

    let fonts_with_hints: Vec<_> = fonts
        .iter()
        .filter(|f| {
            f.table_data(fpgm_tag).is_some()
                || f.table_data(prep_tag).is_some()
                || f.table_data(cvt_tag).is_some()
        })
        .collect();

    if fonts_with_hints.len() <= 1 {
        return true;
    }

    // Check if all fonts have identical hinting tables
    let first = fonts_with_hints[0];
    let first_fpgm = first.table_data(fpgm_tag);
    let first_prep = first.table_data(prep_tag);
    let first_cvt = first.table_data(cvt_tag);

    for font in fonts_with_hints.iter().skip(1) {
        let fpgm = font.table_data(fpgm_tag);
        let prep = font.table_data(prep_tag);
        let cvt = font.table_data(cvt_tag);

        let fpgm_match = match (&first_fpgm, &fpgm) {
            (Some(a), Some(b)) => a.as_bytes() == b.as_bytes(),
            (None, None) => true,
            _ => false,
        };

        let prep_match = match (&first_prep, &prep) {
            (Some(a), Some(b)) => a.as_bytes() == b.as_bytes(),
            (None, None) => true,
            _ => false,
        };

        let cvt_match = match (&first_cvt, &cvt) {
            (Some(a), Some(b)) => a.as_bytes() == b.as_bytes(),
            (None, None) => true,
            _ => false,
        };

        if !fpgm_match || !prep_match || !cvt_match {
            warn!(
                "Fonts have different TrueType hinting programs. \
                 Only hinting from the first font will be used. \
                 Glyphs from other fonts may not hint correctly."
            );
            return false;
        }
    }

    true
}

/// Log information about hinting tables being merged
pub fn log_hint_info(fonts: &[FontRef]) {
    let fpgm_tag = Tag::new(b"fpgm");
    let prep_tag = Tag::new(b"prep");
    let cvt_tag = Tag::new(b"cvt ");

    for (i, font) in fonts.iter().enumerate() {
        let has_fpgm = font.table_data(fpgm_tag).is_some();
        let has_prep = font.table_data(prep_tag).is_some();
        let has_cvt = font.table_data(cvt_tag).is_some();

        if has_fpgm || has_prep || has_cvt {
            debug!("Font {i}: fpgm={has_fpgm}, prep={has_prep}, cvt={has_cvt}");
        }
    }
}
