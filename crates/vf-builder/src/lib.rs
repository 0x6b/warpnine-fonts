//! # Variable Font Builder
//!
//! Build variable fonts from static master fonts.
//!
//! This crate provides functionality similar to fontTools varLib, allowing
//! construction of variable fonts from a set of static master fonts with
//! defined axis locations.
//!
//! ## Example
//!
//! ```no_run
//! use warpnine_font_vf_builder::{DesignSpace, Axis, Source, build_variable_font};
//!
//! let designspace = DesignSpace::new(
//!     vec![
//!         Axis::new("wght", "Weight", 300.0, 400.0, 1000.0),
//!         Axis::new("ital", "Italic", 0.0, 0.0, 1.0),
//!     ],
//!     vec![
//!         Source::new("Regular.ttf", vec![("wght", 400.0), ("ital", 0.0)]),
//!         Source::new("Bold.ttf", vec![("wght", 700.0), ("ital", 0.0)]),
//!     ],
//! );
//!
//! let vf_data = build_variable_font(&designspace).unwrap();
//! std::fs::write("Variable.ttf", vf_data).unwrap();
//! ```

mod designspace;
mod error;
mod variation_model;
mod vf_builder;

pub use designspace::{Axis, DesignSpace, Instance, Source};
pub use error::{Error, Result};
pub use vf_builder::build_variable_font;
