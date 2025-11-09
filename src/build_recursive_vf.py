#!/usr/bin/env python3
"""
Script to build variable font from Recursive Casual instances only (no Noto merge)

This creates a test variable font to verify the FeatureVariations approach works
"""

import subprocess
import sys
from dataclasses import dataclass

from fontTools.designspaceLib import (
    AxisDescriptor,
    DesignSpaceDocument,
    SourceDescriptor,
)

from src.logger import logger
from src.paths import BUILD_DIR


@dataclass
class SourceInfo:
    filename: str
    weight: int
    italic: int


def create_designspace():
    """Generate designspace file for Recursive Casual only"""
    doc = DesignSpaceDocument()

    # Weight axis: 300-1000
    weight_axis = AxisDescriptor()
    weight_axis.name = "Weight"
    weight_axis.tag = "wght"
    weight_axis.minimum = 300
    weight_axis.default = 400
    weight_axis.maximum = 1000
    doc.addAxis(weight_axis)

    # Italic axis: 0-1
    italic_axis = AxisDescriptor()
    italic_axis.name = "Italic"
    italic_axis.tag = "ital"
    italic_axis.minimum = 0
    italic_axis.default = 0
    italic_axis.maximum = 1
    doc.addAxis(italic_axis)

    # Source fonts (all Casual)
    sources = [
        SourceInfo("RecMonoDuotone-Light.ttf", 300, 0),
        SourceInfo("RecMonoDuotone-Regular.ttf", 400, 0),
        SourceInfo("RecMonoDuotone-Medium.ttf", 500, 0),
        SourceInfo("RecMonoDuotone-SemiBold.ttf", 600, 0),
        SourceInfo("RecMonoDuotone-Bold.ttf", 700, 0),
        SourceInfo("RecMonoDuotone-ExtraBold.ttf", 800, 0),
        SourceInfo("RecMonoDuotone-Black.ttf", 900, 0),
        SourceInfo("RecMonoDuotone-ExtraBlack.ttf", 1000, 0),
        SourceInfo("RecMonoDuotone-LightItalic.ttf", 300, 1),
        SourceInfo("RecMonoDuotone-Italic.ttf", 400, 1),
        SourceInfo("RecMonoDuotone-MediumItalic.ttf", 500, 1),
        SourceInfo("RecMonoDuotone-SemiBoldItalic.ttf", 600, 1),
        SourceInfo("RecMonoDuotone-BoldItalic.ttf", 700, 1),
        SourceInfo("RecMonoDuotone-ExtraBoldItalic.ttf", 800, 1),
        SourceInfo("RecMonoDuotone-BlackItalic.ttf", 900, 1),
        SourceInfo("RecMonoDuotone-ExtraBlackItalic.ttf", 1000, 1),
    ]

    for source_info in sources:
        source = SourceDescriptor()
        font_path = str((BUILD_DIR / source_info.filename).resolve())
        source.filename = font_path
        source.path = font_path
        source.familyName = "RecursiveMono"
        source.styleName = source_info.filename.replace("RecMonoDuotone-", "").replace(".ttf", "")
        source.location = {"Weight": source_info.weight, "Italic": source_info.italic}
        doc.addSource(source)

    return doc


def build_variable_font():
    """Build variable font from Recursive Casual instances"""
    logger.info("Building Recursive Casual variable font")

    # Create designspace
    logger.info("Creating designspace file")
    doc = create_designspace()
    designspace_path = BUILD_DIR / "RecursiveMono-Casual.designspace"
    doc.write(designspace_path)
    logger.info(f"Designspace file created: {designspace_path}")

    # Build variable font
    logger.info("Building variable font")
    output_path = BUILD_DIR / "RecursiveMono-Casual-VF.ttf"

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

        size = output_path.stat().st_size / 1024 / 1024  # MB
        logger.info(f"  Size: {size:.2f} MB")

        if result.stdout:
            logger.info(result.stdout)

        return True
    except subprocess.CalledProcessError as e:
        logger.error(f"Error building variable font: {e}")
        if e.stderr:
            logger.error(e.stderr)
        return False


def main():
    # Check existence of required fonts
    required_fonts = [
        BUILD_DIR / "RecMonoDuotone-Light.ttf",
        BUILD_DIR / "RecMonoDuotone-Regular.ttf",
    ]

    for font_path in required_fonts:
        if not font_path.exists():
            logger.error(f"Required font not found: {font_path}")
            logger.error("  Run extract_duotone first")
            sys.exit(1)

    # Build variable font
    if build_variable_font():
        logger.info("Build Complete")
        logger.info(f"Variable Font: {BUILD_DIR / 'RecursiveMono-Casual-VF.ttf'}")
        logger.info("  - wght axis: 300-1000")
        logger.info("  - ital axis: 0-1")
    else:
        sys.exit(1)


if __name__ == "__main__":
    main()
