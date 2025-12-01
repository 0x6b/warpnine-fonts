"""
Font validation.

Tests fonts in dist/ including VF and static fonts.
"""

import sys
from pathlib import Path

from fontTools.ttLib import TTFont

from warpnine_fonts.config.paths import (
    DIST_DIR,
    WARPNINE_MONO,
    WARPNINE_SANS,
    WARPNINE_SANS_CONDENSED,
)
from warpnine_fonts.utils.logging import logger


# Expected frozen feature glyph mappings
# Maps codepoint to (expected_glyph_or_list, feature_description)
# When a list is provided, any of those glyphs is acceptable (e.g., upright vs italic variants)
FROZEN_GLYPH_CHECKS = {
    # ss01: Single-story a
    0x0061: ("a.simple", "ss01"),
    # ss02: Single-story g
    0x0067: ("g.simple", "ss02"),
    # ss03: Simplified f
    0x0066: ("f.simple", "ss03"),
    # ss04: Simplified i (i.mono for upright, i.italic for italic fonts)
    0x0069: (["i.mono", "i.italic"], "ss04"),
    # ss05: Simplified l
    0x006C: ("l.simple", "ss05"),
    # ss06: Simplified r
    0x0072: ("r.simple", "ss06"),
    # ss08: No-serif L and Z
    0x004C: ("L.sans", "ss08"),
    0x005A: ("Z.sans", "ss08"),
    # ss10 + pnum: Dotted zero with proportional numerals
    0x0030: ("zero.dotted_pnum", "ss10+pnum"),
    # ss11: Simplified 1
    0x0031: ("one.simple", "ss11"),
    # ss12: Simplified @
    0x0040: ("at.alt", "ss12"),
}

# Expected values
# wght range: 300 (Light) to 1000 (ExtraBlack), default 400 (Regular)
# ital range: 0 (upright) to 1 (italic), default 0
EXPECTED_AXES = {
    "wght": {"min": 300, "max": 1000, "default": 400},
    "ital": {"min": 0, "max": 1, "default": 0},
}
# Glyph count may vary slightly between builds; use None to skip strict check
EXPECTED_GLYPHS: int | None = None


def test_file_exists() -> bool:
    """Check file existence."""
    font_path = DIST_DIR / f"{WARPNINE_MONO}-VF.ttf"
    if not font_path.exists():
        logger.error(f"Font file not found: {font_path}")
        return False
    logger.info("Font file exists")
    return True


def test_font_structure() -> bool:
    """Test font structure."""
    font_path = DIST_DIR / f"{WARPNINE_MONO}-VF.ttf"

    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    logger.info("Font file is valid TTF")
    success = True

    if "fvar" not in font:
        logger.error("Missing 'fvar' table (not a Variable Font)")
        return False

    logger.info("Font is a Variable Font")

    fvar = font["fvar"]
    actual_axes = {axis.axisTag: axis for axis in fvar.axes}

    for axis_tag, expected in EXPECTED_AXES.items():
        if axis_tag not in actual_axes:
            logger.error(f"Missing axis: {axis_tag}")
            success = False
            continue

        axis = actual_axes[axis_tag]
        if (
            axis.minValue != expected["min"]
            or axis.maxValue != expected["max"]
            or axis.defaultValue != expected["default"]
        ):
            logger.error(f"Axis {axis_tag} range mismatch")
            success = False
        else:
            logger.info(
                f"Axis {axis_tag}: {axis.minValue}-{axis.maxValue} (default: {axis.defaultValue})"
            )

    glyph_count = len(font.getGlyphOrder())
    if EXPECTED_GLYPHS is not None and glyph_count != EXPECTED_GLYPHS:
        logger.error(
            f"Glyph count mismatch: expected {EXPECTED_GLYPHS}, got {glyph_count}"
        )
        success = False
    else:
        logger.info(f"Glyph count: {glyph_count}")

    cmap = font.getBestCmap()
    test_chars = {
        "あ": 0x3042,
        "ア": 0x30A2,
        "漢": 0x6F22,
        "A": 0x0041,
    }

    for char, codepoint in test_chars.items():
        if codepoint in cmap:
            logger.info(f"Character '{char}' (U+{codepoint:04X}) is supported")
        else:
            logger.error(f"Character '{char}' (U+{codepoint:04X}) is missing")
            success = False

    if "GSUB" in font:
        gsub = font["GSUB"].table
        if hasattr(gsub, "FeatureList") and gsub.FeatureList.FeatureRecord:
            features = [rec.FeatureTag for rec in gsub.FeatureList.FeatureRecord]
            logger.info(f"GSUB features present: {', '.join(sorted(set(features)))}")
        else:
            logger.info("GSUB table present (features may be frozen)")
    else:
        logger.warning("GSUB table not found (features may be frozen into glyphs)")

    font.close()
    return success


