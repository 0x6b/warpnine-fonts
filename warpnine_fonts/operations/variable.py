"""
Variable font building operations.

Builds WarpnineMono variable font from static masters.
"""

import shutil
import sys
from dataclasses import dataclass
from pathlib import Path

from fontTools.designspaceLib import (
    AxisDescriptor,
    DesignSpaceDocument,
    InstanceDescriptor,
    SourceDescriptor,
)
from fontTools.ttLib import TTFont

from warpnine_fonts.config.instances import DUOTONE_INSTANCES
from warpnine_fonts.config.paths import BUILD_DIR, DIST_DIR, RECURSIVE_VF, WARPNINE_MONO
from warpnine_fonts.core.gsub import (
    copy_gsub_from,
    fix_calt_registration,
    remove_gsub_table,
)
from warpnine_fonts.core.naming import update_vf_names
from warpnine_fonts.utils.logging import logger
from warpnine_fonts.utils.subprocess import run_fonttools


@dataclass
class SourceInfo:
    """Source font configuration for designspace."""

    filename: str
    weight: int
    italic: int
    is_copy_info: bool = False


@dataclass
class InstanceInfo:
    """Instance configuration for designspace."""

    style_name: str
    weight: int
    italic: int


def create_designspace() -> DesignSpaceDocument:
    """Generate designspace file for variable font."""
    doc = DesignSpaceDocument()

    # Weight axis: 300 (Light) - 1000 (ExtraBlack)
    weight_axis = AxisDescriptor()
    weight_axis.name = "Weight"
    weight_axis.tag = "wght"
    weight_axis.minimum = 300
    weight_axis.default = 400
    weight_axis.maximum = 1000
    doc.addAxis(weight_axis)

    # Italic axis: 0 (Upright) - 1 (Italic)
    italic_axis = AxisDescriptor()
    italic_axis.name = "Italic"
    italic_axis.tag = "ital"
    italic_axis.minimum = 0
    italic_axis.default = 0
    italic_axis.maximum = 1
    doc.addAxis(italic_axis)

    # Generate sources from DUOTONE_INSTANCES
    for instance in DUOTONE_INSTANCES:
        source = SourceDescriptor()
        font_path = str((DIST_DIR / f"{WARPNINE_MONO}-{instance.style}.ttf").resolve())
        source.filename = font_path
        source.path = font_path
        source.familyName = WARPNINE_MONO
        source.styleName = instance.style
        source.location = {
            "Weight": int(instance.wght),
            "Italic": 1 if instance.italic else 0,
        }
        doc.addSource(source)

    # Generate instances
    for instance in DUOTONE_INSTANCES:
        inst = InstanceDescriptor()
        inst.familyName = WARPNINE_MONO
        inst.styleName = instance.style
        inst.filename = str(DIST_DIR / f"instance_{instance.style}.ttf")
        inst.location = {
            "Weight": int(instance.wght),
            "Italic": 1 if instance.italic else 0,
        }
        doc.addInstance(inst)

    return doc


def remove_gsub_from_static_fonts() -> None:
    """
    Remove GSUB tables from all static mono fonts.

    This is necessary before building the variable font to avoid conflicts
    between upright/italic glyph substitutions.
    """
    logger.info("Removing GSUB tables from static fonts")
    logger.info("  (GSUB will be copied from Recursive VF in the copy-gsub step)")

    for instance in DUOTONE_INSTANCES:
        font_path = DIST_DIR / f"{WARPNINE_MONO}-{instance.style}.ttf"
        font = TTFont(str(font_path))
        if remove_gsub_table(font):
            font.save(str(font_path))
            logger.info(f"  Removed GSUB from {font_path.name}")
        font.close()


def backup_frozen_static_fonts() -> None:
    """
    Backup frozen static mono fonts before VF build.

    The VF build process modifies static fonts by removing GSUB.
    We need to restore them after the build.
    """
    logger.info("Backing up frozen static mono fonts")

    for instance in DUOTONE_INSTANCES:
        font_path = DIST_DIR / f"{WARPNINE_MONO}-{instance.style}.ttf"
        backup_path = font_path.with_suffix(".ttf.frozen")
        if font_path.exists():
            shutil.copy2(font_path, backup_path)
            logger.info(f"  Backed up {font_path.name}")


def restore_frozen_static_fonts() -> None:
    """
    Restore frozen static mono fonts after VF build.

    Replaces the GSUB-stripped static fonts with the pre-build frozen versions.
    """
    logger.info("Restoring frozen static mono fonts")

    for instance in DUOTONE_INSTANCES:
        font_path = DIST_DIR / f"{WARPNINE_MONO}-{instance.style}.ttf"
        backup_path = font_path.with_suffix(".ttf.frozen")
        if backup_path.exists():
            shutil.copy2(backup_path, font_path)
            backup_path.unlink()
            logger.info(f"  Restored {font_path.name}")


