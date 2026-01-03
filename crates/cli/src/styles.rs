//! Unified style definitions for font generation.

use font_instancer::AxisLocation;

/// Font slant (upright or italic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slant {
    Upright,
    Italic,
}

impl Slant {
    pub const fn slnt(self) -> f32 {
        match self {
            Slant::Upright => 0.0,
            Slant::Italic => -15.0,
        }
    }

    pub const fn crsv(self) -> f32 {
        match self {
            Slant::Upright => 0.5,
            Slant::Italic => 1.0,
        }
    }

    pub const fn ital(self) -> f32 {
        match self {
            Slant::Upright => 0.0,
            Slant::Italic => 1.0,
        }
    }

    pub const fn is_italic(self) -> bool {
        matches!(self, Slant::Italic)
    }
}

/// OS/2 weight class (usWeightClass) as a newtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightClass(pub u16);

impl From<Weight> for WeightClass {
    fn from(weight: Weight) -> Self {
        WeightClass(weight.0 as u16)
    }
}

/// Font weight as a newtype for type safety.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Weight(pub f32);

impl Weight {
    pub fn as_class(&self) -> WeightClass {
        WeightClass::from(*self)
    }
}

/// A font style definition with weight and slant.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub name: &'static str,
    pub weight: Weight,
    pub slant: Slant,
}

impl Style {
    pub const fn new(name: &'static str, weight: Weight, slant: Slant) -> Self {
        Self { name, weight, slant }
    }

    /// Display name with space before "Italic" (e.g., "LightItalic" -> "Light Italic").
    /// "Italic" alone stays as "Italic".
    pub fn display_name(&self) -> String {
        if self.name.ends_with("Italic") && self.name != "Italic" {
            let base = self.name.strip_suffix("Italic").unwrap();
            format!("{base} Italic")
        } else {
            self.name.to_string()
        }
    }

    pub fn axis_locations(&self, mono: f32, casl: f32) -> [AxisLocation; 5] {
        [
            AxisLocation::new("MONO", mono),
            AxisLocation::new("CASL", casl),
            AxisLocation::new("wght", self.weight.0),
            AxisLocation::new("slnt", self.slant.slnt()),
            AxisLocation::new("CRSV", self.slant.crsv()),
        ]
    }
}

/// All 16 WarpnineMono styles (8 weights × 2 italic states).
pub const MONO_STYLES: &[Style] = &[
    Style::new("Light", Weight(300.0), Slant::Upright),
    Style::new("LightItalic", Weight(300.0), Slant::Italic),
    Style::new("Regular", Weight(400.0), Slant::Upright),
    Style::new("Italic", Weight(400.0), Slant::Italic),
    Style::new("Medium", Weight(500.0), Slant::Upright),
    Style::new("MediumItalic", Weight(500.0), Slant::Italic),
    Style::new("SemiBold", Weight(600.0), Slant::Upright),
    Style::new("SemiBoldItalic", Weight(600.0), Slant::Italic),
    Style::new("Bold", Weight(700.0), Slant::Upright),
    Style::new("BoldItalic", Weight(700.0), Slant::Italic),
    Style::new("ExtraBold", Weight(800.0), Slant::Upright),
    Style::new("ExtraBoldItalic", Weight(800.0), Slant::Italic),
    Style::new("Black", Weight(900.0), Slant::Upright),
    Style::new("BlackItalic", Weight(900.0), Slant::Italic),
    Style::new("ExtraBlack", Weight(1000.0), Slant::Upright),
    Style::new("ExtraBlackItalic", Weight(1000.0), Slant::Italic),
];

/// Sans styles (14 styles - no ExtraBlack).
pub const SANS_STYLES: &[Style] = &[
    Style::new("Light", Weight(300.0), Slant::Upright),
    Style::new("LightItalic", Weight(300.0), Slant::Italic),
    Style::new("Regular", Weight(400.0), Slant::Upright),
    Style::new("Italic", Weight(400.0), Slant::Italic),
    Style::new("Medium", Weight(500.0), Slant::Upright),
    Style::new("MediumItalic", Weight(500.0), Slant::Italic),
    Style::new("SemiBold", Weight(600.0), Slant::Upright),
    Style::new("SemiBoldItalic", Weight(600.0), Slant::Italic),
    Style::new("Bold", Weight(700.0), Slant::Upright),
    Style::new("BoldItalic", Weight(700.0), Slant::Italic),
    Style::new("ExtraBold", Weight(800.0), Slant::Upright),
    Style::new("ExtraBoldItalic", Weight(800.0), Slant::Italic),
    Style::new("Black", Weight(900.0), Slant::Upright),
    Style::new("BlackItalic", Weight(900.0), Slant::Italic),
];

/// CASL axis value for duotone instances (Linear for Light/Regular, Casual for Medium+).
pub fn duotone_casl(wght: f32) -> f32 {
    if wght < 500.0 { 0.0 } else { 1.0 }
}

#[derive(Debug, Clone, Copy)]
pub struct FeatureTag(pub &'static str);

impl AsRef<str> for FeatureTag {
    fn as_ref(&self) -> &str {
        self.0
    }
}

/// Base features shared by both mono and sans fonts.
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

/// Features to freeze in mono fonts (base + pnum).
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

/// Features to freeze in sans fonts (same as base).
pub const SANS_FEATURES: &[FeatureTag] = BASE_FEATURES;

