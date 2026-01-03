//! Warpnine fonts CLI library.

pub mod cli;
pub mod commands;
pub mod io;
pub mod parallel;
pub mod styles;

// Batch processing wrappers (these add real value)
pub mod freeze;
pub mod instance;
pub mod merge;

// Project-specific operations
pub mod warpnine;

// Re-export from extracted crates for convenience
pub use warpnine_font_condense::apply_horizontal_scale;
pub use warpnine_font_metadata::{FontVersion, MonospaceSettings};
pub use warpnine_font_ops::{apply_family_style_names, copy_table, map_name_records, rewrite_font};
pub use warpnine_font_subsetter::{JAPANESE_RANGES, Subsetter};

pub use styles::{MONO_STYLES, SANS_STYLES, Slant, Style, Weight};
