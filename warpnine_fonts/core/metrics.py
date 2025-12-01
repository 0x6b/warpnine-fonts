"""
Font metrics manipulation utilities.
"""

from fontTools.ttLib import TTFont

from warpnine_fonts.utils.logging import logger

# PANOSE value for monospaced fonts
PANOSE_MONOSPACED = 9

# Target average character width for monospace fonts
MONOSPACE_AVG_WIDTH = 600


def set_monospace_flags(font: TTFont) -> None:
    """
    Set font flags to advertise as monospaced.

    Updates:
    - post.isFixedPitch
    - OS/2.panose.bProportion
    - OS/2.xAvgCharWidth

    Args:
        font: TTFont instance to modify
    """
    # Set isFixedPitch in post table
    if "post" in font:
        font["post"].isFixedPitch = 1

    # Set PANOSE proportion to monospaced
    if "OS/2" in font:
        os2 = font["OS/2"]

        if hasattr(os2, "panose"):
            os2.panose.bProportion = PANOSE_MONOSPACED

        # Set average character width
        os2.xAvgCharWidth = MONOSPACE_AVG_WIDTH


def apply_horizontal_scale(font: TTFont, scale: float) -> None:
    """
    Apply horizontal scaling to all glyphs in a font.

    Used to create condensed variants.

    Args:
        font: TTFont instance to modify
        scale: Scale factor (e.g., 0.85 for 85% width)
    """
    from fontTools.pens.recordingPen import DecomposingRecordingPen
    from fontTools.pens.t2CharStringPen import T2CharStringPen
    from fontTools.pens.ttGlyphPen import TTGlyphPen
    from fontTools.pens.transformPen import TransformPen

    glyph_set = font.getGlyphSet()

    # Determine outline format
    if "glyf" in font:
        _scale_truetype_glyphs(font, glyph_set, scale)
    elif "CFF " in font:
        _scale_cff_glyphs(font, glyph_set, scale)

    # Scale horizontal metrics
    if "hmtx" in font:
        hmtx = font["hmtx"]
        for glyph_name in hmtx.metrics:
            width, lsb = hmtx.metrics[glyph_name]
            hmtx.metrics[glyph_name] = (int(width * scale), int(lsb * scale))

    # Scale font-wide metrics
    if "hhea" in font:
        font["hhea"].advanceWidthMax = int(font["hhea"].advanceWidthMax * scale)

    if "OS/2" in font:
        os2 = font["OS/2"]
        os2.xAvgCharWidth = int(os2.xAvgCharWidth * scale)


def _scale_truetype_glyphs(font: TTFont, glyph_set, scale: float) -> None:
    """Scale TrueType glyph outlines."""
    from fontTools.pens.recordingPen import DecomposingRecordingPen
    from fontTools.pens.ttGlyphPen import TTGlyphPen
    from fontTools.pens.transformPen import TransformPen

    glyf = font["glyf"]

    for glyph_name in glyph_set.keys():
        if glyph_name not in glyf:
            continue

        glyph = glyf[glyph_name]
        if not glyph.numberOfContours:
            continue

        # Record original paths
        rec_pen = DecomposingRecordingPen(glyph_set)
        glyph_set[glyph_name].draw(rec_pen)

        # Transform and redraw
        tt_pen = TTGlyphPen(None)
        transform_pen = TransformPen(tt_pen, (scale, 0, 0, 1, 0, 0))
        rec_pen.replay(transform_pen)

        glyf[glyph_name] = tt_pen.glyph()


def _scale_cff_glyphs(font: TTFont, glyph_set, scale: float) -> None:
    """Scale CFF glyph outlines."""
    from fontTools.pens.recordingPen import DecomposingRecordingPen
    from fontTools.pens.t2CharStringPen import T2CharStringPen
    from fontTools.pens.transformPen import TransformPen

    cff = font["CFF "]
    top_dict = cff.cff.topDictIndex[0]
    char_strings = top_dict.CharStrings

    for glyph_name in glyph_set.keys():
        if glyph_name not in char_strings:
            continue

        # Record original paths
        rec_pen = DecomposingRecordingPen(glyph_set)
        glyph_set[glyph_name].draw(rec_pen)

        # Transform and redraw
        t2_pen = T2CharStringPen(0, None)
        transform_pen = TransformPen(t2_pen, (scale, 0, 0, 1, 0, 0))
        rec_pen.replay(transform_pen)

        char_strings[glyph_name] = t2_pen.getCharString()