def test_monospace_width() -> bool:
    """Test monospace width consistency."""
    font_path = DIST_DIR / f"{WARPNINE_MONO}-VF.ttf"

    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    success = True
    hmtx = font["hmtx"]
    cmap = font.getBestCmap()

    ascii_chars = list(range(0x0020, 0x007F))
    widths = {}

    for codepoint in ascii_chars:
        if codepoint in cmap:
            glyph_name = cmap[codepoint]
            width, lsb = hmtx[glyph_name]
            widths[chr(codepoint)] = width

    if widths:
        expected_width = widths.get(" ")
        if expected_width:
            non_matching = {c: w for c, w in widths.items() if w != expected_width}
            if non_matching:
                logger.error(
                    f"ASCII chars with non-monospace width (expected {expected_width})"
                )
                success = False
            else:
                logger.info(
                    f"All ASCII characters have monospace width: {expected_width}"
                )

    cjk_samples = [0x3042, 0x30A2, 0x6F22, 0xFF21]
    expected_cjk_width = 1000

    for codepoint in cjk_samples:
        if codepoint in cmap:
            glyph_name = cmap[codepoint]
            width, lsb = hmtx[glyph_name]
            if width != expected_cjk_width:
                logger.error(
                    f"CJK char U+{codepoint:04X} width: {width} (expected {expected_cjk_width})"
                )
                success = False

    font.close()
    return success


def test_font_metadata() -> bool:
    """Test font metadata."""
    font_path = DIST_DIR / f"{WARPNINE_MONO}-VF.ttf"

    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    success = True
    name_table = font["name"]

    # Check copyright
    expected_parts = [
        "Copyright 2020 The Recursive Project Authors",
        "Copyright 2014-2021 Adobe",
    ]

    copyright_found = False
    for record in name_table.names:
        if record.nameID == 0:
            copyright_text = record.toUnicode()
            copyright_found = True
            if all(p in copyright_text for p in expected_parts):
                logger.info("Copyright notice is correct")
            else:
                logger.error("Copyright notice incomplete")
                success = False
            break

    if not copyright_found:
        logger.error("Copyright notice not found")
        success = False

    # Check family name
    for record in name_table.names:
        if record.nameID == 16:
            family_name = record.toUnicode()
            if family_name == "Warpnine Mono":
                logger.info(f"Font family name is correct: {family_name}")
            else:
                logger.error(f"Font family name mismatch: {family_name}")
                success = False
            break

    # Check isFixedPitch
    if font["post"].isFixedPitch:
        logger.info("isFixedPitch flag is set correctly")
    else:
        logger.error("isFixedPitch flag is not set")
        success = False

    font.close()
    return success


def test_required_tables() -> bool:
    """Test presence of required OpenType tables."""
    font_path = DIST_DIR / f"{WARPNINE_MONO}-VF.ttf"

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
        "fvar",
    ]

    success = True
    for table_name in required_tables:
        if table_name in font:
            logger.info(f"Table '{table_name}' is present")
        else:
            logger.error(f"Required table '{table_name}' is missing")
            success = False

    font.close()
    return success


def validate_vf() -> None:
    """Run all validation tests."""
    logger.info("Testing Variable Font")

    tests = [
        ("File existence", test_file_exists),
        ("Font structure", test_font_structure),
        ("Monospace width", test_monospace_width),
        ("Font metadata", test_font_metadata),
        ("Required tables", test_required_tables),
    ]

    all_passed = True
    for name, test_func in tests:
        logger.info(f"--- {name} ---")
        if not test_func():
            all_passed = False

    if all_passed:
        logger.info("All tests passed")
    else:
        logger.error("Some tests failed")
        sys.exit(1)


