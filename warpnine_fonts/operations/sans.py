"""
Sans font creation operations.

Creates WarpnineSans and WarpnineSansCondensed from Recursive VF.
"""

import sys
from pathlib import Path

from fontTools.misc.transform import Transform
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont

from warpnine_fonts.config.instances import SANS_INSTANCES
from warpnine_fonts.config.paths import BUILD_DIR, DIST_DIR, RECURSIVE_VF
from warpnine_fonts.core.instancer import extract_recursive_instance
from warpnine_fonts.utils.logging import logger

# Condensed width factor
CONDENSED_WIDTH = 0.85


def update_sans_name_table(font: TTFont, family: str, style: str) -> None:
    """Update name table for a sans font."""
    name_table = font["name"]

    postscript_family = family.replace(" ", "")

    for record in name_table.names:
        try:
            record.toUnicode()
        except UnicodeDecodeError:
            continue

        if record.nameID == 1:
            record.string = f"{family} {style}"
        elif record.nameID == 4:
            record.string = f"{family} {style}"
        elif record.nameID == 6:
            record.string = f"{postscript_family}-{style}"
        elif record.nameID == 16:
            record.string = family
        elif record.nameID == 17:
            record.string = style


def apply_horizontal_scale_ttf(font: TTFont, scale_x: float) -> None:
    """Apply horizontal scaling to TrueType glyphs."""
    if "glyf" not in font:
        raise ValueError("Font does not contain 'glyf' table")

    glyf_table = font["glyf"]
    glyph_set = font.getGlyphSet()
    transform = Transform(scale_x, 0, 0, 1, 0, 0)

    for glyph_name in list(glyph_set.keys()):
        glyph = glyf_table[glyph_name]

        if glyph.numberOfContours == 0:
            continue

        if glyph.isComposite():
            for component in glyph.components:
                if hasattr(component, "x"):
                    component.x = int(component.x * scale_x)
            glyph.recalcBounds(glyf_table)
        else:
            recording_pen = DecomposingRecordingPen(glyph_set)
            transform_pen = TransformPen(recording_pen, transform)
            glyph_set[glyph_name].draw(transform_pen)

            tt_pen = TTGlyphPen(None)
            recording_pen.replay(tt_pen)
            new_glyph = tt_pen.glyph()
            if hasattr(glyph, "program"):
                new_glyph.program = glyph.program
            glyf_table[glyph_name] = new_glyph


def apply_horizontal_scale_cff(font: TTFont, scale_x: float) -> None:
    """Apply horizontal scaling to CFF glyphs."""
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
    """Scale horizontal metrics."""
    hmtx = font["hmtx"]
    for glyph_name in hmtx.metrics:
        width, lsb = hmtx.metrics[glyph_name]
        hmtx.metrics[glyph_name] = (int(width * scale_x), int(lsb * scale_x))


def scale_font_wide_metrics(font: TTFont, scale_x: float) -> None:
    """Scale font-wide horizontal metrics."""
    if "head" in font:
        head = font["head"]
        head.xMin = int(head.xMin * scale_x)
        head.xMax = int(head.xMax * scale_x)

    if "hhea" in font:
        hhea = font["hhea"]
        hhea.advanceWidthMax = int(hhea.advanceWidthMax * scale_x)
        hhea.minLeftSideBearing = int(hhea.minLeftSideBearing * scale_x)
        hhea.minRightSideBearing = int(hhea.minRightSideBearing * scale_x)
        hhea.xMaxExtent = int(hhea.xMaxExtent * scale_x)

    if "OS/2" in font:
        os2 = font["OS/2"]
        os2.xAvgCharWidth = int(os2.xAvgCharWidth * scale_x)


def create_sans() -> None:
    """Create WarpnineSans fonts from Recursive VF."""
    input_font = BUILD_DIR / RECURSIVE_VF
    output_dir = DIST_DIR

    if not input_font.exists():
        logger.error(f"Input file not found: {input_font}")
        logger.error("Run download first")
        sys.exit(1)

    output_dir.mkdir(parents=True, exist_ok=True)

    for instance in SANS_INSTANCES:
        output = output_dir / f"WarpnineSans-{instance.style}.ttf"
        temp_static = output.with_suffix(".temp.ttf")

        logger.info(f"Creating {instance.style}")

        # Extract instance
        extract_recursive_instance(
            input_font,
            temp_static,
            mono=instance.mono,
            casl=instance.casl,
            wght=instance.wght,
            slnt=instance.slnt,
            crsv=instance.crsv,
        )

        # Update names
        font = TTFont(temp_static)
        update_sans_name_table(font, "Warpnine Sans", instance.style)
        font.save(output)
        font.close()

        temp_static.unlink()
        logger.info(f"Created: {output}")

    logger.info(f"All sans instances created in {output_dir}/")


def create_condensed() -> None:
    """Create WarpnineSansCondensed fonts from Recursive VF."""
    input_font = BUILD_DIR / RECURSIVE_VF
    output_dir = DIST_DIR

    if not input_font.exists():
        logger.error(f"Input file not found: {input_font}")
        logger.error("Run download first")
        sys.exit(1)

    output_dir.mkdir(parents=True, exist_ok=True)

    for instance in SANS_INSTANCES:
        output = output_dir / f"WarpnineSansCondensed-{instance.style}.ttf"
        temp_static = output.with_suffix(".temp.ttf")

        logger.info(f"Creating {instance.style} condensed ({CONDENSED_WIDTH:.0%})")

        # Extract instance
        extract_recursive_instance(
            input_font,
            temp_static,
            mono=instance.mono,
            casl=instance.casl,
            wght=instance.wght,
            slnt=instance.slnt,
            crsv=instance.crsv,
        )

        # Load and transform
        font = TTFont(temp_static)

        # Apply horizontal scaling
        if "glyf" in font:
            apply_horizontal_scale_ttf(font, CONDENSED_WIDTH)
        elif "CFF " in font:
            apply_horizontal_scale_cff(font, CONDENSED_WIDTH)
        else:
            raise ValueError("Unknown font format")

        # Scale metrics
        scale_horizontal_metrics(font, CONDENSED_WIDTH)
        scale_font_wide_metrics(font, CONDENSED_WIDTH)

        # Set width class to Condensed (3)
        font["OS/2"].usWidthClass = 3

        # Update names
        update_sans_name_table(font, "Warpnine Sans Condensed", instance.style)

        font.save(output)
        font.close()

        temp_static.unlink()
        logger.info(f"Created: {output}")

    logger.info(f"All condensed instances created in {output_dir}/")
