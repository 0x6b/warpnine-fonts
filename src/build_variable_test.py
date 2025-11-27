#!/usr/bin/env python3
"""
Test building variable font with just 4 fonts to verify the process works
"""

import subprocess
import sys

from fontTools.designspaceLib import (
    AxisDescriptor,
    DesignSpaceDocument,
    SourceDescriptor,
)

from src.logger import logger
from src.paths import BUILD_DIR, DIST_DIR


def test_build():
    """Build variable font with just Regular and Bold to test"""
    doc = DesignSpaceDocument()

    # Weight axis
    weight_axis = AxisDescriptor()
    weight_axis.name = "Weight"
    weight_axis.tag = "wght"
    weight_axis.minimum = 400
    weight_axis.default = 400
    weight_axis.maximum = 700
    doc.addAxis(weight_axis)

    # Italic axis
    italic_axis = AxisDescriptor()
    italic_axis.name = "Italic"
    italic_axis.tag = "ital"
    italic_axis.minimum = 0
    italic_axis.default = 0
    italic_axis.maximum = 1
    doc.addAxis(italic_axis)

    # Sources - using merged fonts
    sources = [
        ("WarpnineMono-Regular.ttf", 400, 0),
        ("WarpnineMono-Bold.ttf", 700, 0),
        ("WarpnineMono-Italic.ttf", 400, 1),
        ("WarpnineMono-BoldItalic.ttf", 700, 1),
    ]

    for filename, weight, italic in sources:
        source = SourceDescriptor()
        font_path = str((DIST_DIR / filename).resolve())
        source.filename = font_path
        source.path = font_path
        source.familyName = "WarpnineMono"
        source.styleName = filename.replace("WarpnineMono-", "").replace(".ttf", "")
        source.location = {"Weight": weight, "Italic": italic}
        doc.addSource(source)

    designspace_path = BUILD_DIR / "WarpnineMono-Test.designspace"
    doc.write(designspace_path)

    logger.info(f"Building test variable font from 4 masters")
    output_path = DIST_DIR / "WarpnineMono-Test-VF.ttf"

    cmd = ["fonttools", "varLib", str(designspace_path), "-o", str(output_path)]

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        logger.info(f"SUCCESS! Variable font created: {output_path}")
        return True
    except subprocess.CalledProcessError as e:
        logger.error(f"FAILED: {e}")
        if e.stderr:
            logger.error(e.stderr[:1000])
        return False


if __name__ == "__main__":
    sys.exit(0 if test_build() else 1)
