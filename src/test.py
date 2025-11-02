#!/usr/bin/env python3
"""
Variable Font validation script.

Tests the `WarpnineMono-VF.ttf` already in `dist/`. If rebuild is
needed, generate it with `uv run python src/all.py` etc. before running this script.
"""

import sys

from fontTools.ttLib import TTFont

from src.logger import logger
from src.paths import DIST_DIR

# Expected values
EXPECTED_AXES = {
    "wght": {"min": 400, "max": 700, "default": 400},
    "ital": {"min": 0, "max": 1, "default": 0},
}
EXPECTED_GLYPHS = 51178  # Expected glyph count


def test_file_exists() -> bool:
    """Check file existence"""
    font_path = DIST_DIR / "WarpnineMono-VF.ttf"
    if not font_path.exists():
        logger.error(f"Font file not found: {font_path}")
        return False
    logger.info("Font file exists")
    return True


def test_font_structure() -> bool:
    """Test font structure"""
    font_path = DIST_DIR / "WarpnineMono-VF.ttf"

    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    logger.info("Font file is valid TTF")

    success = True

    # Check if Variable Font table exists
    if "fvar" not in font:
        logger.error("Missing 'fvar' table (not a Variable Font)")
        return False

    logger.info("Font is a Variable Font")

    # Validate axes
    fvar = font["fvar"]
    actual_axes = {axis.axisTag: axis for axis in fvar.axes}

    for axis_tag, expected in EXPECTED_AXES.items():
        if axis_tag not in actual_axes:
            logger.error(f"Missing axis: {axis_tag}")
            success = False
            continue

        axis = actual_axes[axis_tag]

        # Check range
        if (
            axis.minValue != expected["min"]
            or axis.maxValue != expected["max"]
            or axis.defaultValue != expected["default"]
        ):
            logger.error(f"Axis {axis_tag} range mismatch:")
            logger.error(
                f"  Expected: {expected['min']}-{expected['max']} "
                f"(default: {expected['default']})"
            )
            logger.error(
                f"  Actual: {axis.minValue}-{axis.maxValue} "
                f"(default: {axis.defaultValue})"
            )
            success = False
        else:
            logger.info(
                f"Axis {axis_tag}: {axis.minValue}-{axis.maxValue} "
                f"(default: {axis.defaultValue})"
            )

    # Check glyph count
    glyph_count = len(font.getGlyphOrder())
    if glyph_count != EXPECTED_GLYPHS:
        logger.error(
            f"Glyph count mismatch: expected {EXPECTED_GLYPHS}, got {glyph_count}"
        )
        success = False
    else:
        logger.info(f"Glyph count: {glyph_count}")

    # Sample check for Unicode ranges (are Japanese characters included?)
    cmap = font.getBestCmap()
    test_chars = {
        "あ": 0x3042,  # Hiragana sample
        "ア": 0x30A2,  # Katakana sample
        "漢": 0x6F22,  # Kanji sample
        "A": 0x0041,  # Latin sample
    }

    for char, codepoint in test_chars.items():
        if codepoint in cmap:
            logger.info(f"Character '{char}' (U+{codepoint:04X}) is supported")
        else:
            logger.error(f"Character '{char}' (U+{codepoint:04X}) is missing")
            success = False

    # Check calt feature registration
    if "GSUB" in font:
        gsub = font["GSUB"].table
        if hasattr(gsub, "FeatureList"):
            calt_found = False
            for record in gsub.FeatureList.FeatureRecord:
                if record.FeatureTag == "calt":
                    calt_found = True
                    break

            if calt_found:
                logger.info("calt feature is present")

                # Check if calt is registered to all scripts
                if hasattr(gsub, "ScriptList"):
                    scripts_with_calt = []
                    for script_record in gsub.ScriptList.ScriptRecord:
                        script = script_record.Script
                        has_calt = False

                        # Check DefaultLangSys
                        if script.DefaultLangSys and script.DefaultLangSys.FeatureIndex:
                            for feature_idx in script.DefaultLangSys.FeatureIndex:
                                if (
                                    gsub.FeatureList.FeatureRecord[
                                        feature_idx
                                    ].FeatureTag
                                    == "calt"
                                ):
                                    has_calt = True
                                    break

                        if has_calt:
                            scripts_with_calt.append(script_record.ScriptTag)

                    if scripts_with_calt:
                        logger.info(
                            f"calt registered to scripts: {', '.join(scripts_with_calt)}"
                        )
                    else:
                        logger.error("calt feature not registered to any script")
                        success = False
            else:
                logger.error("calt feature is missing")
                success = False
        else:
            logger.error("GSUB FeatureList not found")
            success = False
    else:
        logger.error("GSUB table not found")
        success = False

    font.close()
    return success


