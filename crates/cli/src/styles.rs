//! Unified style definitions for font generation.

/// A font style definition with weight and italic flag.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub name: &'static str,
    pub wght: f32,
    pub italic: bool,
}

impl Style {
    pub const fn new(name: &'static str, wght: f32, italic: bool) -> Self {
        Self { name, wght, italic }
    }

    pub fn slnt(&self) -> f32 {
        if self.italic { -15.0 } else { 0.0 }
    }

    pub fn crsv(&self) -> f32 {
        if self.italic { 1.0 } else { 0.5 }
    }

    pub fn ital(&self) -> f32 {
        if self.italic { 1.0 } else { 0.0 }
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
}

/// All 16 WarpnineMono styles (8 weights × 2 italic states).
pub const MONO_STYLES: &[Style] = &[
    Style::new("Light", 300.0, false),
    Style::new("LightItalic", 300.0, true),
    Style::new("Regular", 400.0, false),
    Style::new("Italic", 400.0, true),
    Style::new("Medium", 500.0, false),
    Style::new("MediumItalic", 500.0, true),
    Style::new("SemiBold", 600.0, false),
    Style::new("SemiBoldItalic", 600.0, true),
    Style::new("Bold", 700.0, false),
    Style::new("BoldItalic", 700.0, true),
    Style::new("ExtraBold", 800.0, false),
    Style::new("ExtraBoldItalic", 800.0, true),
    Style::new("Black", 900.0, false),
    Style::new("BlackItalic", 900.0, true),
    Style::new("ExtraBlack", 1000.0, false),
    Style::new("ExtraBlackItalic", 1000.0, true),
];

/// Sans styles (14 styles - no ExtraBlack).
pub const SANS_STYLES: &[Style] = &[
    Style::new("Light", 300.0, false),
    Style::new("LightItalic", 300.0, true),
    Style::new("Regular", 400.0, false),
    Style::new("Italic", 400.0, true),
    Style::new("Medium", 500.0, false),
    Style::new("MediumItalic", 500.0, true),
    Style::new("SemiBold", 600.0, false),
    Style::new("SemiBoldItalic", 600.0, true),
    Style::new("Bold", 700.0, false),
    Style::new("BoldItalic", 700.0, true),
    Style::new("ExtraBold", 800.0, false),
    Style::new("ExtraBoldItalic", 800.0, true),
    Style::new("Black", 900.0, false),
    Style::new("BlackItalic", 900.0, true),
];

/// CASL axis value for duotone instances (Linear for Light/Regular, Casual for Medium+).
pub fn duotone_casl(wght: f32) -> f32 {
    if wght < 500.0 { 0.0 } else { 1.0 }
}

/// Features to freeze in mono fonts.
pub const MONO_FEATURES: &[&str] = &[
    "dlig", "ss01", "ss02", "ss03", "ss04", "ss05", "ss06", "ss07", "ss08", "ss10", "ss11", "ss12",
    "pnum", "liga",
];
