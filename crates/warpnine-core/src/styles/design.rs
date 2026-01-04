//! Style and design type definitions.

use font_instancer::AxisLocation;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightClass(pub u16);

impl From<Weight> for WeightClass {
    fn from(weight: Weight) -> Self {
        WeightClass(weight.0 as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Weight(pub f32);

impl Weight {
    pub fn as_class(&self) -> WeightClass {
        WeightClass::from(*self)
    }
}

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

pub fn duotone_casl(wght: f32) -> f32 {
    if wght < 500.0 { 0.0 } else { 1.0 }
}
