//! Unified style definitions for font generation.

mod design;
mod features;
mod instances;

pub use design::{MONO_STYLES, SANS_STYLES, Slant, Style, Weight, WeightClass, duotone_casl};
pub use features::{FeatureTag, MONO_FEATURES, SANS_FEATURES};
pub use instances::build_style_instances;
