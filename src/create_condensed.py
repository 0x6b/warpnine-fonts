#!/usr/bin/env python3
"""
Create a condensed variant of Recursive Sans Linear Static.

This script:
1. Extracts a static instance from Recursive VF (Sans=MONO=0, Linear=CASL=0)
2. Applies horizontal scaling to create a condensed variant
3. Adjusts advance widths and metrics proportionally
4. Updates font naming for the condensed variant
"""

import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from fontTools.misc.transform import Transform
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont

from src.logger import logger
from src.paths import BUILD_DIR, DIST_DIR


@dataclass(frozen=True)
class CondensedConfig:
    """Configuration for a condensed variant."""

    style: str
    output_name: str
    # Axis values for the base extraction
    mono: float  # 0 = Sans (proportional), 1 = Mono
    casl: float  # 0 = Linear, 1 = Casual
    wght: float
    slnt: float
    crsv: float
    # Condensed factor (1.0 = normal, 0.8 = 80% width)
    width_factor: float


# Condensed instances to create
# Recursive Sans Linear Static - condensed to 85% width
INSTANCES = [
    # Light
    CondensedConfig(
        style="Light",
        output_name="WarpnineSansCondensed-Light.ttf",
        mono=0.0,
        casl=0.0,
        wght=300.0,
        slnt=0.0,
        crsv=0.5,
        width_factor=0.85,
    ),
    CondensedConfig(
        style="LightItalic",
        output_name="WarpnineSansCondensed-LightItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=300.0,
        slnt=-15.0,
        crsv=1.0,
        width_factor=0.85,
    ),
    # Regular
    CondensedConfig(
        style="Regular",
        output_name="WarpnineSansCondensed-Regular.ttf",
        mono=0.0,
        casl=0.0,
        wght=400.0,
        slnt=0.0,
        crsv=0.5,
        width_factor=0.85,
    ),
    CondensedConfig(
        style="Italic",
        output_name="WarpnineSansCondensed-Italic.ttf",
        mono=0.0,
        casl=0.0,
        wght=400.0,
        slnt=-15.0,
        crsv=1.0,
        width_factor=0.85,
    ),
    # Medium
    CondensedConfig(
        style="Medium",
        output_name="WarpnineSansCondensed-Medium.ttf",
        mono=0.0,
        casl=0.0,
        wght=500.0,
        slnt=0.0,
        crsv=0.5,
        width_factor=0.85,
    ),
    CondensedConfig(
        style="MediumItalic",
        output_name="WarpnineSansCondensed-MediumItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=500.0,
        slnt=-15.0,
        crsv=1.0,
        width_factor=0.85,
    ),
    # SemiBold
    CondensedConfig(
        style="SemiBold",
        output_name="WarpnineSansCondensed-SemiBold.ttf",
        mono=0.0,
        casl=0.0,
        wght=600.0,
        slnt=0.0,
        crsv=0.5,
        width_factor=0.85,
    ),
    CondensedConfig(
        style="SemiBoldItalic",
        output_name="WarpnineSansCondensed-SemiBoldItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=600.0,
        slnt=-15.0,
        crsv=1.0,
        width_factor=0.85,
    ),
    # Bold
    CondensedConfig(
        style="Bold",
        output_name="WarpnineSansCondensed-Bold.ttf",
        mono=0.0,
        casl=0.0,
        wght=700.0,
        slnt=0.0,
        crsv=0.5,
        width_factor=0.85,
    ),
    CondensedConfig(
        style="BoldItalic",
        output_name="WarpnineSansCondensed-BoldItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=700.0,
        slnt=-15.0,
        crsv=1.0,
        width_factor=0.85,
    ),
    # ExtraBold
    CondensedConfig(
        style="ExtraBold",
        output_name="WarpnineSansCondensed-ExtraBold.ttf",
        mono=0.0,
        casl=0.0,
        wght=800.0,
        slnt=0.0,
        crsv=0.5,
        width_factor=0.85,
    ),
    CondensedConfig(
        style="ExtraBoldItalic",
        output_name="WarpnineSansCondensed-ExtraBoldItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=800.0,
        slnt=-15.0,
        crsv=1.0,
        width_factor=0.85,
    ),
    # Black
    CondensedConfig(
        style="Black",
        output_name="WarpnineSansCondensed-Black.ttf",
        mono=0.0,
        casl=0.0,
        wght=900.0,
        slnt=0.0,
        crsv=0.5,
        width_factor=0.85,
    ),
    CondensedConfig(
        style="BlackItalic",
        output_name="WarpnineSansCondensed-BlackItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=900.0,
        slnt=-15.0,
        crsv=1.0,
        width_factor=0.85,
    ),
]


