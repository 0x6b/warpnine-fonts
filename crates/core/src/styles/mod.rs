//! Unified style definitions for font generation.

mod design;
mod features;
mod instances;

pub use design::{duotone_casl, Slant, Style, Weight, WeightClass, MONO_STYLES, SANS_STYLES};
pub use features::{FeatureTag, MONO_FEATURES, SANS_FEATURES};
pub use instances::build_style_instances;
