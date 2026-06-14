//! Style and design type definitions.

use font_instancer::AxisLocation;
use warpnine_font_ops::{StyleBits, StyleNames, style_display_name};

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
    /// Returns the weight value as f32.
    pub const fn value(&self) -> f32 {
        self.0
    }

    /// Converts to OS/2 usWeightClass (u16).
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
        style_display_name(self.name)
    }

    /// RIBBI-grouped name-table strings for this style.
    ///
    /// Thin wrapper over [`warpnine_font_ops::ribbi_names`]; the derivation is
    /// brand-agnostic so other projects can reuse it directly.
    pub fn ribbi_names(&self, family: &str, ps_family: &str) -> StyleNames {
        warpnine_font_ops::ribbi_names(
            family,
            ps_family,
            self.name,
            self.weight.value() as u16,
            self.slant.is_italic(),
        )
    }

    /// OS/2 / head style bits for this style.
    pub fn style_bits(&self) -> StyleBits {
        warpnine_font_ops::style_bits(self.weight.value() as u16, self.slant.is_italic())
    }

    pub fn axis_locations(&self, mono: f32, casl: f32) -> [AxisLocation; 5] {
        [
            AxisLocation::new("MONO", mono),
            AxisLocation::new("CASL", casl),
            AxisLocation::new("wght", self.weight.value()),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn s(name: &'static str, weight: f32, slant: Slant) -> Style {
        Style::new(name, Weight(weight), slant)
    }

    #[test]
    fn regular_uses_base_family() {
        let n = s("Regular", 400.0, Slant::Upright).ribbi_names("Warpnine Mono", "WarpnineMono");
        assert_eq!(n.family, "Warpnine Mono");
        assert_eq!(n.subfamily, "Regular");
        assert_eq!(n.full_name, "Warpnine Mono");
        assert_eq!(n.postscript, "WarpnineMono-Regular");
        assert_eq!(n.typo_family, "Warpnine Mono");
        assert_eq!(n.typo_subfamily, "Regular");
    }

    #[test]
    fn italic_400_in_base_quad() {
        let n = s("Italic", 400.0, Slant::Italic).ribbi_names("Warpnine Mono", "WarpnineMono");
        assert_eq!(n.family, "Warpnine Mono");
        assert_eq!(n.subfamily, "Italic");
        assert_eq!(n.full_name, "Warpnine Mono Italic");
        assert_eq!(n.typo_subfamily, "Italic");
    }

    #[test]
    fn bold_in_base_quad() {
        let n = s("Bold", 700.0, Slant::Upright).ribbi_names("Warpnine Mono", "WarpnineMono");
        assert_eq!(n.family, "Warpnine Mono");
        assert_eq!(n.subfamily, "Bold");
        assert_eq!(n.full_name, "Warpnine Mono Bold");
        assert_eq!(n.typo_subfamily, "Bold");
    }

    #[test]
    fn bold_italic_in_base_quad() {
        let n = s("BoldItalic", 700.0, Slant::Italic).ribbi_names("Warpnine Mono", "WarpnineMono");
        assert_eq!(n.family, "Warpnine Mono");
        assert_eq!(n.subfamily, "Bold Italic");
        assert_eq!(n.full_name, "Warpnine Mono Bold Italic");
        assert_eq!(n.postscript, "WarpnineMono-BoldItalic");
        assert_eq!(n.typo_subfamily, "Bold Italic");
    }

    #[test]
    fn semibold_is_own_subfamily() {
        let n = s("SemiBold", 600.0, Slant::Upright).ribbi_names("Warpnine Mono", "WarpnineMono");
        assert_eq!(n.family, "Warpnine Mono SemiBold");
        assert_eq!(n.subfamily, "Regular");
        assert_eq!(n.full_name, "Warpnine Mono SemiBold");
        assert_eq!(n.typo_family, "Warpnine Mono");
        assert_eq!(n.typo_subfamily, "SemiBold");
    }

    #[test]
    fn semibold_italic_is_own_subfamily() {
        let n =
            s("SemiBoldItalic", 600.0, Slant::Italic).ribbi_names("Warpnine Mono", "WarpnineMono");
        assert_eq!(n.family, "Warpnine Mono SemiBold");
        assert_eq!(n.subfamily, "Italic");
        assert_eq!(n.full_name, "Warpnine Mono SemiBold Italic");
        assert_eq!(n.typo_subfamily, "SemiBold Italic");
    }

    #[test]
    fn extrablack_italic_names() {
        let n = s("ExtraBlackItalic", 1000.0, Slant::Italic)
            .ribbi_names("Warpnine Mono", "WarpnineMono");
        assert_eq!(n.family, "Warpnine Mono ExtraBlack");
        assert_eq!(n.subfamily, "Italic");
        assert_eq!(n.typo_subfamily, "ExtraBlack Italic");
        assert_eq!(n.postscript, "WarpnineMono-ExtraBlackItalic");
    }

    #[test]
    fn condensed_base_family_carries_spaces() {
        let n = s("SemiBold", 600.0, Slant::Upright)
            .ribbi_names("Warpnine Sans Condensed", "WarpnineSansCondensed");
        assert_eq!(n.family, "Warpnine Sans Condensed SemiBold");
        assert_eq!(n.postscript, "WarpnineSansCondensed-SemiBold");
    }

    #[test]
    fn bits_regular() {
        let b = s("Regular", 400.0, Slant::Upright).style_bits();
        assert!(b.regular && !b.bold && !b.italic);
        assert_eq!(b.weight_class, 400);
    }

    #[test]
    fn bits_bold() {
        let b = s("Bold", 700.0, Slant::Upright).style_bits();
        assert!(b.bold && !b.regular && !b.italic);
        assert_eq!(b.weight_class, 700);
    }

    #[test]
    fn bits_bold_italic() {
        let b = s("BoldItalic", 700.0, Slant::Italic).style_bits();
        assert!(b.bold && b.italic && !b.regular);
    }

    #[test]
    fn bits_heavy_upright_is_regular() {
        let b = s("Black", 900.0, Slant::Upright).style_bits();
        assert!(b.regular && !b.bold && !b.italic);
        assert_eq!(b.weight_class, 900);
    }

    #[test]
    fn bits_semibold_italic() {
        let b = s("SemiBoldItalic", 600.0, Slant::Italic).style_bits();
        assert!(b.italic && !b.bold && !b.regular);
    }
}