def extract_static_instance(
    variable_font: Path, config: CondensedConfig, output: Path
) -> None:
    """Extract a static instance from the Variable Font."""
    cmd = [
        "fonttools",
        "varLib.instancer",
        str(variable_font),
        f"MONO={config.mono}",
        f"CASL={config.casl}",
        f"wght={config.wght}",
        f"slnt={config.slnt}",
        f"CRSV={config.crsv}",
        "-o",
        str(output),
    ]
    result = subprocess.run(cmd, check=True, capture_output=True, text=True)
    if result.stderr:
        logger.warning(result.stderr)


def apply_horizontal_scale_ttf(font: TTFont, scale_x: float) -> None:
    """
    Apply horizontal scaling to all glyphs in a TrueType font.

    Handles composite glyphs correctly by only transforming simple glyphs
    and updating composite glyph component offsets.
    """
    if "glyf" not in font:
        raise ValueError("Font does not contain 'glyf' table (not TrueType)")

    glyf_table = font["glyf"]
    glyph_set = font.getGlyphSet()
    transform = Transform(scale_x, 0, 0, 1, 0, 0)

    for glyph_name in list(glyph_set.keys()):
        glyph = glyf_table[glyph_name]

        if glyph.numberOfContours == 0:
            # Empty glyph (space, etc.)
            continue

        if glyph.isComposite():
            # For composite glyphs, scale the component offsets
            for component in glyph.components:
                if hasattr(component, "x"):
                    component.x = int(component.x * scale_x)
            # Update bounding box
            glyph.recalcBounds(glyf_table)
        else:
            # Simple glyph: apply transform via pen
            recording_pen = DecomposingRecordingPen(glyph_set)
            transform_pen = TransformPen(recording_pen, transform)
            glyph_set[glyph_name].draw(transform_pen)

            tt_pen = TTGlyphPen(None)
            recording_pen.replay(tt_pen)
            new_glyph = tt_pen.glyph()
            # Preserve instructions if present
            if hasattr(glyph, "program"):
                new_glyph.program = glyph.program
            glyf_table[glyph_name] = new_glyph


def apply_horizontal_scale_cff(font: TTFont, scale_x: float) -> None:
    """
    Apply horizontal scaling to all glyphs in a CFF font.
    """
    if "CFF " not in font:
        raise ValueError("Font does not contain 'CFF ' table")

    cff = font["CFF "]
    top_dict = cff.cff.topDictIndex[0]
    char_strings = top_dict.CharStrings
    glyph_set = font.getGlyphSet()
    transform = Transform(scale_x, 0, 0, 1, 0, 0)

    for glyph_name in list(char_strings.keys()):
        recording_pen = DecomposingRecordingPen(glyph_set)
        transform_pen = TransformPen(recording_pen, transform)
        glyph_set[glyph_name].draw(transform_pen)

        t2_pen = T2CharStringPen(width=None, glyphSet=glyph_set)
        recording_pen.replay(t2_pen)
        char_strings[glyph_name] = t2_pen.getCharString()


def scale_horizontal_metrics(font: TTFont, scale_x: float) -> None:
    """Scale horizontal metrics (advance widths and LSB)."""
    hmtx = font["hmtx"]
    for glyph_name in hmtx.metrics:
        width, lsb = hmtx.metrics[glyph_name]
        hmtx.metrics[glyph_name] = (int(width * scale_x), int(lsb * scale_x))


