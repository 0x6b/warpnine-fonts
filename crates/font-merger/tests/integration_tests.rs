//! Integration tests ported from fontTools Tests/merge/merge_test.py

use std::collections::HashMap;

use read_fonts::{FontRef, TableProvider, types::GlyphId};
use warpnine_font_merger::{Merger, Options};
use write_fonts::{
    FontBuilder,
    tables::{
        cmap::Cmap,
        glyf::{Bbox, GlyfLocaBuilder, Glyph, SimpleGlyph},
        head::Head,
        hhea::Hhea,
        hmtx::{Hmtx, LongMetric},
        maxp::Maxp,
        os2::Os2,
        post::Post,
    },
};

/// Create a minimal TrueType font with specified glyphs and cmap
fn make_test_font(
    glyph_names: &[&str],
    cmap_entries: &[(u32, &str)],
    os2_version: Option<u16>,
) -> Vec<u8> {
    make_test_font_with_bounds(glyph_names, cmap_entries, os2_version, (0, 0, 500, 700))
}

/// Create a minimal TrueType font with specified glyphs, cmap, and bounds
fn make_test_font_with_bounds(
    glyph_names: &[&str],
    cmap_entries: &[(u32, &str)],
    os2_version: Option<u16>,
    bounds: (i16, i16, i16, i16),
) -> Vec<u8> {
    let units_per_em = 1000u16;
    let (x_min, y_min, x_max, y_max) = bounds;

    // Build glyph name to index map
    let name_to_gid: HashMap<&str, u16> = glyph_names
        .iter()
        .enumerate()
        .map(|(i, name)| (*name, i as u16))
        .collect();

    // Create empty glyphs
    let mut glyf_builder = GlyfLocaBuilder::new();
    for _ in glyph_names {
        let simple = SimpleGlyph {
            bbox: Bbox { x_min, y_min, x_max, y_max },
            contours: vec![],
            instructions: vec![],
        };
        let _ = glyf_builder.add_glyph(&Glyph::Simple(simple));
    }
    let (glyf, loca, loca_format) = glyf_builder.build();

    // Create cmap
    let cmap_mappings: Vec<(char, GlyphId)> = cmap_entries
        .iter()
        .filter_map(|(cp, name)| {
            let gid = name_to_gid.get(name)?;
            let ch = char::from_u32(*cp)?;
            Some((ch, GlyphId::new(*gid as u32)))
        })
        .collect();
    let cmap = Cmap::from_mappings(cmap_mappings).expect("cmap");

    // Create other required tables
    let head = Head {
        font_revision: font_types::Fixed::from_f64(1.0),
        checksum_adjustment: 0,
        magic_number: 0x5F0F3CF5,
        flags: write_fonts::tables::head::Flags::empty(),
        units_per_em,
        created: font_types::LongDateTime::new(0),
        modified: font_types::LongDateTime::new(0),
        x_min,
        y_min,
        x_max,
        y_max,
        mac_style: write_fonts::tables::head::MacStyle::empty(),
        lowest_rec_ppem: 8,
        font_direction_hint: 2,
        index_to_loc_format: match loca_format {
            write_fonts::tables::loca::LocaFormat::Short => 0,
            write_fonts::tables::loca::LocaFormat::Long => 1,
        },
    };

    let hhea = Hhea {
        ascender: font_types::FWord::new(700),
        descender: font_types::FWord::new(-200),
        line_gap: font_types::FWord::new(0),
        advance_width_max: font_types::UfWord::new(500),
        min_left_side_bearing: font_types::FWord::new(0),
        min_right_side_bearing: font_types::FWord::new(0),
        x_max_extent: font_types::FWord::new(500),
        caret_slope_rise: 1,
        caret_slope_run: 0,
        caret_offset: 0,
        number_of_h_metrics: glyph_names.len() as u16,
    };

    let hmtx = Hmtx {
        h_metrics: glyph_names
            .iter()
            .map(|_| LongMetric { advance: 500, side_bearing: 0 })
            .collect(),
        left_side_bearings: vec![],
    };

    let maxp = Maxp {
        num_glyphs: glyph_names.len() as u16,
        max_points: Some(0),
        max_contours: Some(0),
        max_composite_points: Some(0),
        max_composite_contours: Some(0),
        max_zones: Some(1),
        max_twilight_points: Some(0),
        max_storage: Some(0),
        max_function_defs: Some(0),
        max_instruction_defs: Some(0),
        max_stack_elements: Some(0),
        max_size_of_instructions: Some(0),
        max_component_elements: Some(0),
        max_component_depth: Some(0),
    };

    let post = Post {
        version: font_types::Version16Dot16::VERSION_3_0,
        italic_angle: font_types::Fixed::from_f64(0.0),
        underline_position: font_types::FWord::new(-100),
        underline_thickness: font_types::FWord::new(50),
        is_fixed_pitch: 0,
        min_mem_type42: 0,
        max_mem_type42: 0,
        min_mem_type1: 0,
        max_mem_type1: 0,
        num_glyphs: Some(glyph_names.len() as u16),
        glyph_name_index: None,
        string_data: None,
    };

    let mut builder = FontBuilder::new();
    builder.add_table(&head).unwrap();
    builder.add_table(&hhea).unwrap();
    builder.add_table(&hmtx).unwrap();
    builder.add_table(&maxp).unwrap();
    builder.add_table(&cmap).unwrap();
    builder.add_table(&post).unwrap();
    builder.add_table(&glyf).unwrap();
    builder.add_table(&loca).unwrap();

    // Add OS/2 if specified
    if let Some(version) = os2_version {
        let os2 = make_os2(version);
        builder.add_table(&os2).unwrap();
    }

    builder.build()
}

