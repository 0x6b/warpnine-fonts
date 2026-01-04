//! Warpnine Core - reusable build pipeline logic for Warpnine fonts.

pub mod freeze;
pub mod instance;
pub mod io;
pub mod merge;
pub mod parallel;
pub mod pipeline;
pub mod styles;
pub mod warpnine;

pub use warpnine_font_condense::apply_horizontal_scale;
pub use warpnine_font_metadata::{FontVersion, MonospaceSettings};
pub use warpnine_font_ops::{apply_family_style_names, copy_table, map_name_records, rewrite_font};
pub use warpnine_font_subsetter::{JAPANESE_RANGES, Subsetter};

pub use styles::{MONO_STYLES, SANS_STYLES, Slant, Style, Weight};

pub use pipeline::{
    PipelineContext, build_all, build_condensed, build_mono, build_sans, build_warpnine_mono_vf,
    warpnine_mono_designspace,
};
