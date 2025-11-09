#!/usr/bin/env python3
"""
Normalize GSUB tables by removing substitution features that differ between upright and italic.

The issue: Upright fonts use `.mono` glyphs, italic fonts use `.italic` glyphs.
fontTools varLib requires all masters to have identical GSUB substitution mappings.

Solution: Remove these substitution features entirely, using the base glyphs instead.
"""

import sys
from pathlib import Path

from fontTools import ttLib

from src.logger import logger
from src.paths import DIST_DIR


def remove_gsub_features(font_path: Path, output_path: Path) -> bool:
    """Remove GSUB table entirely to avoid incompatibility issues"""
    try:
        font = ttLib.TTFont(font_path)

        if "GSUB" not in font:
            logger.info(f"  No GSUB table in {font_path.name}")
            font.save(output_path)
            return True

        # Simply remove the entire GSUB table
        del font["GSUB"]
        logger.info(f"  Removed GSUB table from {font_path.name}")

        font.save(output_path)
        return True

    except Exception as e:
        logger.error(f"Error processing {font_path}: {e}")
        return False


def main():
    """Normalize GSUB tables for all fonts in dist/"""
    logger.info("Normalizing GSUB tables...")

    # Process all WarpnineMono fonts
    font_files = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))

    if not font_files:
        logger.error("No WarpnineMono fonts found in dist/")
        return False

    success_count = 0
    for font_path in font_files:
        logger.info(f"Processing {font_path.name}...")

        # Create backup
        backup_path = font_path.with_suffix(".ttf.bak")
        if not backup_path.exists():
            import shutil

            shutil.copy2(font_path, backup_path)
            logger.info(f"  Created backup: {backup_path.name}")

        # Normalize GSUB
        if remove_gsub_features(font_path, font_path):
            success_count += 1

    logger.info(f"Normalized {success_count}/{len(font_files)} fonts")
    return success_count == len(font_files)


if __name__ == "__main__":
    sys.exit(0 if main() else 1)