def build_variable_font() -> bool:
    """Build the variable font from static masters."""
    logger.info("Building Variable Font")

    # Check required fonts exist
    for instance in DUOTONE_INSTANCES:
        font_path = DIST_DIR / f"{WARPNINE_MONO}-{instance.style}.ttf"
        if not font_path.exists():
            logger.error(f"Required font not found: {font_path}")
            logger.error("  Run merge first")
            return False

    # Create designspace
    logger.info("Creating designspace file")
    doc = create_designspace()
    designspace_path = BUILD_DIR / f"{WARPNINE_MONO}.designspace"
    designspace_path.parent.mkdir(parents=True, exist_ok=True)
    doc.write(designspace_path)
    logger.info(f"Designspace file created: {designspace_path}")

    # Remove GSUB tables
    remove_gsub_from_static_fonts()

    # Build variable font
    output_path = DIST_DIR / f"{WARPNINE_MONO}-VF.ttf"

    try:
        run_fonttools(
            "varLib",
            [str(designspace_path), "-o", str(output_path)],
            "Building variable font",
        )

        size = output_path.stat().st_size / 1024 / 1024
        logger.info(f"Variable font created: {output_path}")
        logger.info(f"  Size: {size:.2f} MB")

        # Fix font names
        logger.info("Fixing font names")
        font = TTFont(str(output_path))
        update_vf_names(font, "Warpnine Mono", WARPNINE_MONO)
        fix_calt_registration(font)
        font.save(str(output_path))
        font.close()
        logger.info("Font names fixed")

        return True

    except Exception as e:
        logger.error(f"Error building variable font: {e}")
        return False


def copy_gsub_to_vf() -> bool:
    """
    Copy GSUB table from Recursive VF to WarpnineMono VF.

    The Recursive variable font has FeatureVariations that handle ligatures
    and contextual substitutions. We copy the entire GSUB to preserve this.
    """
    logger.info("Copying GSUB table with FeatureVariations to WarpnineMono VF...")

    source_vf = BUILD_DIR / RECURSIVE_VF
    target_vf = DIST_DIR / f"{WARPNINE_MONO}-VF.ttf"
    temp_vf = DIST_DIR / f"{WARPNINE_MONO}-VF.tmp.ttf"

    if not source_vf.exists():
        logger.error(f"Source VF not found: {source_vf}")
        logger.error("Run download step first")
        return False

    if not target_vf.exists():
        logger.error(f"Target VF not found: {target_vf}")
        logger.error("Run build step first")
        return False

    try:
        logger.info(f"Loading source VF: {source_vf}")
        source_font = TTFont(source_vf)

        logger.info(f"Loading target VF: {target_vf}")
        target_font = TTFont(target_vf)

        if "GSUB" not in source_font:
            logger.error(f"No GSUB table in source font")
            return False

        source_gsub = source_font["GSUB"]

        # Log FeatureVariations info
        if (
            hasattr(source_gsub.table, "FeatureVariations")
            and source_gsub.table.FeatureVariations
        ):
            num_variations = len(
                source_gsub.table.FeatureVariations.FeatureVariationRecord
            )
            logger.info(f"  Source has {num_variations} FeatureVariationRecords")

        # List features
        if hasattr(source_gsub.table, "FeatureList") and source_gsub.table.FeatureList:
            features = [
                rec.FeatureTag for rec in source_gsub.table.FeatureList.FeatureRecord
            ]
            unique_features = sorted(set(features))
            logger.info(f"  Features: {', '.join(unique_features)}")

        # Copy table
        target_font["GSUB"] = source_gsub

        logger.info(f"Saving to: {temp_vf}")
        target_font.save(temp_vf)

        # Replace original
        temp_vf.replace(target_vf)

        size_mb = target_vf.stat().st_size / 1024 / 1024
        logger.info("GSUB table copied successfully!")
        logger.info(f"Variable font updated: {target_vf}")
        logger.info(f"  Size: {size_mb:.2f} MB")

        return True

    except Exception as e:
        logger.error(f"Error copying GSUB: {e}")
        if temp_vf.exists():
            temp_vf.unlink()
        return False


def build() -> None:
    """Full variable font build process."""
    if not build_variable_font():
        sys.exit(1)

    logger.info("Build Complete")
    logger.info(f"Variable Font: {DIST_DIR / f'{WARPNINE_MONO}-VF.ttf'}")
    logger.info("  - wght axis: 300-1000")
    logger.info("  - ital axis: 0-1")
