//! OpenType feature definitions.

#[derive(Debug, Clone, Copy)]
pub struct FeatureTag(pub &'static str);

impl AsRef<str> for FeatureTag {
    fn as_ref(&self) -> &str {
        self.0
    }
}

const BASE_FEATURES: &[FeatureTag] = &[
    FeatureTag("dlig"),
    FeatureTag("ss01"),
    FeatureTag("ss02"),
    FeatureTag("ss03"),
    FeatureTag("ss04"),
    FeatureTag("ss05"),
    FeatureTag("ss06"),
    FeatureTag("ss07"),
    FeatureTag("ss08"),
    FeatureTag("ss10"),
    FeatureTag("ss11"),
    FeatureTag("ss12"),
    FeatureTag("liga"),
];

pub const MONO_FEATURES: &[FeatureTag] = &[
    FeatureTag("dlig"),
    FeatureTag("ss01"),
    FeatureTag("ss02"),
    FeatureTag("ss03"),
    FeatureTag("ss04"),
    FeatureTag("ss05"),
    FeatureTag("ss06"),
    FeatureTag("ss07"),
    FeatureTag("ss08"),
    FeatureTag("ss10"),
    FeatureTag("ss11"),
    FeatureTag("ss12"),
    FeatureTag("pnum"),
    FeatureTag("liga"),
];

pub const SANS_FEATURES: &[FeatureTag] = BASE_FEATURES;
