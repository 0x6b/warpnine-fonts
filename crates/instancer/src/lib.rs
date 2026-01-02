//! # Font Instancer
//!
//! Convert variable fonts to static instances.
//!
//! A Rust port of fonttools varLib.instancer for TrueType outline fonts.
//!
//! ## Example
//!
//! ```no_run
//! use font_instancer::{instantiate, AxisLocation};
//!
//! let vf_data = std::fs::read("variable.ttf").unwrap();
//! let location = [
//!     AxisLocation::new("wght", 700.0),
//!     AxisLocation::new("wdth", 100.0),
//! ];
//! let static_font = instantiate(&vf_data, &location).unwrap();
//! std::fs::write("static.ttf", static_font).unwrap();
//! ```
//!
//! You can also use tuple syntax:
//!
//! ```no_run
//! use font_instancer::{instantiate, AxisLocation};
//!
//! let vf_data = std::fs::read("variable.ttf").unwrap();
//! let location: Vec<AxisLocation> = [("wght", 700.0), ("wdth", 100.0)]
//!     .into_iter()
//!     .map(Into::into)
//!     .collect();
//! let static_font = instantiate(&vf_data, &location).unwrap();
//! ```

mod error;
mod instancer;

pub use error::{Error, Result};
pub use instancer::instantiate;
use read_fonts::types::Tag;

/// Axis location specification (tag + user-space value).
///
/// The value is in user-space coordinates (design units), matching what
/// you'd see in font variation settings. For example, `wght=700` for Bold.
#[derive(Debug, Clone, Copy)]
pub struct AxisLocation {
    pub tag: Tag,
    pub value: f32,
}

impl AxisLocation {
    /// Create a new axis location.
    ///
    /// # Example
    ///
    /// ```
    /// use font_instancer::AxisLocation;
    /// let loc = AxisLocation::new("wght", 700.0);
    /// ```
    pub fn new(tag: &str, value: f32) -> Self {
        let mut tag_bytes = [b' '; 4];
        for (dst, src) in tag_bytes.iter_mut().zip(tag.as_bytes().iter()) {
            *dst = *src;
        }
        Self {
            tag: Tag::new(&tag_bytes),
            value,
        }
    }
}

impl From<(&str, f32)> for AxisLocation {
    fn from((tag, value): (&str, f32)) -> Self {
        Self::new(tag, value)
    }
}

impl From<(Tag, f32)> for AxisLocation {
    fn from((tag, value): (Tag, f32)) -> Self {
        Self { tag, value }
    }
}
