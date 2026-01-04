//! # Font Feature Freezer
//!
//! Permanently apply OpenType GSUB features by remapping the cmap table.
//!
//! This is a Rust port of [fonttools-opentype-feature-freezer](https://github.com/twardoch/fonttools-opentype-feature-freezer).
//!
//! ## Example
//!
//! ```no_run
//! use font_feature_freezer::{Font, FreezeOptions};
//!
//! let data = std::fs::read("input.ttf").unwrap();
//! let font = Font::new(&data).unwrap();
//! let options = FreezeOptions::new(["ss01", "ss02"]);
//! let frozen = font.freeze(&options).unwrap();
//! std::fs::write("output.ttf", frozen.data).unwrap();
//! ```

mod error;
mod font;
mod gsub;
mod types;

pub use error::{Error, Result};
pub use font::Font;
pub use gsub::GlyphSubstitutions;
pub use types::{
    FontReport, FreezeOptions, FreezeResult, FreezeStats, ScriptLangFilter, SuffixConfig,
};

/// Generate a report of available scripts, languages, and features.
pub fn report(data: &[u8]) -> Result<FontReport> {
    Font::new(data)?.report()
}

/// Freeze OpenType features into font data.
pub fn freeze(data: &[u8], options: &FreezeOptions) -> Result<FreezeResult> {
    Font::new(data)?.freeze(options)
}

/// Freeze OpenType features into font data (simple API).
///
/// Accepts any iterable of string-like items (e.g., `["smcp", "onum"]`, `vec!["ss01"]`).
pub fn freeze_features<I, S>(data: &[u8], features: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    freeze(data, &options_from_features(features)).map(|r| r.data)
}

/// Freeze OpenType features into font data, returning statistics (simple API).
///
/// Accepts any iterable of string-like items (e.g., `["smcp", "onum"]`, `vec!["ss01"]`).
pub fn freeze_features_with_stats<I, S>(data: &[u8], features: I) -> Result<(Vec<u8>, FreezeStats)>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    freeze(data, &options_from_features(features)).map(|r| (r.data, r.stats))
}

/// Create FreezeOptions from an iterable of feature tags.
///
/// This is a convenience function for creating options with just feature tags.
///
/// # Example
///
/// ```
/// use font_feature_freezer::options_from_features;
///
/// let options = options_from_features(["smcp", "onum"]);
/// ```
pub fn options_from_features<I, S>(features: I) -> FreezeOptions
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    FreezeOptions::new(features.into_iter().map(|s| s.as_ref().to_owned()))
}
