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
