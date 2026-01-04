mod context;
mod convert;
mod error;
mod glyph_order;
mod merger;
mod options;
mod strategies;
mod tables;
mod types;

pub use context::{GidRemap, GlyphOrder, MergeContext};
pub use convert::ToWrite;
pub use error::{MergeError, Result};
pub use glyph_order::GlyphName;
pub use merger::Merger;
pub use options::Options;
pub use types::{Codepoint, FontIndex, GlyphId, MegaGlyphId, TableTag};

/// Merge multiple fonts from raw byte slices using default options.
///
/// This is a convenience wrapper around [`Merger`] for the common case
/// of merging fonts with default settings.
///
/// # Example
///
/// ```no_run
/// use warpnine_font_merger::merge_fonts_bytes;
///
/// let font1 = std::fs::read("font1.ttf").unwrap();
/// let font2 = std::fs::read("font2.ttf").unwrap();
/// let merged = merge_fonts_bytes(&[&font1, &font2]).unwrap();
/// ```
pub fn merge_fonts_bytes(fonts: &[&[u8]]) -> Result<Vec<u8>> {
    Merger::default().merge(fonts)
}
