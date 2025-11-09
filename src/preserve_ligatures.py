#!/usr/bin/env python3
"""
Preserve ligatures from Recursive fonts while normalizing incompatible GSUB substitutions.

Strategy:
1. Extract GSUB from RecMonoDuotone fonts (before merge with Noto)
2. Keep ligature features (liga, dlig, calt)
3. Remove stylistic set features (ss01-ss20) that contain .mono/.italic substitutions
4. Apply preserved GSUB to merged WarpnineMono fonts
"""

import shutil
import sys
from pathlib import Path

from fontTools import ttLib

from src.logger import logger
from src.paths import BUILD_DIR, DIST_DIR


def create_ligature_only_gsub(source_font_path: Path) -> dict:
    """Create a modified GSUB table with only ligature features"""
    try:
        font = ttLib.TTFont(source_font_path)

        if "GSUB" not in font:
            logger.error(f"No GSUB table in {source_font_path}")
            return None

        gsub = font["GSUB"]

        # Features to KEEP (only pure ligatures, no contextual alternates)
        # NOTE: calt, ccmp, rvrn all contain .mono/.italic substitutions which break varLib
        features_to_keep = {
            "liga",  # Standard ligatures
            "dlig",  # Discretionary ligatures
        }

        # Get feature list
        if hasattr(gsub.table, "FeatureList") and gsub.table.FeatureList:
            feature_records = gsub.table.FeatureList.FeatureRecord

            # Find features to remove
            indices_to_remove = []
            kept_features = []

            for i, feature_record in enumerate(feature_records):
                if feature_record.FeatureTag not in features_to_keep:
                    indices_to_remove.append(i)
                else:
                    kept_features.append(feature_record.FeatureTag)

            logger.info(f"  Keeping features: {', '.join(sorted(kept_features))}")
            logger.info(f"  Removing {len(indices_to_remove)} other features")

            # Remove features in reverse order
            for i in reversed(indices_to_remove):
                feature_records.pop(i)

            gsub.table.FeatureList.FeatureCount = len(feature_records)

            # Update script references
            if hasattr(gsub.table, "ScriptList") and gsub.table.ScriptList:
                for script_record in gsub.table.ScriptList.ScriptRecord:
                    script = script_record.Script

                    # Update DefaultLangSys
                    if hasattr(script, "DefaultLangSys") and script.DefaultLangSys:
                        lang_sys = script.DefaultLangSys
                        new_feature_indices = []

                        for feature_index in lang_sys.FeatureIndex:
                            offset = sum(
                                1
                                for removed_idx in indices_to_remove
                                if removed_idx < feature_index
                            )
                            new_index = feature_index - offset

                            if feature_index not in indices_to_remove:
                                new_feature_indices.append(new_index)

                        lang_sys.FeatureIndex = new_feature_indices
                        lang_sys.FeatureCount = len(new_feature_indices)

                    # Update other LangSys records
                    if hasattr(script, "LangSysRecord"):
                        for lang_sys_record in script.LangSysRecord:
                            lang_sys = lang_sys_record.LangSys
                            new_feature_indices = []

                            for feature_index in lang_sys.FeatureIndex:
                                offset = sum(
                                    1
                                    for removed_idx in indices_to_remove
                                    if removed_idx < feature_index
                                )
                                new_index = feature_index - offset

                                if feature_index not in indices_to_remove:
                                    new_feature_indices.append(new_index)

                            lang_sys.FeatureIndex = new_feature_indices
                            lang_sys.FeatureCount = len(new_feature_indices)

        return gsub

    except Exception as e:
        logger.error(f"Error extracting GSUB from {source_font_path}: {e}")
        import traceback

        traceback.print_exc()
        return None


def apply_gsub_to_font(target_font_path: Path, gsub_table) -> bool:
    """Apply GSUB table to target font"""
    try:
        # Load target font
        target_font = ttLib.TTFont(target_font_path)

        # Replace GSUB table
        target_font["GSUB"] = gsub_table

        # Save
        target_font.save(target_font_path)
        return True

    except Exception as e:
        logger.error(f"Error applying GSUB to {target_font_path}: {e}")
        import traceback

        traceback.print_exc()
        return False


def main():
    """Preserve ligatures in WarpnineMono fonts"""
    logger.info("Preserving ligatures from Recursive fonts...")

    # Step 1: Extract ligature GSUB from Recursive fonts
    recursive_regular = BUILD_DIR / "RecMonoDuotone-Regular.ttf"
    recursive_italic = BUILD_DIR / "RecMonoDuotone-Italic.ttf"

    if not recursive_regular.exists():
        logger.error(f"RecMonoDuotone-Regular.ttf not found in {BUILD_DIR}")
        logger.error("Run extract-duotone first")
        return False

    # Create GSUB for upright and italic separately
    logger.info("Creating ligature GSUB from upright font...")
    gsub_upright = create_ligature_only_gsub(recursive_regular)
    if not gsub_upright:
        return False

    logger.info("Creating ligature GSUB from italic font...")
    gsub_italic = create_ligature_only_gsub(recursive_italic)
    if not gsub_italic:
        return False

    # Step 2: Apply GSUB to all WarpnineMono fonts
    logger.info("\nApplying ligature GSUB to WarpnineMono fonts...")

    warpnine_fonts = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))

    # Exclude variable fonts and test fonts
    warpnine_fonts = [
        f for f in warpnine_fonts if not f.stem.endswith("-VF") and "Test" not in f.stem
    ]

    # Restore backups first
    for font_path in warpnine_fonts:
        backup_path = font_path.with_suffix(".ttf.bak")
        if backup_path.exists():
            shutil.copy2(backup_path, font_path)
            logger.info(f"Restored backup: {font_path.name}")

    success_count = 0
    for font_path in warpnine_fonts:
        # Determine if italic or upright
        is_italic = "Italic" in font_path.stem

        gsub_table = gsub_italic if is_italic else gsub_upright

        logger.info(
            f"Applying {'italic' if is_italic else 'upright'} GSUB to {font_path.name}..."
        )

        if apply_gsub_to_font(font_path, gsub_table):
            success_count += 1

    logger.info(
        f"\nApplied ligature GSUB to {success_count}/{len(warpnine_fonts)} fonts"
    )
    return success_count == len(warpnine_fonts)


if __name__ == "__main__":
    sys.exit(0 if main() else 1)
