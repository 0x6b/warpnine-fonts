//! CFF table merging
//!
//! **Current Limitations:**
//!
//! CFF (Compact Font Format) table merging is not yet fully implemented.
//! The current implementation copies the CFF table from the first font only.
//!
//! A proper implementation would require:
//! 1. Parsing CFF charstrings from all fonts
//! 2. Desubroutinizing charstrings (inlining all local and global subroutines)
//! 3. Reordering charstrings to match the mega glyph order
//! 4. Combining charstrings into a single CFF table
//! 5. Optionally re-subroutinizing for size optimization
//!
//! This is complex because:
//! - CFF uses a stack-based bytecode for charstrings
//! - Subroutines are shared code sequences that must be inlined before merging
//! - The `write-fonts` crate doesn't yet have CFF table support
//!
//! For fonts with CFF outlines, users may want to convert to TrueType outlines
//! before merging, or use fontTools' merge functionality instead.

use log::{info, warn};
use read_fonts::{FontRef, TableProvider};

use crate::{Result, context::MergeContext};

/// Check if any font has CFF outlines
///
/// Returns true if at least one font has a CFF table.
/// Note: CID-keyed CFF fonts are not currently supported.
pub fn check_cff(fonts: &[FontRef]) -> Result<bool> {
    for font in fonts {
        if font.cff().is_ok() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Merge CFF tables from multiple fonts
///
/// # Current Implementation
///
/// This function currently **only copies the CFF table from the first font**.
/// Glyphs from subsequent fonts will not be included in the output.
///
/// # Warnings
///
/// When merging multiple CFF fonts, a warning is logged because the merge
/// is incomplete. The resulting font will only contain glyphs from the first font.
///
/// # Future Implementation
///
/// A proper implementation would:
/// 1. Parse each CFF table using a CFF parser
/// 2. Desubroutinize all charstrings (inline subroutine calls)
/// 3. Create a new charstring index with glyphs in mega glyph order
/// 4. Build a new CFF table with combined data
/// 5. Optionally apply subroutinization for size optimization
pub fn merge_cff(ctx: &MergeContext) -> Result<Option<Vec<u8>>> {
    // Check if any font has CFF
    let cff_fonts: Vec<_> = ctx.fonts().iter().filter(|f| f.cff().is_ok()).collect();
    if cff_fonts.is_empty() {
        return Ok(None);
    }

    // Warn if we're merging multiple CFF fonts
    if cff_fonts.len() > 1 {
        warn!(
            "CFF merging is not fully implemented. Only glyphs from the first font will be included. \
             {} fonts have CFF outlines but only the first will be used.",
            cff_fonts.len()
        );
    }

    // Return the first font's CFF table
    if let Some(font) = cff_fonts.first()
        && let Some(data) = font.table_data(read_fonts::types::Tag::new(b"CFF "))
    {
        info!("Copying CFF table from first font ({} bytes)", data.len());
        return Ok(Some(data.as_bytes().to_vec()));
    }

    Ok(None)
}

/// Merge CFF2 tables from multiple fonts
///
/// # Current Implementation
///
/// Like CFF merging, this currently **only copies the CFF2 table from the first font**.
/// CFF2 is used for variable fonts and has similar complexity to CFF.
pub fn merge_cff2(ctx: &MergeContext) -> Result<Option<Vec<u8>>> {
    let cff2_fonts: Vec<_> = ctx.fonts().iter().filter(|f| f.cff2().is_ok()).collect();
    if cff2_fonts.is_empty() {
        return Ok(None);
    }

    if cff2_fonts.len() > 1 {
        warn!(
            "CFF2 merging is not fully implemented. Only the first font's CFF2 table will be used. \
             {} fonts have CFF2 tables.",
            cff2_fonts.len()
        );
    }

    if let Some(font) = cff2_fonts.first()
        && let Some(data) = font.table_data(read_fonts::types::Tag::new(b"CFF2"))
    {
        info!("Copying CFF2 table from first font ({} bytes)", data.len());
        return Ok(Some(data.as_bytes().to_vec()));
    }

    Ok(None)
}