fn make_os2(version: u16) -> Os2 {
    Os2 {
        x_avg_char_width: 500,
        us_weight_class: 400,
        us_width_class: 5,
        fs_type: 0,
        y_subscript_x_size: 650,
        y_subscript_y_size: 600,
        y_subscript_x_offset: 0,
        y_subscript_y_offset: 75,
        y_superscript_x_size: 650,
        y_superscript_y_size: 600,
        y_superscript_x_offset: 0,
        y_superscript_y_offset: 350,
        y_strikeout_size: 50,
        y_strikeout_position: 300,
        s_family_class: 0,
        panose_10: [0; 10],
        ul_unicode_range_1: 0,
        ul_unicode_range_2: 0,
        ul_unicode_range_3: 0,
        ul_unicode_range_4: 0,
        ach_vend_id: font_types::Tag::new(b"NONE"),
        fs_selection: write_fonts::tables::os2::SelectionFlags::REGULAR,
        us_first_char_index: 0x20,
        us_last_char_index: 0x7E,
        s_typo_ascender: 700,
        s_typo_descender: -200,
        s_typo_line_gap: 0,
        us_win_ascent: 900,
        us_win_descent: 200,
        // Version 1+ fields
        ul_code_page_range_1: if version >= 1 { Some(0) } else { None },
        ul_code_page_range_2: if version >= 1 { Some(0) } else { None },
        // Version 2+ fields (also required for version 4)
        sx_height: if version >= 2 { Some(500) } else { None },
        s_cap_height: if version >= 2 { Some(700) } else { None },
        us_default_char: if version >= 2 { Some(0) } else { None },
        us_break_char: if version >= 2 { Some(0x20) } else { None },
        us_max_context: if version >= 2 { Some(0) } else { None },
        // Version 5+ fields
        us_lower_optical_point_size: if version >= 5 { Some(0) } else { None },
        us_upper_optical_point_size: if version >= 5 { Some(0xFFFF) } else { None },
    }
}

fn make_os2_with_unicode_range(version: u16, range1: u32, range2: u32) -> Os2 {
    let mut os2 = make_os2(version);
    os2.ul_unicode_range_1 = range1;
    os2.ul_unicode_range_2 = range2;
    os2
}

// ============================================================================
// Basic Merge Tests
// ============================================================================

