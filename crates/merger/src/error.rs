use std::{io, result};

use read_fonts::ReadError;
use thiserror::Error;
use write_fonts::{error, BuilderError};

#[derive(Error, Debug)]
pub enum MergeError {
    #[error("failed to read font: {0}")]
    ReadError(#[from] ReadError),

    #[error("failed to write font: {0}")]
    WriteError(#[from] error::Error),

    #[error("failed to build font: {0}")]
    BuilderError(#[from] BuilderError),

    #[error("no fonts provided for merging")]
    NoFonts,

    #[error("fonts have incompatible unitsPerEm: expected {expected}, got {actual}")]
    IncompatibleUnitsPerEm { expected: u16, actual: u16 },

    #[error("table values must be equal for '{table}' field '{field}'")]
    NotEqual { table: &'static str, field: &'static str },

    #[error("required table '{0}' not found")]
    MissingTable(&'static str),

    #[error("CID-keyed CFF fonts are not supported")]
    CidKeyedCffNotSupported,

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("failed to build cmap table")]
    CmapBuildError,
}

pub type Result<T> = result::Result<T, MergeError>;