def scale_font_wide_metrics(font: TTFont, scale_x: float) -> None:
    """Scale font-wide horizontal metrics in head, hhea, OS/2 tables."""
    # head table
    if "head" in font:
        head = font["head"]
        head.xMin = int(head.xMin * scale_x)
        head.xMax = int(head.xMax * scale_x)

    # hhea table
    if "hhea" in font:
        hhea = font["hhea"]
        hhea.advanceWidthMax = int(hhea.advanceWidthMax * scale_x)
        hhea.minLeftSideBearing = int(hhea.minLeftSideBearing * scale_x)
        hhea.minRightSideBearing = int(hhea.minRightSideBearing * scale_x)
        hhea.xMaxExtent = int(hhea.xMaxExtent * scale_x)

    # OS/2 table
    if "OS/2" in font:
        os2 = font["OS/2"]
        os2.xAvgCharWidth = int(os2.xAvgCharWidth * scale_x)


def update_name_table(font: TTFont, style: str) -> None:
    """Update name table for Warpnine Sans Condensed."""
    name_table = font["name"]

    # Name IDs to update
    # 1: Family name
    # 2: Subfamily name
    # 4: Full name
    # 6: PostScript name
    # 16: Typographic Family name
    # 17: Typographic Subfamily name

    for record in name_table.names:
        try:
            text = record.toUnicode()
        except UnicodeDecodeError:
            continue

        # Update family name
        if record.nameID == 1:
            record.string = f"Warpnine Sans Condensed {style}"

        # Update full name
        elif record.nameID == 4:
            record.string = f"Warpnine Sans Condensed {style}"

        # Update PostScript name
        elif record.nameID == 6:
            # PostScript names cannot have spaces
            record.string = f"WarpnineSansCondensed-{style}"

        # Update typographic family name
        elif record.nameID == 16:
            record.string = "Warpnine Sans Condensed"

        # Update typographic subfamily name
        elif record.nameID == 17:
            record.string = style


def create_condensed(
    variable_font: Path, config: CondensedConfig, output: Path
) -> None:
    """Create a condensed variant from the variable font."""
    # Step 1: Extract static instance to a temp file
    temp_static = output.with_suffix(".temp.ttf")
    logger.info(f"Extracting {config.style} instance")
    extract_static_instance(variable_font, config, temp_static)

    # Step 2: Load and transform
    logger.info(f"Applying horizontal scale: {config.width_factor}")
    font = TTFont(temp_static)

    # Determine font type and apply appropriate scaling
    if "glyf" in font:
        apply_horizontal_scale_ttf(font, config.width_factor)
    elif "CFF " in font:
        apply_horizontal_scale_cff(font, config.width_factor)
    else:
        raise ValueError("Unknown font format (neither TrueType nor CFF)")

    # Step 3: Scale metrics
    scale_horizontal_metrics(font, config.width_factor)
    scale_font_wide_metrics(font, config.width_factor)

    # Step 4: Update names
    update_name_table(font, config.style)

    # Step 5: Save
    font.save(output)
    font.close()

    # Cleanup temp file
    temp_static.unlink()
    logger.info(f"Created: {output}")


def main():
    input_font = BUILD_DIR / "Recursive_VF_1.085.ttf"
    output_dir = DIST_DIR

    if not input_font.exists():
        logger.error(f"Input file not found: {input_font}")
        logger.error("Run 'uv run download' first")
        sys.exit(1)

    output_dir.mkdir(parents=True, exist_ok=True)

    for config in INSTANCES:
        output = output_dir / config.output_name
        logger.info(f"Creating {config.style} condensed ({config.width_factor:.0%})")
        try:
            create_condensed(input_font, config, output)
        except Exception as e:
            logger.error(f"Failed to create {config.style}: {e}")
            raise

    logger.info(f"All condensed instances created in {output_dir}/")


if __name__ == "__main__":
    main()
