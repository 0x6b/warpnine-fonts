#!/usr/bin/env python3
"""
Script to freeze OpenType features into fonts.

Permanently applies specific OpenType feature substitutions to glyph outlines,
making these features always active without requiring application support.

This approach is based on recursive-code-config:
https://github.com/arrowtype/recursive-code-config
"""

import subprocess
import sys
from pathlib import Path

from src.logger import logger
from src.paths import DIST_DIR

# Features to freeze for WarpnineMono
MONO_FEATURES = [
    "dlig",
    "ss01",
    "ss02",
    "ss03",
    "ss04",
    "ss05",
    "ss06",
    "ss07",
    "ss08",
    "ss10",
    "ss11",
    "ss12",
    "pnum",
    "liga",
]

# Features to freeze for WarpnineSansCondensed
SANS_FEATURES = [
    "ss01",
    "ss02",
    "ss03",
    "ss04",
    "ss05",
    "ss06",
    "ss07",
    "ss08",
    "ss10",
    "ss12",
    "case",
    "titl",
    "pnum",
    "liga",
]

# Font family configurations
FONT_CONFIGS = [
    ("WarpnineMono-VF.ttf", MONO_FEATURES),
    ("WarpnineSansCondensed-*.ttf", SANS_FEATURES),
]


def freeze_features_in_font(font_path: Path, features: list[str]) -> bool:
    """
    Freeze OpenType features into a font file using pyftfeatfreeze.

    Args:
        font_path: Path to the font file
        features: List of feature tags to freeze

    Returns:
        True if successful, False if failed
    """
    logger.info(f"Freezing features in {font_path.name}")
    features_str = f"rvrn,{','.join(features)}"
    logger.info(f"  Features: {features_str}")

    temp_path = font_path.with_suffix(".tmp.ttf")

    try:
        cmd = [
            "pyftfeatfreeze",
            f"--features={features_str}",
            str(font_path),
            str(temp_path),
        ]
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        temp_path.replace(font_path)
        logger.info(f"  ✓ Features frozen successfully")
        return True

    except subprocess.CalledProcessError as e:
        logger.error(f"  Error freezing features: {e}")
        if e.stderr:
            logger.error(f"  {e.stderr}")
        if temp_path.exists():
            temp_path.unlink()
        return False


def main():
    """Freeze features in all static fonts"""
    total_fonts = 0
    failures = []

    for pattern, features in FONT_CONFIGS:
        font_files = sorted(DIST_DIR.glob(pattern))

        if not font_files:
            continue

        logger.info(f"Processing {pattern}: {len(font_files)} fonts")
        total_fonts += len(font_files)

        for font_path in font_files:
            if not freeze_features_in_font(font_path, features):
                failures.append(font_path.name)

    if total_fonts == 0:
        logger.error("No fonts found in dist/")
        sys.exit(1)

    # Summary
    success_count = total_fonts - len(failures)
    logger.info(f"Feature freezing summary: {success_count}/{total_fonts} successful")

    if failures:
        logger.error(f"Failed to freeze features in {len(failures)} fonts:")
        for name in failures:
            logger.error(f"  - {name}")
        sys.exit(1)

    logger.info("Feature freezing completed successfully")


if __name__ == "__main__":
    main()
