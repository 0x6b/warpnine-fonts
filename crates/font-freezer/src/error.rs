//! Error types for font feature freezing operations.

use std::result;

use read_fonts::ReadError;
use write_fonts::BuilderError;

/// Errors that can occur during font feature freezing.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to parse font: {0}")]
    Parse(#[from] ReadError),

    #[error("no GSUB table in font")]
    NoGsub,

    #[error("no cmap table in font")]
    NoCmap,

    #[error("failed to build font: {0}")]
    Build(#[from] BuilderError),

    #[error("no matching features found for {0:?}")]
    NoMatchingFeatures(Vec<String>),

    #[error("no substitutions found for features {0:?}")]
    NoSubstitutions(Vec<String>),
}

pub type Result<T> = result::Result<T, Error>;
