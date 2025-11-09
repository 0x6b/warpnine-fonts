#!/usr/bin/env python3
"""
Script to generate Variable Font

Generate one Variable Font from 4 styles in dist/
- wght axis: 400 (Regular) - 700 (Bold)
- ital axis: 0 (Upright) - 1 (Italic)
"""

import subprocess
import sys
from dataclasses import dataclass

from fontTools.designspaceLib import (
    AxisDescriptor,
    DesignSpaceDocument,
    InstanceDescriptor,
    SourceDescriptor,
)
from fontTools.ttLib import TTFont

from src.logger import logger
from src.merge import fix_calt_registration
from src.paths import BUILD_DIR, DIST_DIR


@dataclass
class SourceInfo:
    filename: str
    weight: int
    italic: int
    is_copy_info: bool = False


@dataclass
class InstanceInfo:
    style_name: str
    weight: int
    italic: int


def create_designspace():
    """
    Generate designspace file
    """
    doc = DesignSpaceDocument()

    # Axis definitions
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

    # Source fonts (masters) definition - all weights
    sources = [
        SourceInfo("WarpnineMono-Light.ttf", 300, 0, False),
        SourceInfo("WarpnineMono-Regular.ttf", 400, 0, False),
        SourceInfo("WarpnineMono-Medium.ttf", 500, 0, False),
        SourceInfo("WarpnineMono-SemiBold.ttf", 600, 0, False),
        SourceInfo("WarpnineMono-Bold.ttf", 700, 0, False),
        SourceInfo("WarpnineMono-ExtraBold.ttf", 800, 0, False),
        SourceInfo("WarpnineMono-Black.ttf", 900, 0, False),
        SourceInfo("WarpnineMono-ExtraBlack.ttf", 1000, 0, False),
        SourceInfo("WarpnineMono-LightItalic.ttf", 300, 1, False),
        SourceInfo("WarpnineMono-Italic.ttf", 400, 1, False),
        SourceInfo("WarpnineMono-MediumItalic.ttf", 500, 1, False),
        SourceInfo("WarpnineMono-SemiBoldItalic.ttf", 600, 1, False),
        SourceInfo("WarpnineMono-BoldItalic.ttf", 700, 1, False),
        SourceInfo("WarpnineMono-ExtraBoldItalic.ttf", 800, 1, False),
        SourceInfo("WarpnineMono-BlackItalic.ttf", 900, 1, False),
        SourceInfo("WarpnineMono-ExtraBlackItalic.ttf", 1000, 1, False),
    ]

    for source_info in sources:
        source = SourceDescriptor()
        # Use absolute paths
        font_path = str((DIST_DIR / source_info.filename).resolve())
        source.filename = font_path
        source.path = font_path
        source.familyName = "WarpnineMono"
        source.styleName = source_info.filename.replace("WarpnineMono-", "").replace(
            ".ttf", ""
        )
        source.location = {"Weight": source_info.weight, "Italic": source_info.italic}
        if source_info.is_copy_info:
            source.copyInfo = True
        doc.addSource(source)

    # Instance definitions (individual styles to generate)
    instances = [
        InstanceInfo("Light", 300, 0),
        InstanceInfo("Regular", 400, 0),
        InstanceInfo("Medium", 500, 0),
        InstanceInfo("SemiBold", 600, 0),
        InstanceInfo("Bold", 700, 0),
        InstanceInfo("ExtraBold", 800, 0),
        InstanceInfo("Black", 900, 0),
        InstanceInfo("ExtraBlack", 1000, 0),
        InstanceInfo("Light Italic", 300, 1),
        InstanceInfo("Italic", 400, 1),
        InstanceInfo("Medium Italic", 500, 1),
        InstanceInfo("SemiBold Italic", 600, 1),
        InstanceInfo("Bold Italic", 700, 1),
        InstanceInfo("ExtraBold Italic", 800, 1),
        InstanceInfo("Black Italic", 900, 1),
        InstanceInfo("ExtraBlack Italic", 1000, 1),
    ]

    for instance_info in instances:
        instance = InstanceDescriptor()
        instance.familyName = "WarpnineMono"
        instance.styleName = instance_info.style_name
        instance.filename = str(
            DIST_DIR / f"instance_{instance_info.style_name.replace(' ', '')}.ttf"
        )
        instance.location = {
            "Weight": instance_info.weight,
            "Italic": instance_info.italic,
        }
        doc.addInstance(instance)

    return doc


def fix_font_names(font_path):
    """
    Fix font name table to set name to Warpnine Mono
    """
    logger.info("Fixing font names")
    font = TTFont(str(font_path))

    # Get name table
    name_table = font["name"]

    # Mapping of names to fix
    # nameID descriptions:
    # 1: Font Family name
    # 3: Unique font identifier
    # 4: Full font name
    # 6: PostScript name
    # 16: Typographic Family name (preferred)
    # 17: Typographic Subfamily name

    for record in name_table.names:
        # nameID 0 (Copyright) - Include copyright of all source fonts
        if record.nameID == 0:
            copyright_text = (
                "Copyright 2020 The Recursive Project Authors (https://github.com/arrowtype/recursive). "
                "Copyright 2014-2021 Adobe (http://www.adobe.com/), with Reserved Font Name 'Source'. "
                "Warpnine Mono is based on Recursive Mono Duotone and Noto Sans Mono CJK JP."
            )
            record.string = copyright_text

        # nameID 1 (Font Family) - Basic family name
        elif record.nameID == 1:
            record.string = "Warpnine Mono"

        # nameID 3 (Unique ID) - Unique identifier including version info
        elif record.nameID == 3:
            record.string = "1.0;WARPNINE;WarpnineMono"

        # nameID 4 (Full Name)
        elif record.nameID == 4:
            record.string = "Warpnine Mono"

        # nameID 6 (PostScript Name) - Variable Font does not include style info
        elif record.nameID == 6:
            record.string = "WarpnineMono"

        # nameID 16 (Typographic Family) - This is displayed in Font Book
        elif record.nameID == 16:
            record.string = "Warpnine Mono"

        # nameID 17 (Typographic Subfamily)
        elif record.nameID == 17:
            record.string = "Regular"

    font.save(str(font_path))
    font.close()
    logger.info("Font names fixed")