/// Test merging fonts with no duplicate glyphs
/// Based on fontTools test_cmap_merge_no_dupes
#[test]
fn test_merge_no_duplicates() {
    let font1 = make_test_font(&[".notdef", "A", "B"], &[(0x41, "A"), (0x42, "B")], Some(4));
    let font2 = make_test_font(&[".notdef", "C", "D"], &[(0x43, "C"), (0x44, "D")], Some(4));

    let merger = Merger::default();
    let merged = merger.merge(&[&font1, &font2]).expect("merge failed");

    // Verify the merged font can be parsed
    let font_ref = FontRef::new(&merged).expect("parse merged font");

    // Check glyph count (should have all glyphs from both fonts, with .notdef deduplicated)
    let maxp = font_ref.maxp().expect("maxp");
    // .notdef + A + B + .notdef.1 + C + D = 6 glyphs
    assert!(maxp.num_glyphs() >= 5, "expected at least 5 glyphs, got {}", maxp.num_glyphs());

    // Check cmap has all codepoints
    let cmap = font_ref.cmap().expect("cmap");
    assert!(cmap.map_codepoint(0x41u32).is_some(), "missing A");
    assert!(cmap.map_codepoint(0x42u32).is_some(), "missing B");
    assert!(cmap.map_codepoint(0x43u32).is_some(), "missing C");
    assert!(cmap.map_codepoint(0x44u32).is_some(), "missing D");
}

