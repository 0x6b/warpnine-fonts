#!/usr/bin/env python3
"""
Copy GSUB table (with FeatureVariations for ligatures) from Recursive VF to WarpnineMono VF.

The Recursive variable font has proper FeatureVariations that handle:
- Ligatures (liga, dlig, calt)
- Contextual glyph substitutions based on axis values (.mono vs .italic)

This script copies that entire GSUB table to our variable font.
"""

import sys
from pathlib import Path

from fontTools import ttLib

from src.logger import logger
from src.paths import BUILD_DIR, DIST_DIR


def copy_gsub_table(
    source_vf_path: Path, target_vf_path: Path, output_path: Path
) -> bool:
    """Copy GSUB table from source VF to target VF"""
    try:
        logger.info(f"Loading source VF: {source_vf_path}")
        source_font = ttLib.TTFont(source_vf_path)

        logger.info(f"Loading target VF: {target_vf_path}")
        target_font = ttLib.TTFont(target_vf_path)

        if "GSUB" not in source_font:
            logger.error(f"No GSUB table in source font: {source_vf_path}")
            return False

        # Copy GSUB table
        logger.info("Copying GSUB table...")
        source_gsub = source_font["GSUB"]

        # Check for FeatureVariations
        if hasattr(source_gsub.table, "FeatureVariations"):
            if source_gsub.table.FeatureVariations:
                num_variations = len(
                    source_gsub.table.FeatureVariations.FeatureVariationRecord
                )
                logger.info(f"  Source has {num_variations} FeatureVariationRecords")
            else:
                logger.info("  Source has no FeatureVariationRecords")
        else:
            logger.info("  Source has no FeatureVariations support")

        # List features
        if hasattr(source_gsub.table, "FeatureList") and source_gsub.table.FeatureList:
            features = [
                rec.FeatureTag for rec in source_gsub.table.FeatureList.FeatureRecord
            ]
            unique_features = sorted(set(features))
            logger.info(f"  Features: {', '.join(unique_features)}")

        # Copy the table
        target_font["GSUB"] = source_gsub

        # Save
        logger.info(f"Saving to: {output_path}")
        target_font.save(output_path)

        logger.info("GSUB table copied successfully!")
        return True

    except Exception as e:
        logger.error(f"Error copying GSUB: {e}")
        import traceback

        traceback.print_exc()
        return False


def main():
    """Copy GSUB from Recursive VF to WarpnineMono VF"""
    logger.info("Copying GSUB table with FeatureVariations to WarpnineMono VF...")

    # Source: Recursive variable font (has FeatureVariations)
    source_vf = BUILD_DIR / "Recursive_VF_1.085.ttf"

    # Target: WarpnineMono variable font (no GSUB currently)
    target_vf = DIST_DIR / "WarpnineMono-VF.ttf"

    # Output: new WarpnineMono VF with ligatures
    output_vf = DIST_DIR / "WarpnineMono-VF-with-ligatures.ttf"

    if not source_vf.exists():
        logger.error(f"Source VF not found: {source_vf}")
        logger.error("Run download step first")
        return False

    if not target_vf.exists():
        logger.error(f"Target VF not found: {target_vf}")
        logger.error("Run build step first")
        return False

    # Copy GSUB
    if copy_gsub_table(source_vf, target_vf, output_vf):
        logger.info("Success!")
        logger.info(f"Variable font with ligatures: {output_vf}")

        # Get file size
        size_mb = output_vf.stat().st_size / 1024 / 1024
        logger.info(f"  Size: {size_mb:.2f} MB")

        return True
    else:
        return False


if __name__ == "__main__":
    sys.exit(0 if main() else 1)
