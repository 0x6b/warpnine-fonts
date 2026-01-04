//! OpenType feature definitions.

#[derive(Debug, Clone, Copy)]
pub struct FeatureTag(pub &'static str);

impl AsRef<str> for FeatureTag {
    fn as_ref(&self) -> &str {
        self.0
    }
}

/// Features to freeze for WarpnineMono (static and variable).
pub const MONO_FEATURES: &[FeatureTag] = &[
    FeatureTag("dlig"),  // Discretionary ligatures (programming ligatures)
    FeatureTag("ss01"),  // Single-story a
    FeatureTag("ss02"),  // Single-story g
    FeatureTag("ss03"),  // Simplified f
    FeatureTag("ss04"),  // Simplified i
    FeatureTag("ss05"),  // Simplified l
    FeatureTag("ss06"),  // Simplified r
    FeatureTag("ss07"),  // Simplified italic diagonals / Serifless I
    FeatureTag("ss08"),  // No-serif L and Z
    FeatureTag("ss10"),  // Dotted zero
    FeatureTag("ss11"),  // Simplified 1
    FeatureTag("ss12"),  // Simplified @
    FeatureTag("pnum"),  // Proportional numerals
    FeatureTag("liga"),  // Standard ligatures
];

/// Features to freeze for WarpnineSans and WarpnineSansCondensed.
pub const SANS_FEATURES: &[FeatureTag] = &[
    FeatureTag("ss01"),  // Single-story a
    FeatureTag("ss02"),  // Single-story g
    FeatureTag("ss03"),  // Simplified f
    FeatureTag("ss04"),  // Simplified i
    FeatureTag("ss05"),  // Simplified l
    FeatureTag("ss06"),  // Simplified r
    FeatureTag("ss07"),  // Simplified italic diagonals / Serifless I
    FeatureTag("ss08"),  // No-serif L and Z
    FeatureTag("ss10"),  // Dotted zero
    FeatureTag("ss12"),  // Simplified @
    FeatureTag("case"),  // Case-sensitive forms
    FeatureTag("pnum"),  // Proportional numerals
    FeatureTag("liga"),  // Standard ligatures
];