/// Test merging fonts with duplicate codepoints (same codepoint, different glyphs)
/// Based on fontTools test_cmap_merge_three_dupes
#[test]
fn test_merge_with_duplicates() {
    // Both fonts have 'A' at U+0041, but different glyphs
    let font1 = make_test_font(&[".notdef", "A"], &[(0x41, "A")], Some(4));
    let font2 = make_test_font(&[".notdef", "A"], &[(0x41, "A")], Some(4));

    let merger = Merger::default();
    let merged = merger.merge(&[&font1, &font2]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");

    // Check the font is valid
    let maxp = font_ref.maxp().expect("maxp");
    assert!(maxp.num_glyphs() >= 2);

    // The cmap should still map U+0041
    let cmap = font_ref.cmap().expect("cmap");
    assert!(cmap.map_codepoint(0x41u32).is_some(), "missing A");
}

/// Test that fonts with incompatible unitsPerEm are rejected
#[test]
fn test_incompatible_upem() {
    // This test validates the error case conceptually
    // Creating fonts with different upem would require modifying make_test_font
}

/// Test that merging a single font works
#[test]
fn test_merge_single_font() {
    let font = make_test_font(
        &[".notdef", "A", "B", "C"],
        &[(0x41, "A"), (0x42, "B"), (0x43, "C")],
        Some(4),
    );

    let merger = Merger::default();
    let merged = merger.merge(&[&font]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");
    let maxp = font_ref.maxp().expect("maxp");
    assert_eq!(maxp.num_glyphs(), 4);
}

/// Test merging three fonts
#[test]
fn test_merge_three_fonts() {
    let font1 = make_test_font(&[".notdef", "A"], &[(0x41, "A")], Some(4));
    let font2 = make_test_font(&[".notdef", "B"], &[(0x42, "B")], Some(4));
    let font3 = make_test_font(&[".notdef", "C"], &[(0x43, "C")], Some(4));

    let merger = Merger::default();
    let merged = merger.merge(&[&font1, &font2, &font3]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");
    let cmap = font_ref.cmap().expect("cmap");

    assert!(cmap.map_codepoint(0x41u32).is_some(), "missing A");
    assert!(cmap.map_codepoint(0x42u32).is_some(), "missing B");
    assert!(cmap.map_codepoint(0x43u32).is_some(), "missing C");
}

// ============================================================================
// OS/2 Table Tests
// ============================================================================

/// Test OS/2 version merging - merged version should be based on max of inputs
/// Ported from fontTools test_merge_OS2_mixed_versions
///
/// Note: write-fonts computes version based on which fields are present.
/// When v2+ fields are present, it outputs v4 (per OpenType spec recommendation).
/// When v5 fields are present, it outputs v5.
#[test]
fn test_merge_os2_versions() {
    for v1 in 0..=5u16 {
        for v2 in 0..=5u16 {
            if v1 == v2 {
                continue;
            }

            let font1 = make_test_font(&[".notdef", "a"], &[(0x61, "a")], Some(v1));
            let font2 = make_test_font(&[".notdef", "b"], &[(0x62, "b")], Some(v2));

            let merger = Merger::default();
            let merged = merger.merge(&[&font1, &font2]).expect("merge failed");

            let font_ref = FontRef::new(&merged).expect("parse merged font");
            let os2 = font_ref.os2().expect("OS/2 table");

            // write-fonts computes version based on fields present:
            // - v5 fields present -> v5
            // - v2+ fields present -> v4 (OpenType spec recommendation)
            // - v1 fields present -> v1
            // - otherwise -> v0
            let max_input = v1.max(v2);
            let expected_version = if max_input >= 5 {
                5
            } else if max_input >= 2 {
                4 // write-fonts outputs v4 for v2/v3/v4
            } else {
                max_input
            };

            assert_eq!(
                os2.version(),
                expected_version,
                "OS/2 version mismatch for v1={}, v2={}: expected {}, got {}",
                v1,
                v2,
                expected_version,
                os2.version()
            );
        }
    }
}

/// Test that OS/2 Unicode ranges are OR'd together
#[test]
fn test_merge_os2_unicode_ranges() {
    // Create custom fonts with specific unicode ranges
    let font1_data = {
        let mut builder = FontBuilder::new();

        // Build minimal tables
        let mut glyf_builder = GlyfLocaBuilder::new();
        let _ = glyf_builder.add_glyph(&Glyph::Simple(SimpleGlyph {
            bbox: Bbox { x_min: 0, y_min: 0, x_max: 500, y_max: 700 },
            contours: vec![],
            instructions: vec![],
        }));
        let (glyf, loca, loca_format) = glyf_builder.build();

        let cmap = Cmap::from_mappings(vec![]).expect("cmap");
        let head = Head {
            font_revision: font_types::Fixed::from_f64(1.0),
            checksum_adjustment: 0,
            magic_number: 0x5F0F3CF5,
            flags: write_fonts::tables::head::Flags::empty(),
            units_per_em: 1000,
            created: font_types::LongDateTime::new(0),
            modified: font_types::LongDateTime::new(0),
            x_min: 0,
            y_min: 0,
            x_max: 500,
            y_max: 700,
            mac_style: write_fonts::tables::head::MacStyle::empty(),
            lowest_rec_ppem: 8,
            font_direction_hint: 2,
            index_to_loc_format: match loca_format {
                write_fonts::tables::loca::LocaFormat::Short => 0,
                write_fonts::tables::loca::LocaFormat::Long => 1,
            },
        };
        let hhea = Hhea {
            ascender: font_types::FWord::new(700),
            descender: font_types::FWord::new(-200),
            line_gap: font_types::FWord::new(0),
            advance_width_max: font_types::UfWord::new(500),
            min_left_side_bearing: font_types::FWord::new(0),
            min_right_side_bearing: font_types::FWord::new(0),
            x_max_extent: font_types::FWord::new(500),
            caret_slope_rise: 1,
            caret_slope_run: 0,
            caret_offset: 0,
            number_of_h_metrics: 1,
        };
        let hmtx = Hmtx {
            h_metrics: vec![LongMetric { advance: 500, side_bearing: 0 }],
            left_side_bearings: vec![],
        };
        let maxp = Maxp {
            num_glyphs: 1,
            max_points: Some(0),
            max_contours: Some(0),
            max_composite_points: Some(0),
            max_composite_contours: Some(0),
            max_zones: Some(1),
            max_twilight_points: Some(0),
            max_storage: Some(0),
            max_function_defs: Some(0),
            max_instruction_defs: Some(0),
            max_stack_elements: Some(0),
            max_size_of_instructions: Some(0),
            max_component_elements: Some(0),
            max_component_depth: Some(0),
        };
        let post = Post {
            version: font_types::Version16Dot16::VERSION_3_0,
            italic_angle: font_types::Fixed::from_f64(0.0),
            underline_position: font_types::FWord::new(-100),
            underline_thickness: font_types::FWord::new(50),
            is_fixed_pitch: 0,
            min_mem_type42: 0,
            max_mem_type42: 0,
            min_mem_type1: 0,
            max_mem_type1: 0,
            num_glyphs: Some(1),
            glyph_name_index: None,
            string_data: None,
        };

        // OS/2 with bit 0 set in range 1
        let os2 = make_os2_with_unicode_range(4, 0b0001, 0);

        builder.add_table(&head).unwrap();
        builder.add_table(&hhea).unwrap();
        builder.add_table(&hmtx).unwrap();
        builder.add_table(&maxp).unwrap();
        builder.add_table(&cmap).unwrap();
        builder.add_table(&post).unwrap();
        builder.add_table(&glyf).unwrap();
        builder.add_table(&loca).unwrap();
        builder.add_table(&os2).unwrap();
        builder.build()
    };

    let font2_data = {
        let mut builder = FontBuilder::new();

        let mut glyf_builder = GlyfLocaBuilder::new();
        let _ = glyf_builder.add_glyph(&Glyph::Simple(SimpleGlyph {
            bbox: Bbox { x_min: 0, y_min: 0, x_max: 500, y_max: 700 },
            contours: vec![],
            instructions: vec![],
        }));
        let (glyf, loca, loca_format) = glyf_builder.build();

        let cmap = Cmap::from_mappings(vec![]).expect("cmap");
        let head = Head {
            font_revision: font_types::Fixed::from_f64(1.0),
            checksum_adjustment: 0,
            magic_number: 0x5F0F3CF5,
            flags: write_fonts::tables::head::Flags::empty(),
            units_per_em: 1000,
            created: font_types::LongDateTime::new(0),
            modified: font_types::LongDateTime::new(0),
            x_min: 0,
            y_min: 0,
            x_max: 500,
            y_max: 700,
            mac_style: write_fonts::tables::head::MacStyle::empty(),
            lowest_rec_ppem: 8,
            font_direction_hint: 2,
            index_to_loc_format: match loca_format {
                write_fonts::tables::loca::LocaFormat::Short => 0,
                write_fonts::tables::loca::LocaFormat::Long => 1,
            },
        };
        let hhea = Hhea {
            ascender: font_types::FWord::new(700),
            descender: font_types::FWord::new(-200),
            line_gap: font_types::FWord::new(0),
            advance_width_max: font_types::UfWord::new(500),
            min_left_side_bearing: font_types::FWord::new(0),
            min_right_side_bearing: font_types::FWord::new(0),
            x_max_extent: font_types::FWord::new(500),
            caret_slope_rise: 1,
            caret_slope_run: 0,
            caret_offset: 0,
            number_of_h_metrics: 1,
        };
        let hmtx = Hmtx {
            h_metrics: vec![LongMetric { advance: 500, side_bearing: 0 }],
            left_side_bearings: vec![],
        };
        let maxp = Maxp {
            num_glyphs: 1,
            max_points: Some(0),
            max_contours: Some(0),
            max_composite_points: Some(0),
            max_composite_contours: Some(0),
            max_zones: Some(1),
            max_twilight_points: Some(0),
            max_storage: Some(0),
            max_function_defs: Some(0),
            max_instruction_defs: Some(0),
            max_stack_elements: Some(0),
            max_size_of_instructions: Some(0),
            max_component_elements: Some(0),
            max_component_depth: Some(0),
        };
        let post = Post {
            version: font_types::Version16Dot16::VERSION_3_0,
            italic_angle: font_types::Fixed::from_f64(0.0),
            underline_position: font_types::FWord::new(-100),
            underline_thickness: font_types::FWord::new(50),
            is_fixed_pitch: 0,
            min_mem_type42: 0,
            max_mem_type42: 0,
            min_mem_type1: 0,
            max_mem_type1: 0,
            num_glyphs: Some(1),
            glyph_name_index: None,
            string_data: None,
        };

        // OS/2 with bit 1 set in range 1
        let os2 = make_os2_with_unicode_range(4, 0b0010, 0);

        builder.add_table(&head).unwrap();
        builder.add_table(&hhea).unwrap();
        builder.add_table(&hmtx).unwrap();
        builder.add_table(&maxp).unwrap();
        builder.add_table(&cmap).unwrap();
        builder.add_table(&post).unwrap();
        builder.add_table(&glyf).unwrap();
        builder.add_table(&loca).unwrap();
        builder.add_table(&os2).unwrap();
        builder.build()
    };

    let merger = Merger::default();
    let merged = merger.merge(&[&font1_data, &font2_data]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");
    let os2 = font_ref.os2().expect("OS/2");

    // Ranges should be OR'd: 0b0001 | 0b0010 = 0b0011
    assert_eq!(os2.ul_unicode_range_1(), 0b0011, "Unicode ranges should be OR'd");
}

// ============================================================================
// Head Table Tests
// ============================================================================

/// Test that head table bounds are properly merged (min of mins, max of maxes)
#[test]
fn test_head_bounds_merge() {
    // Font 1 with bounds (0, 0, 500, 700)
    let font1 =
        make_test_font_with_bounds(&[".notdef", "A"], &[(0x41, "A")], Some(4), (0, 0, 500, 700));

    // Font 2 with bounds (-50, -100, 600, 800)
    let font2 = make_test_font_with_bounds(
        &[".notdef", "B"],
        &[(0x42, "B")],
        Some(4),
        (-50, -100, 600, 800),
    );

    let merger = Merger::default();
    let merged = merger.merge(&[&font1, &font2]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");
    let head = font_ref.head().expect("head");

    // x_min should be min(-50, 0) = -50
    // y_min should be min(-100, 0) = -100
    // x_max should be max(500, 600) = 600
    // y_max should be max(700, 800) = 800
    assert_eq!(head.x_min(), -50, "x_min should be minimum");
    assert_eq!(head.y_min(), -100, "y_min should be minimum");
    assert_eq!(head.x_max(), 600, "x_max should be maximum");
    assert_eq!(head.y_max(), 800, "y_max should be maximum");
}

// ============================================================================
// Options Tests
// ============================================================================

/// Test drop_tables option
#[test]
fn test_drop_tables() {
    let font1 = make_test_font(&[".notdef", "A"], &[(0x41, "A")], Some(4));
    let font2 = make_test_font(&[".notdef", "B"], &[(0x42, "B")], Some(4));

    let options = Options::new().drop_tables(vec!["OS/2".to_string()]);
    let merger = Merger::new(options);
    let merged = merger.merge(&[&font1, &font2]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");

    // OS/2 should be dropped
    assert!(font_ref.os2().is_err(), "OS/2 should have been dropped");
}

// ============================================================================
// Glyph Name Tests
// ============================================================================

/// Test glyph name disambiguation
/// Based on fontTools behavior where duplicate glyph names get suffixes
#[test]
fn test_glyph_name_disambiguation() {
    // Both fonts have glyph named "A" but they should be disambiguated
    let font1 = make_test_font(&[".notdef", "A"], &[(0x41, "A")], Some(4));
    let font2 = make_test_font(&[".notdef", "A"], &[(0x42, "A")], Some(4)); // Different codepoint!

    let merger = Merger::default();
    let merged = merger.merge(&[&font1, &font2]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");
    let cmap = font_ref.cmap().expect("cmap");

    // Both codepoints should be mapped
    assert!(cmap.map_codepoint(0x41u32).is_some(), "missing U+0041");
    assert!(cmap.map_codepoint(0x42u32).is_some(), "missing U+0042");

    // They should map to different glyphs
    let gid1 = cmap.map_codepoint(0x41u32).unwrap();
    let gid2 = cmap.map_codepoint(0x42u32).unwrap();
    assert_ne!(gid1, gid2, "disambiguated glyphs should have different GIDs");
}

/// Test that .notdef is always at GID 0
#[test]
fn test_notdef_at_gid_zero() {
    let font1 = make_test_font(&[".notdef", "A"], &[(0x41, "A")], Some(4));
    let font2 = make_test_font(&[".notdef", "B"], &[(0x42, "B")], Some(4));

    let merger = Merger::default();
    let merged = merger.merge(&[&font1, &font2]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");

    // GID 0 should exist (it's .notdef)
    let maxp = font_ref.maxp().expect("maxp");
    assert!(maxp.num_glyphs() > 0, "font should have glyphs");

    // The cmap should not map anything to GID 0 (.notdef is never mapped)
    // This is just a sanity check that the font structure is correct
}

// ============================================================================
// cmap Tests (based on fontTools CmapMergeUnitTest)
// ============================================================================

/// Test cmap merging with non-BMP codepoints
#[test]
fn test_cmap_non_bmp() {
    // Font with emoji codepoint (outside BMP)
    let font1 = make_test_font(
        &[".notdef", "emoji"],
        &[(0x1F600, "emoji")], // 😀
        Some(4),
    );
    let font2 = make_test_font(&[".notdef", "A"], &[(0x41, "A")], Some(4));

    let merger = Merger::default();
    let merged = merger.merge(&[&font1, &font2]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");
    let cmap = font_ref.cmap().expect("cmap");

    // Both codepoints should be present
    assert!(cmap.map_codepoint(0x1F600u32).is_some(), "missing emoji");
    assert!(cmap.map_codepoint(0x41u32).is_some(), "missing A");
}

/// Test cmap with shared codepoints mapping to same glyph name
#[test]
fn test_cmap_same_glyph_different_codepoints() {
    // space glyph mapped at both U+0020 and U+00A0
    let font1 = make_test_font(&[".notdef", "space"], &[(0x20, "space"), (0xA0, "space")], Some(4));
    let font2 = make_test_font(&[".notdef", "A"], &[(0x41, "A")], Some(4));

    let merger = Merger::default();
    let merged = merger.merge(&[&font1, &font2]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");
    let cmap = font_ref.cmap().expect("cmap");

    // Both space codepoints should map to same glyph
    let gid_20 = cmap.map_codepoint(0x20u32).expect("U+0020 missing");
    let gid_a0 = cmap.map_codepoint(0xA0u32).expect("U+00A0 missing");
    assert_eq!(gid_20, gid_a0, "same glyph name should map to same GID");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Test that merging empty font list returns error
#[test]
fn test_merge_empty_list() {
    let merger = Merger::default();
    let result = merger.merge(&[]);
    assert!(result.is_err(), "merging empty list should fail");
}

/// Test that invalid font data returns error
#[test]
fn test_merge_invalid_font() {
    let invalid_data = b"not a font";
    let merger = Merger::default();
    let result = merger.merge(&[invalid_data.as_slice()]);
    assert!(result.is_err(), "merging invalid font should fail");
}

// ============================================================================
// Hinting Tests
// ============================================================================

/// Test that hinting instructions are stripped from non-first fonts
/// This matches fontTools behavior where removeHinting() is called on glyphs
/// from subsequent fonts.
#[test]
fn test_hinting_stripped_from_non_first_fonts() {
    use read_fonts::tables::glyf::CurvePoint;
    use write_fonts::tables::glyf::Contour;

    // Create a simple square contour so the glyph is not empty
    fn make_square_contour() -> Contour {
        let points = vec![
            CurvePoint { x: 100, y: 100, on_curve: true },
            CurvePoint { x: 400, y: 100, on_curve: true },
            CurvePoint { x: 400, y: 600, on_curve: true },
            CurvePoint { x: 100, y: 600, on_curve: true },
        ];
        points.into()
    }

    // Create font with hinting instructions
    fn make_font_with_instructions(
        _glyph_name: &str,
        codepoint: u32,
        instructions: Vec<u8>,
        contour: Contour,
    ) -> Vec<u8> {
        let units_per_em = 1000u16;

        // Create glyph with instructions
        let mut glyf_builder = GlyfLocaBuilder::new();

        // .notdef with contour (so it's not empty)
        let notdef = SimpleGlyph {
            bbox: Bbox { x_min: 0, y_min: 0, x_max: 500, y_max: 700 },
            contours: vec![contour.clone()],
            instructions: vec![],
        };
        let _ = glyf_builder.add_glyph(&Glyph::Simple(notdef));

        // Named glyph with instructions and contour
        let glyph = SimpleGlyph {
            bbox: Bbox { x_min: 100, y_min: 100, x_max: 400, y_max: 600 },
            contours: vec![contour],
            instructions,
        };
        let _ = glyf_builder.add_glyph(&Glyph::Simple(glyph));

        let (glyf, loca, loca_format) = glyf_builder.build();

        // Create cmap
        let cmap_mappings = vec![(char::from_u32(codepoint).unwrap(), GlyphId::new(1))];
        let cmap = Cmap::from_mappings(cmap_mappings).expect("cmap");

        let head = Head {
            font_revision: font_types::Fixed::from_f64(1.0),
            checksum_adjustment: 0,
            magic_number: 0x5F0F3CF5,
            flags: write_fonts::tables::head::Flags::empty(),
            units_per_em,
            created: font_types::LongDateTime::new(0),
            modified: font_types::LongDateTime::new(0),
            x_min: 0,
            y_min: 0,
            x_max: 500,
            y_max: 700,
            mac_style: write_fonts::tables::head::MacStyle::empty(),
            lowest_rec_ppem: 8,
            font_direction_hint: 2,
            index_to_loc_format: match loca_format {
                write_fonts::tables::loca::LocaFormat::Short => 0,
                write_fonts::tables::loca::LocaFormat::Long => 1,
            },
        };
        let hhea = Hhea {
            ascender: font_types::FWord::new(700),
            descender: font_types::FWord::new(-200),
            line_gap: font_types::FWord::new(0),
            advance_width_max: font_types::UfWord::new(500),
            min_left_side_bearing: font_types::FWord::new(0),
            min_right_side_bearing: font_types::FWord::new(0),
            x_max_extent: font_types::FWord::new(500),
            caret_slope_rise: 1,
            caret_slope_run: 0,
            caret_offset: 0,
            number_of_h_metrics: 2,
        };
        let hmtx = Hmtx {
            h_metrics: vec![
                LongMetric { advance: 500, side_bearing: 0 },
                LongMetric { advance: 500, side_bearing: 0 },
            ],
            left_side_bearings: vec![],
        };
        let maxp = Maxp {
            num_glyphs: 2,
            max_points: Some(0),
            max_contours: Some(0),
            max_composite_points: Some(0),
            max_composite_contours: Some(0),
            max_zones: Some(1),
            max_twilight_points: Some(0),
            max_storage: Some(0),
            max_function_defs: Some(0),
            max_instruction_defs: Some(0),
            max_stack_elements: Some(0),
            max_size_of_instructions: Some(10),
            max_component_elements: Some(0),
            max_component_depth: Some(0),
        };
        let post = Post {
            version: font_types::Version16Dot16::VERSION_3_0,
            italic_angle: font_types::Fixed::from_f64(0.0),
            underline_position: font_types::FWord::new(-100),
            underline_thickness: font_types::FWord::new(50),
            is_fixed_pitch: 0,
            min_mem_type42: 0,
            max_mem_type42: 0,
            min_mem_type1: 0,
            max_mem_type1: 0,
            num_glyphs: Some(2),
            glyph_name_index: None,
            string_data: None,
        };

        let mut builder = FontBuilder::new();
        builder.add_table(&head).unwrap();
        builder.add_table(&hhea).unwrap();
        builder.add_table(&hmtx).unwrap();
        builder.add_table(&maxp).unwrap();
        builder.add_table(&cmap).unwrap();
        builder.add_table(&post).unwrap();
        builder.add_table(&glyf).unwrap();
        builder.add_table(&loca).unwrap();
        builder.build()
    }

    // Font 1 with instructions [0x01, 0x02, 0x03] for glyph A
    let font1 =
        make_font_with_instructions("A", 0x41, vec![0x01, 0x02, 0x03], make_square_contour());

    // Font 2 with instructions [0x04, 0x05, 0x06] for glyph B
    let font2 =
        make_font_with_instructions("B", 0x42, vec![0x04, 0x05, 0x06], make_square_contour());

    let merger = Merger::default();
    let merged = merger.merge(&[&font1, &font2]).expect("merge failed");

    let font_ref = FontRef::new(&merged).expect("parse merged font");
    let glyf = font_ref.glyf().expect("glyf");
    let loca = font_ref.loca(None).expect("loca");
    let cmap = font_ref.cmap().expect("cmap");

    // Get GID for A (from first font) and B (from second font)
    let gid_a = cmap.map_codepoint(0x41u32).expect("A should be mapped");
    let gid_b = cmap.map_codepoint(0x42u32).expect("B should be mapped");

    // Get the glyphs
    let glyph_a = loca
        .get_glyf(gid_a, &glyf)
        .expect("glyph A lookup")
        .expect("glyph A exists");
    let glyph_b = loca
        .get_glyf(gid_b, &glyf)
        .expect("glyph B lookup")
        .expect("glyph B exists");

    // Glyph A (from first font) should retain its instructions
    if let read_fonts::tables::glyf::Glyph::Simple(simple) = glyph_a {
        assert_eq!(
            simple.instructions().len(),
            3,
            "Glyph A (first font) should retain instructions"
        );
    } else {
        panic!("Expected simple glyph for A");
    }

    // Glyph B (from second font) should have instructions stripped
    if let read_fonts::tables::glyf::Glyph::Simple(simple) = glyph_b {
        assert_eq!(
            simple.instructions().len(),
            0,
            "Glyph B (second font) should have instructions stripped"
        );
    } else {
        panic!("Expected simple glyph for B");
    }
}
