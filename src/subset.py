#!/usr/bin/env python3
"""
Script to subset fonts

Extract only Japanese ranges from build/Noto-*.ttf
"""

import subprocess
import sys
from pathlib import Path

from src.logger import logger
from src.paths import BUILD_DIR

# Japanese-related Unicode ranges (basic)
# Reference: https://www.unicode.org/charts/
# Note: pyftsubset requires hex format without U+ prefix
JAPANESE_UNICODE_RANGES_BASIC = [
    # Symbols and punctuation
    "3000-303F",  # CJK Symbols and Punctuation
    # Hiragana
    "3041-3096",  # Hiragana (basic)
    "3099-309F",  # Hiragana (combining marks)
    # Katakana
    "30A0-30FF",  # Katakana
    # Kanji (basic)
    "4E00-9FFF",  # CJK Unified Ideographs
    # Fullwidth alphanumeric and symbols
    "FF00-FFEF",  # Halfwidth and Fullwidth Forms
]

# Japanese-related Unicode ranges (extended)
JAPANESE_UNICODE_RANGES_EXTENDED = [
    # Hiragana extension
    "1B100-1B12F",  # Kana Extended-A
    "1AFF0-1AFFF",  # Kana Extended-B
    "1B000-1B0FF",  # Kana Supplement
    "1B130-1B16F",  # Small Kana Extension
    # Kanji extension
    "3400-4DBF",  # CJK Unified Ideographs Extension A
    "20000-2A6DF",  # CJK Unified Ideographs Extension B
    "2A700-2B739",  # CJK Unified Ideographs Extension C
    "2B740-2B81D",  # CJK Unified Ideographs Extension D
    "2B820-2CEA1",  # CJK Unified Ideographs Extension E
    "2CEB0-2EBE0",  # CJK Unified Ideographs Extension F
    "30000-3134A",  # CJK Unified Ideographs Extension G
    "31350-323AF",  # CJK Unified Ideographs Extension H
    "2EBF0-2EE5D",  # CJK Unified Ideographs Extension I
    "F900-FAFF",  # CJK Compatibility Ideographs
    "2F800-2FA1F",  # CJK Compatibility Ideographs Supplement
]

# Default: basic ranges + extended ranges
JAPANESE_UNICODE_RANGES = (
    JAPANESE_UNICODE_RANGES_BASIC + JAPANESE_UNICODE_RANGES_EXTENDED
)


def subset_font(
    input_font: Path, output: Path, unicode_ranges: list[str], keep_layout: bool = True
) -> None:
    """
    Subset a font

    Args:
    input_font: Input font path
    output: Output file path
    unicode_ranges: List of Unicode ranges
    keep_layout: Whether to keep layout features
    """
    output.parent.mkdir(parents=True, exist_ok=True)

    unicodes = ",".join(unicode_ranges)

    logger.info(f"Subsetting {input_font.name}")
    logger.info(f"  Unicode ranges: {unicodes}")
    logger.info(f"  Output: {output}")

    cmd = [
        "pyftsubset",
        str(input_font),
        f"--unicodes={unicodes}",
        "--glyph-names",
        f"--output-file={output}",
        # Remove Variable Font tables to prevent merge errors
        "--drop-tables=HVAR,MVAR,STAT,avar,fvar,gvar,cvar",
    ]

    if keep_layout:
        cmd.append("--layout-features=*")

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        logger.info("Successfully subsetted")
        if result.stdout:
            logger.info(result.stdout)
    except subprocess.CalledProcessError as e:
        logger.error(f"Error subsetting font: {e}")
        if e.stderr:
            logger.error(e.stderr)
        sys.exit(1)


def main():
    # Search for build/Noto-*.ttf
    input_fonts = list(BUILD_DIR.glob("Noto-*.ttf"))

    # Exclude already subset files
    input_fonts = [f for f in input_fonts if "-subset" not in f.name]

    if not input_fonts:
        logger.error(f"Input files not found: {BUILD_DIR / 'Noto-*.ttf'}")
        logger.error("  Run extract first")
        sys.exit(1)

    # Subset each file
    for input_font in input_fonts:
        # Generate output filename
        output = BUILD_DIR / f"{input_font.stem}-subset.ttf"

        subset_font(input_font, output, JAPANESE_UNICODE_RANGES, keep_layout=True)

    logger.info("All fonts subsetted")


if __name__ == "__main__":
    main()
