"""
Font subsetting operations.

Subsets Noto CJK fonts to Japanese Unicode ranges.
"""

import sys

from warpnine_fonts.config.paths import BUILD_DIR
from warpnine_fonts.config.unicode_ranges import JAPANESE_RANGES, VF_TABLES_TO_DROP
from warpnine_fonts.core.font_io import iter_fonts
from warpnine_fonts.utils.logging import logger
from warpnine_fonts.utils.subprocess import run_pyftsubset


def subset_noto_fonts() -> None:
    """
    Subset Noto CJK fonts to Japanese Unicode ranges.

    Processes all Noto-*.ttf files in the build directory (excluding already subsetted files)
    and creates *-subset.ttf versions with only Japanese characters.
    """
    input_fonts = list(
        iter_fonts(BUILD_DIR, "Noto-*.ttf", exclude_patterns=["-subset"])
    )

    if not input_fonts:
        logger.error(f"Input files not found: {BUILD_DIR / 'Noto-*.ttf'}")
        logger.error("  Run extract-weights first")
        sys.exit(1)

    unicodes = ",".join(JAPANESE_RANGES)
    logger.info(f"Subsetting {len(input_fonts)} Noto fonts")
    logger.info(f"  Unicode ranges: {len(JAPANESE_RANGES)} ranges")

    for input_font in input_fonts:
        output = BUILD_DIR / f"{input_font.stem}-subset.ttf"
        output.parent.mkdir(parents=True, exist_ok=True)

        run_pyftsubset(
            input_font,
            output,
            unicodes,
            keep_layout=True,
            drop_tables=VF_TABLES_TO_DROP,
        )
        logger.info(f"Created {output.name}")

    logger.info("All fonts subsetted")
