//! Error types for variable font building.

use std::path::PathBuf;

/// Result type for variable font building operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during variable font building.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to read a source font file.
    #[error("Failed to read font file '{path}': {source}")]
    ReadFont {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to parse a font file.
    #[error("Failed to parse font '{path}': {message}")]
    ParseFont { path: PathBuf, message: String },

    /// Source font is missing a required table.
    #[error("Font '{path}' is missing required table '{table}'")]
    MissingTable { path: PathBuf, table: String },

    /// Glyph count mismatch between masters.
    #[error("Glyph count mismatch: master '{path}' has {actual} glyphs, expected {expected}")]
    GlyphCountMismatch {
        path: PathBuf,
        expected: u16,
        actual: u16,
    },

    /// Point count mismatch for a glyph between masters.
    #[error("Point count mismatch for glyph {glyph_id}: master '{path}' has {actual} points, expected {expected}")]
    PointCountMismatch {
        path: PathBuf,
        glyph_id: u32,
        expected: usize,
        actual: usize,
    },

    /// Contour count mismatch for a glyph between masters.
    #[error("Contour count mismatch for glyph {glyph_id}: master '{path}' has {actual} contours, expected {expected}")]
    ContourCountMismatch {
        path: PathBuf,
        glyph_id: u32,
        expected: usize,
        actual: usize,
    },

    /// Invalid designspace configuration.
    #[error("Invalid designspace: {0}")]
    InvalidDesignspace(String),

    /// No default source found in designspace.
    #[error("No source at default location found in designspace")]
    NoDefaultSource,

    /// Failed to build font table.
    #[error("Failed to build {table} table: {message}")]
    BuildTable { table: String, message: String },

    /// Font builder error.
    #[error("Font builder error: {0}")]
    FontBuilder(#[from] write_fonts::BuilderError),

    /// Read error.
    #[error("Font read error: {0}")]
    ReadError(#[from] read_fonts::ReadError),

    /// Write error.
    #[error("Font write error: {0}")]
    WriteError(#[from] write_fonts::error::Error),

    /// Gvar building error.
    #[error("Error building gvar table: {0:?}")]
    GvarBuild(write_fonts::tables::gvar::GvarInputError),
}