def test_monospace_width() -> bool:
    """Test monospace width consistency"""
    font_path = DIST_DIR / "WarpnineMono-VF.ttf"

    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    success = True

    # Get advance widths
    hmtx = font["hmtx"]
    cmap = font.getBestCmap()

    # Test ASCII/Latin characters should all have same width
    ascii_chars = list(range(0x0020, 0x007F))  # Basic Latin
    widths = {}

    for codepoint in ascii_chars:
        if codepoint in cmap:
            glyph_name = cmap[codepoint]
            width, lsb = hmtx[glyph_name]
            widths[chr(codepoint)] = width

    if widths:
        expected_width = widths.get(" ")  # Use space as reference
        if expected_width:
            non_matching = {
                char: width for char, width in widths.items() if width != expected_width
            }

            if non_matching:
                logger.error(
                    f"ASCII characters with non-monospace width (expected {expected_width}):"
                )
                for char, width in list(non_matching.items())[:10]:  # Show first 10
                    logger.error(f"  '{char}' (U+{ord(char):04X}): {width}")
                if len(non_matching) > 10:
                    logger.error(f"  ... and {len(non_matching) - 10} more")
                success = False
            else:
                logger.info(
                    f"All ASCII characters have monospace width: {expected_width}"
                )

    # Test CJK characters (Noto uses 1000 unit width, not 2x ASCII)
    cjk_samples = [
        0x3042,  # あ Hiragana
        0x30A2,  # ア Katakana
        0x6F22,  # 漢 Kanji
        0xFF21,  # Ａ Fullwidth Latin
    ]

    cjk_widths = {}
    for codepoint in cjk_samples:
        if codepoint in cmap:
            glyph_name = cmap[codepoint]
            width, lsb = hmtx[glyph_name]
            cjk_widths[chr(codepoint)] = width

    if cjk_widths:
        # Noto Sans Mono CJK JP uses 1000 unit width for CJK characters
        expected_cjk_width = 1000
        for char, width in cjk_widths.items():
            if width == expected_cjk_width:
                logger.info(
                    f"CJK character '{char}' (U+{ord(char):04X}) has correct width: {width}"
                )
            else:
                logger.error(
                    f"CJK character '{char}' (U+{ord(char):04X}) width mismatch: expected {expected_cjk_width}, got {width}"
                )
                success = False

    font.close()
    return success


def test_font_metadata() -> bool:
    """Test font metadata (copyright, family name, isFixedPitch)"""
    font_path = DIST_DIR / "WarpnineMono-VF.ttf"

    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    success = True

    # Check copyright
    name_table = font["name"]
    copyright_found = False
    expected_copyright_parts = [
        "Copyright 2020 The Recursive Project Authors",
        "Copyright 2014-2021 Adobe",
        "with Reserved Font Name 'Source'",
        "Warpnine Mono is based on Recursive Mono Duotone and Noto Sans Mono CJK JP",
    ]

    for record in name_table.names:
        if record.nameID == 0:
            copyright_text = record.toUnicode()
            copyright_found = True

            all_parts_present = all(
                part in copyright_text for part in expected_copyright_parts
            )

            if all_parts_present:
                logger.info("Copyright notice is correct")
            else:
                logger.error(f"Copyright notice is incomplete: {copyright_text}")
                for part in expected_copyright_parts:
                    if part not in copyright_text:
                        logger.error(f"  Missing: {part}")
                success = False
            break

    if not copyright_found:
        logger.error("Copyright notice (nameID 0) not found")
        success = False

    # Check family name (nameID 16 - Typographic Family)
    family_name_found = False
    for record in name_table.names:
        if record.nameID == 16:
            family_name = record.toUnicode()
            family_name_found = True
            if family_name == "Warpnine Mono":
                logger.info(f"Font family name is correct: {family_name}")
            else:
                logger.error(
                    f"Font family name mismatch: expected 'Warpnine Mono', got '{family_name}'"
                )
                success = False
            break

    if not family_name_found:
        logger.error("Font family name (nameID 16) not found")
        success = False

    # Check isFixedPitch flag
    post_table = font["post"]
    if post_table.isFixedPitch:
        logger.info("isFixedPitch flag is set correctly")
    else:
        logger.error("isFixedPitch flag is not set")
        success = False

    font.close()
    return success


def test_required_tables() -> bool:
    """Test presence of required OpenType tables"""
    font_path = DIST_DIR / "WarpnineMono-VF.ttf"

    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    required_tables = [
        "head",
        "hhea",
        "hmtx",
        "maxp",
        "name",
        "OS/2",
        "post",
        "cmap",
        "glyf",
        "GSUB",
        "fvar",  # Variable font specific
    ]

    success = True
    missing_tables = []

    for table_name in required_tables:
        if table_name in font:
            logger.info(f"Table '{table_name}' is present")
        else:
            logger.error(f"Required table '{table_name}' is missing")
            missing_tables.append(table_name)
            success = False

    if missing_tables:
        logger.error(f"Missing {len(missing_tables)} required tables")

    font.close()
    return success


