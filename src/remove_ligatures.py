#!/usr/bin/env python3
"""
Script to remove specific ligatures from Recursive Mono Duotone fonts.

Removes the three-backtick (```) ligature from the build/RecMonoDuotone-*.ttf files.
"""

import sys
from pathlib import Path

from fontTools.ttLib import TTFont

from src.logger import logger
from src.paths import BUILD_DIR


def remove_grave_ligature(font_path: Path) -> None:
    """
    Remove the three-backtick ligature from a font.

    Args:
        font_path: Path to the font file
    """
    logger.info(f"Removing three-backtick ligature from {font_path.name}")

    font = TTFont(font_path)

    if "GSUB" not in font:
        logger.warning(f"No GSUB table found in {font_path.name}")
        font.close()
        return

    gsub = font["GSUB"].table
    modified = False

    # Remove the grave_grave_grave.code glyph substitution
    # This ligature is triggered by: grave + grave (lookahead) + grave (lookahead)
    # and substitutes: grave → grave_grave_grave.code, grave → LIG, grave → LIG

    for lookup_idx, lookup in enumerate(gsub.LookupList.Lookup):
        # Type 6 is chaining contextual substitution
        if lookup.LookupType == 6:
            for subtable_idx, subtable in enumerate(lookup.SubTable):
                # Check if this is the three-backtick pattern
                if hasattr(subtable, "Format"):
                    # Format 3: Coverage-based Chaining Context
                    if subtable.Format == 3:
                        # Check if this matches the grave pattern
                        if (
                            hasattr(subtable, "InputCoverage")
                            and len(subtable.InputCoverage) > 0
                        ):
                            first_input_glyphs = list(subtable.InputCoverage[0].glyphs)
                            # Check for lookahead graves
                            if (
                                "grave" in first_input_glyphs
                                and hasattr(subtable, "LookAheadCoverage")
                                and len(subtable.LookAheadCoverage) == 2
                            ):
                                # Check if both lookaheads are grave
                                lookahead_0 = list(subtable.LookAheadCoverage[0].glyphs)
                                lookahead_1 = list(subtable.LookAheadCoverage[1].glyphs)
                                if "grave" in lookahead_0 and "grave" in lookahead_1:
                                    logger.info(
                                        f"  Found three-backtick pattern in Lookup {lookup_idx}, Subtable {subtable_idx}"
                                    )
                                    # Remove this subtable by clearing it
                                    subtable.SubstLookupRecord = []
                                    modified = True

        # Type 1 is single substitution - remove grave → grave_grave_grave.code
        elif lookup.LookupType == 1:
            for subtable in lookup.SubTable:
                if hasattr(subtable, "mapping"):
                    if "grave" in subtable.mapping:
                        if subtable.mapping["grave"] == "grave_grave_grave.code":
                            logger.info(
                                f"  Removing grave → grave_grave_grave.code substitution in Lookup {lookup_idx}"
                            )
                            del subtable.mapping["grave"]
                            modified = True

    if modified:
        # Save the modified font
        font.save(font_path)
        logger.info(f"  Saved modified font to {font_path}")
    else:
        logger.info(f"  No three-backtick ligature found in {font_path.name}")

    font.close()


def main():
    """Remove three-backtick ligature from all Recursive Mono Duotone fonts"""
    rec_fonts = list(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))

    if not rec_fonts:
        logger.error("No Recursive Mono Duotone fonts found in build directory")
        logger.error("  Run download first")
        sys.exit(1)

    logger.info(f"Found {len(rec_fonts)} Recursive Mono Duotone fonts")

    for font_path in rec_fonts:
        remove_grave_ligature(font_path)

    logger.info("Three-backtick ligature removal completed")


if __name__ == "__main__":
    main()
