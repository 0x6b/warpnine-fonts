use std::result;

use read_fonts::ReadError;
use write_fonts::{BuilderError, error};

/// Error types for font-instancer.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to parse font: {0}")]
    Parse(#[from] ReadError),

    #[error("not a variable font (no fvar table)")]
    NotVariableFont,

    #[error("no glyf table (CFF/CFF2 fonts not supported)")]
    NoCff2Support,

    #[error("no gvar table")]
    NoGvar,

    #[error("axis not found: {0}")]
    AxisNotFound(String),

    #[error("failed to build font: {0}")]
    Build(#[from] BuilderError),

    #[error("failed to write table: {0}")]
    Write(#[from] error::Error),

    #[error("invalid axis value {value} for {tag} (range: {min}..{max})")]
    InvalidAxisValue { tag: String, value: f32, min: f32, max: f32 },
}

pub type Result<T> = result::Result<T, Error>;