def remove_gsub_tables():
    """
    Remove GSUB tables from all fonts to avoid varLib incompatibility.

    After feature freezing, GSUB tables still contain FeatureVariations that cause
    conflicts when building a variable font (.mono vs .italic glyph substitutions).
    We'll copy the GSUB table from the original Recursive VF in the copy-gsub step.
    """
    logger.info("Removing GSUB tables from static fonts")
    logger.info("  (GSUB will be copied from Recursive VF in the copy-gsub step)")

    font_files = [
        DIST_DIR / "WarpnineMono-Light.ttf",
        DIST_DIR / "WarpnineMono-Regular.ttf",
        DIST_DIR / "WarpnineMono-Medium.ttf",
        DIST_DIR / "WarpnineMono-SemiBold.ttf",
        DIST_DIR / "WarpnineMono-Bold.ttf",
        DIST_DIR / "WarpnineMono-ExtraBold.ttf",
        DIST_DIR / "WarpnineMono-Black.ttf",
        DIST_DIR / "WarpnineMono-ExtraBlack.ttf",
        DIST_DIR / "WarpnineMono-LightItalic.ttf",
        DIST_DIR / "WarpnineMono-Italic.ttf",
        DIST_DIR / "WarpnineMono-MediumItalic.ttf",
        DIST_DIR / "WarpnineMono-SemiBoldItalic.ttf",
        DIST_DIR / "WarpnineMono-BoldItalic.ttf",
        DIST_DIR / "WarpnineMono-ExtraBoldItalic.ttf",
        DIST_DIR / "WarpnineMono-BlackItalic.ttf",
        DIST_DIR / "WarpnineMono-ExtraBlackItalic.ttf",
    ]

    for font_path in font_files:
        font = TTFont(str(font_path))
        if "GSUB" in font:
            del font["GSUB"]
            font.save(str(font_path))
            logger.info(f"  Removed GSUB from {font_path.name}")
        font.close()


def build_variable_font():
    """
    Build Variable Font
    """
    logger.info("Building Variable Font")

    # Create designspace file
    logger.info("Creating designspace file")
    doc = create_designspace()
    designspace_path = BUILD_DIR / "WarpnineMono.designspace"
    designspace_path.parent.mkdir(parents=True, exist_ok=True)
    doc.write(designspace_path)
    logger.info(f"Designspace file created: {designspace_path}")

    # Build Variable Font
    logger.info("Building variable font")
    output_path = DIST_DIR / "WarpnineMono-VF.ttf"

    cmd = [
        "fonttools",
        "varLib",
        str(designspace_path),
        "-o",
        str(output_path),
    ]

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        logger.info(f"Variable font created: {output_path}")

        # Display file size
        size = output_path.stat().st_size / 1024 / 1024  # MB
        logger.info(f"  Size: {size:.2f} MB")

        if result.stdout:
            logger.info(result.stdout)

        # Fix font names
        fix_font_names(output_path)

        # Register calt feature to all scripts (for browser compatibility)
        logger.info("Fixing calt registration")
        font = TTFont(str(output_path))
        fix_calt_registration(font)
        font.save(str(output_path))
        font.close()
        logger.info("calt registration fixed")

        return True
    except subprocess.CalledProcessError as e:
        logger.error(f"Error building variable font: {e}")
        if e.stderr:
            logger.error(e.stderr)
        return False


def main():
    # Check existence of required font files
    required_fonts = [
        DIST_DIR / "WarpnineMono-Light.ttf",
        DIST_DIR / "WarpnineMono-Regular.ttf",
        DIST_DIR / "WarpnineMono-Medium.ttf",
        DIST_DIR / "WarpnineMono-SemiBold.ttf",
        DIST_DIR / "WarpnineMono-Bold.ttf",
        DIST_DIR / "WarpnineMono-ExtraBold.ttf",
        DIST_DIR / "WarpnineMono-Black.ttf",
        DIST_DIR / "WarpnineMono-ExtraBlack.ttf",
        DIST_DIR / "WarpnineMono-LightItalic.ttf",
        DIST_DIR / "WarpnineMono-Italic.ttf",
        DIST_DIR / "WarpnineMono-MediumItalic.ttf",
        DIST_DIR / "WarpnineMono-SemiBoldItalic.ttf",
        DIST_DIR / "WarpnineMono-BoldItalic.ttf",
        DIST_DIR / "WarpnineMono-ExtraBoldItalic.ttf",
        DIST_DIR / "WarpnineMono-BlackItalic.ttf",
        DIST_DIR / "WarpnineMono-ExtraBlackItalic.ttf",
    ]

    for font_path in required_fonts:
        if not font_path.exists():
            logger.error(f"Required font not found: {font_path}")
            logger.error("  Run merge first")
            sys.exit(1)

    # Remove GSUB tables to avoid varLib incompatibility
    remove_gsub_tables()

    # Build Variable Font
    if build_variable_font():
        logger.info("Build Complete")
        logger.info(f"Variable Font: {DIST_DIR / 'WarpnineMono-VF.ttf'}")
        logger.info("  - wght axis: 300-1000")
        logger.info("  - ital axis: 0-1")
    else:
        sys.exit(1)


if __name__ == "__main__":
    main()