def test_named_instances() -> bool:
    """Test that named instances exist with correct coordinates"""
    font_path = DIST_DIR / "WarpnineMono-VF.ttf"

    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    success = True

    expected_instances = {
        "Regular": {"wght": 400.0, "ital": 0.0},
        "Bold": {"wght": 700.0, "ital": 0.0},
        "Italic": {"wght": 400.0, "ital": 1.0},
        "Bold Italic": {"wght": 700.0, "ital": 1.0},
    }

    if "fvar" not in font:
        logger.error("fvar table not found")
        font.close()
        return False

    fvar = font["fvar"]
    actual_instances = {}

    for instance in fvar.instances:
        # Get instance name from name table
        instance_name = None
        for record in font["name"].names:
            if record.nameID == instance.subfamilyNameID:
                instance_name = record.toUnicode()
                break

        if instance_name:
            actual_instances[instance_name] = instance.coordinates

    # Check each expected instance
    for name, expected_coords in expected_instances.items():
        if name in actual_instances:
            actual_coords = actual_instances[name]
            if actual_coords == expected_coords:
                logger.info(
                    f"Named instance '{name}' has correct coordinates: {actual_coords}"
                )
            else:
                logger.error(
                    f"Named instance '{name}' coordinate mismatch: expected {expected_coords}, got {actual_coords}"
                )
                success = False
        else:
            logger.error(f"Named instance '{name}' not found")
            success = False

    font.close()
    return success


def test_unicode_coverage() -> bool:
    """Test Unicode range coverage"""
    font_path = DIST_DIR / "WarpnineMono-VF.ttf"

    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    success = True
    cmap = font.getBestCmap()

    # Define ranges to test (name, start, end, sample_char)
    test_ranges = [
        (
            "Basic Latin (printable)",
            0x0020,
            0x007E,
            "A",
        ),  # Exclude U+007F (DELETE control char)
        ("CJK Symbols and Punctuation", 0x3000, 0x303F, "、"),
        (
            "Hiragana",
            0x3041,
            0x3096,
            "あ",
        ),  # Exclude U+3040 (unassigned), U+3097-3098 (rare combining marks)
        ("Katakana", 0x30A0, 0x30FF, "ア"),
        ("CJK Unified Ideographs", 0x4E00, 0x9FFF, "漢"),
        (
            "Fullwidth ASCII",
            0xFF01,
            0xFF5E,
            "Ａ",
        ),  # Fullwidth versions of printable ASCII (! to ~)
    ]

    for range_name, start, end, sample in test_ranges:
        # Count how many codepoints in this range are present
        present = sum(1 for cp in range(start, end + 1) if cp in cmap)
        total = end - start + 1
        coverage = (present / total) * 100 if total > 0 else 0

        sample_cp = ord(sample)
        sample_present = sample_cp in cmap

        if sample_present and coverage > 0:
            logger.info(
                f"{range_name}: {coverage:.1f}% coverage ({present}/{total}), sample '{sample}' present"
            )
        else:
            logger.error(
                f"{range_name}: {coverage:.1f}% coverage ({present}/{total}), sample '{sample}' {'present' if sample_present else 'MISSING'}"
            )
            success = False

    font.close()
    return success


def test_ligatures() -> bool:
    """Test that sample ligatures exist in GSUB table"""
    font_path = DIST_DIR / "WarpnineMono-VF.ttf"

    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    success = True

    # Sample ligatures to check (these are from Recursive)
    sample_ligatures = [
        "->",
        "=>",
        "!=",
        "==",
        ">=",
        "<=",
        "||",
        "&&",
        "<!--",
        "-->",
    ]

    if "GSUB" not in font:
        logger.error("GSUB table not found, cannot test ligatures")
        font.close()
        return False

    gsub = font["GSUB"].table

    # This is a basic check - just verify GSUB has lookup tables
    # A more detailed check would parse the actual substitution rules
    if hasattr(gsub, "LookupList") and gsub.LookupList:
        lookup_count = len(gsub.LookupList.Lookup)
        logger.info(
            f"GSUB table has {lookup_count} lookup tables (ligatures likely present)"
        )

        # Verify calt feature exists (already tested elsewhere but good to confirm here)
        if hasattr(gsub, "FeatureList"):
            calt_exists = any(
                record.FeatureTag == "calt" for record in gsub.FeatureList.FeatureRecord
            )
            if calt_exists:
                logger.info("calt feature present (enables contextual ligatures)")
            else:
                logger.error("calt feature not found")
                success = False
    else:
        logger.error("GSUB LookupList is empty")
        success = False

    font.close()
    return success


def main():
    logger.info("Testing Variable Font")

    tests = [
        ("File existence", test_file_exists),
        ("Font structure", test_font_structure),
        ("Monospace width", test_monospace_width),
        ("Font metadata", test_font_metadata),
        ("Required tables", test_required_tables),
        ("Named instances", test_named_instances),
        ("Unicode coverage", test_unicode_coverage),
        ("Ligatures", test_ligatures),
    ]

    all_passed = True
    for name, test_func in tests:
        logger.info(f"--- {name} ---")
        if not test_func():
            all_passed = False

    if all_passed:
        logger.info("All tests passed")
        sys.exit(0)
    else:
        logger.error("Some tests failed")
        sys.exit(1)


if __name__ == "__main__":
    main()
