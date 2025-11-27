#!/usr/bin/env python3
"""
Script to freeze OpenType features into fonts.

Permanently applies specific OpenType feature substitutions (dlig, ss01, ss02, ss04,
ss05, ss07, ss08, ss10, ss12) to glyph outlines, making these features always active
without requiring application support.

This approach is based on recursive-code-config:
https://github.com/arrowtype/recursive-code-config
"""

import subprocess
import sys
from pathlib import Path

from src.logger import logger
from src.paths import DIST_DIR

# Features to freeze
# Note: 'case' feature is excluded as it can cause issues with feature freezing
# (see recursive-code-config issue #20)
FEATURES = [
    "dlig",
    "ss01",
    "ss02",
    "ss04",
    "ss05",
    "ss07",
    "ss08",
    "ss10",
    "ss12",
]


def freeze_features_in_font(font_path: Path) -> bool:
    """
    Freeze OpenType features into a font file using pyftfeatfreeze.

    Args:
        font_path: Path to the font file

    Returns:
        True if successful, False if failed
    """
    logger.info(f"Freezing features in {font_path.name}")
    features_str = f"rvrn,{','.join(FEATURES)}"
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
    font_files = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))

    if not font_files:
        logger.error("No WarpnineMono fonts found in dist/")
        logger.error("  Run merge first")
        sys.exit(1)

    logger.info(f"Found {len(font_files)} fonts to process")

    failures = []
    for font_path in font_files:
        if not freeze_features_in_font(font_path):
            failures.append(font_path.name)

    # Summary
    success_count = len(font_files) - len(failures)
    logger.info(
        f"Feature freezing summary: {success_count}/{len(font_files)} successful"
    )

    if failures:
        logger.error(f"Failed to freeze features in {len(failures)} fonts:")
        for name in failures:
            logger.error(f"  - {name}")
        sys.exit(1)

    logger.info("Feature freezing completed successfully")


if __name__ == "__main__":
    main()
