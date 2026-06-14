//! Tests for `apply_style`: name IDs 1/2/4/6/16/17, OS/2 fsSelection/usWeightClass,
//! and head macStyle.

use read_fonts::{FontRef, TableProvider};
use warpnine_font_ops::{StyleBits, StyleNames, apply_style};

// fsSelection bits
const FS_ITALIC: u16 = 1 << 0;
const FS_BOLD: u16 = 1 << 5;
const FS_REGULAR: u16 = 1 << 6;
// macStyle bits
const MAC_BOLD: u16 = 1 << 0;
const MAC_ITALIC: u16 = 1 << 1;

const FIXTURE: &[u8] = include_bytes!("fixtures/OpenSans-Bold.subset.ttf");

fn name_string(data: &[u8], name_id: u16) -> Option<String> {
    let font = FontRef::new(data).unwrap();
    let name = font.name().unwrap();
    for record in name.name_record() {
        if record.name_id().to_u16() == name_id {
            return record.string(name.string_data()).ok().map(|s| s.chars().collect());
        }
    }
    None
}

fn name_on_platform(data: &[u8], name_id: u16, plat: u16, enc: u16, lang: u16) -> Option<String> {
    let font = FontRef::new(data).unwrap();
    let name = font.name().unwrap();
    for record in name.name_record() {
        if record.name_id().to_u16() == name_id
            && record.platform_id() == plat
            && record.encoding_id() == enc
            && record.language_id() == lang
        {
            return record.string(name.string_data()).ok().map(|s| s.chars().collect());
        }
    }
    None
}

fn fs_selection(data: &[u8]) -> u16 {
    FontRef::new(data).unwrap().os2().unwrap().fs_selection().bits()
}

fn mac_style(data: &[u8]) -> u16 {
    FontRef::new(data).unwrap().head().unwrap().mac_style().bits()
}

fn weight_class(data: &[u8]) -> u16 {
    FontRef::new(data).unwrap().os2().unwrap().us_weight_class()
}

fn names(subfamily: &str) -> StyleNames {
    StyleNames {
        family: "Test Family".into(),
        subfamily: subfamily.into(),
        full_name: format!("Test Family {subfamily}"),
        postscript: format!("TestFamily-{}", subfamily.replace(' ', "")),
        typo_family: "Test Family".into(),
        typo_subfamily: subfamily.into(),
    }
}

#[test]
fn applies_names() {
    let out = apply_style(
        FIXTURE,
        &names("Bold Italic"),
        &StyleBits {
            italic: true,
            bold: true,
            regular: false,
            weight_class: 700,
        },
    )
    .unwrap();

    assert_eq!(name_string(&out, 1).as_deref(), Some("Test Family"));
    assert_eq!(name_string(&out, 2).as_deref(), Some("Bold Italic"));
    assert_eq!(name_string(&out, 4).as_deref(), Some("Test Family Bold Italic"));
    assert_eq!(name_string(&out, 6).as_deref(), Some("TestFamily-BoldItalic"));
}

#[test]
fn adds_missing_typographic_names() {
    // The OpenSans fixture has only name IDs 0-6 (no 16/17). apply_style must
    // synthesize them so typographic grouping works regardless of the donor.
    let mut names = names("Bold Italic");
    names.typo_family = "Type Family".into();
    names.typo_subfamily = "SemiBold Italic".into();

    let out = apply_style(
        FIXTURE,
        &names,
        &StyleBits {
            italic: true,
            bold: false,
            regular: false,
            weight_class: 600,
        },
    )
    .unwrap();

    assert_eq!(name_string(&out, 16).as_deref(), Some("Type Family"));
    assert_eq!(name_string(&out, 17).as_deref(), Some("SemiBold Italic"));
    // The synthesized records must sit on the same platform as ID 1.
    assert_eq!(name_on_platform(&out, 16, 3, 1, 0x409).as_deref(), Some("Type Family"));
    assert_eq!(name_on_platform(&out, 17, 3, 1, 0x409).as_deref(), Some("SemiBold Italic"));
}

#[test]
fn bold_italic_sets_bits() {
    let out = apply_style(
        FIXTURE,
        &names("Bold Italic"),
        &StyleBits {
            italic: true,
            bold: true,
            regular: false,
            weight_class: 700,
        },
    )
    .unwrap();

    // Fixture starts at fsSelection=0x20 (BOLD), macStyle=0x01 (BOLD); only the
    // bold/italic/regular bits change, so the exact values double as a
    // preservation check.
    assert_eq!(fs_selection(&out), FS_ITALIC | FS_BOLD);
    assert_eq!(mac_style(&out), MAC_BOLD | MAC_ITALIC);
    assert_eq!(weight_class(&out), 700);
}

#[test]
fn regular_clears_bold() {
    let out = apply_style(
        FIXTURE,
        &names("Regular"),
        &StyleBits {
            italic: false,
            bold: false,
            regular: true,
            weight_class: 400,
        },
    )
    .unwrap();

    assert_eq!(fs_selection(&out), FS_REGULAR);
    assert_eq!(mac_style(&out), 0);
    assert_eq!(weight_class(&out), 400);
}