def test_frozen_features(font_path: Path) -> bool:
    """
    Test that OpenType features have been frozen correctly.

    Checks that the cmap points to the expected alternate glyphs.
    """
    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    cmap = font.getBestCmap()
    success = True

    for codepoint, (expected, feature) in FROZEN_GLYPH_CHECKS.items():
        actual_glyph = cmap.get(codepoint)
        char = chr(codepoint)

        # Handle both single glyph and list of acceptable glyphs
        expected_glyphs = expected if isinstance(expected, list) else [expected]

        if actual_glyph in expected_glyphs:
            logger.info(
                f"  '{char}' (U+{codepoint:04X}) [{feature:10}] -> {actual_glyph}"
            )
        else:
            expected_str = " or ".join(expected_glyphs)
            logger.error(
                f"  '{char}' (U+{codepoint:04X}) [{feature}]: expected {expected_str}, got {actual_glyph}"
            )
            success = False

    font.close()
    return success


def validate_frozen() -> None:
    """Validate frozen features in all fonts."""
    logger.info("Validating frozen features")

    # Collect all font files to check
    font_patterns = [
        ("Variable Font", f"{WARPNINE_MONO}-VF.ttf"),
        ("Static Mono", f"{WARPNINE_MONO}-*.ttf"),
    ]

    all_passed = True
    fonts_checked = 0

    for category, pattern in font_patterns:
        font_files = sorted(DIST_DIR.glob(pattern))

        # Exclude VF from static pattern
        if "VF" not in pattern:
            font_files = [f for f in font_files if "-VF" not in f.name]

        if not font_files:
            logger.warning(f"No {category} fonts found")
            continue

        logger.info(f"--- {category} ({len(font_files)} fonts) ---")

        for font_path in font_files:
            logger.info(f"Checking {font_path.name}")
            if not test_frozen_features(font_path):
                all_passed = False
            fonts_checked += 1

    if fonts_checked == 0:
        logger.error("No fonts found to validate")
        sys.exit(1)

    if all_passed:
        logger.info(f"All {fonts_checked} fonts have correctly frozen features")
    else:
        logger.error("Some fonts have incorrect frozen features")
        sys.exit(1)


def test_sans_width_class(font_path: Path, expected_width_class: int) -> bool:
    """Test that a sans font has the expected OS/2 usWidthClass."""
    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    actual = font["OS/2"].usWidthClass
    font.close()

    if actual == expected_width_class:
        logger.info(f"  usWidthClass: {actual} (correct)")
        return True
    else:
        logger.error(f"  usWidthClass: {actual} (expected {expected_width_class})")
        return False


def test_sans_family_name(font_path: Path, expected_family: str) -> bool:
    """Test that a sans font has the expected typographic family name (nameID 16)."""
    try:
        font = TTFont(font_path)
    except Exception as e:
        logger.error(f"Failed to load font: {e}")
        return False

    name_table = font["name"]
    typographic_family = name_table.getDebugName(16)
    font.close()

    if typographic_family == expected_family:
        logger.info(f"  Typographic family: {typographic_family} (correct)")
        return True
    else:
        logger.error(
            f"  Typographic family: {typographic_family} (expected {expected_family})"
        )
        return False


def validate_sans() -> None:
    """Validate WarpnineSans and WarpnineSansCondensed fonts."""
    logger.info("Validating Sans fonts")

    # Width class values:
    # 3 = Condensed
    # 5 = Normal (Medium)
    font_configs = [
        (WARPNINE_SANS, "Warpnine Sans", 5),
        (WARPNINE_SANS_CONDENSED, "Warpnine Sans Condensed", 3),
    ]

    all_passed = True
    fonts_checked = 0

    for prefix, expected_family, expected_width_class in font_configs:
        font_files = sorted(DIST_DIR.glob(f"{prefix}-*.ttf"))

        if not font_files:
            logger.warning(f"No {prefix} fonts found")
            continue

        logger.info(f"--- {prefix} ({len(font_files)} fonts) ---")

        for font_path in font_files:
            logger.info(f"Checking {font_path.name}")
            if not test_sans_width_class(font_path, expected_width_class):
                all_passed = False
            if not test_sans_family_name(font_path, expected_family):
                all_passed = False
            fonts_checked += 1

    if fonts_checked == 0:
        logger.error("No sans fonts found to validate")
        sys.exit(1)

    if all_passed:
        logger.info(f"All {fonts_checked} sans fonts validated successfully")
    else:
        logger.error("Some sans fonts failed validation")
        sys.exit(1)
