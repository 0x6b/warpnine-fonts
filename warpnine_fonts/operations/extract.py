"""
Font instance extraction operations.

Extracts static instances from Recursive VF and Noto CJK VF.
"""

import sys
from pathlib import Path

from warpnine_fonts.config.instances import DUOTONE_INSTANCES, NotoWeight
from warpnine_fonts.config.paths import BUILD_DIR, RECURSIVE_VF
from warpnine_fonts.core.instancer import (
    extract_noto_instance,
    extract_recursive_instance,
)
from warpnine_fonts.utils.logging import logger


def extract_duotone() -> None:
    """
    Extract all Duotone instances from Recursive Variable Font.

    Creates 16 static fonts (8 weights x 2 styles) in the build directory.
    """
    input_font = BUILD_DIR / RECURSIVE_VF
    output_dir = BUILD_DIR

    if not input_font.exists():
        logger.error(f"Input file not found: {input_font}")
        logger.error("  Run download first")
        sys.exit(1)

    for instance in DUOTONE_INSTANCES:
        output = output_dir / instance.output_name

        logger.info(f"Extracting {instance.style}")
        extract_recursive_instance(
            input_font,
            output,
            mono=instance.mono,
            casl=instance.casl,
            wght=instance.wght,
            slnt=instance.slnt,
            crsv=instance.crsv,
        )
        logger.info(f"Successfully extracted {instance.style}")

    logger.info("All Duotone instances extracted")


def extract_noto_weights() -> None:
    """
    Extract weight instances from Noto Sans Mono CJK JP Variable Font.

    Extracts 400 (Regular) and 700 (Bold) weights for merging with Recursive.
    """
    input_font = BUILD_DIR / "NotoSansMonoCJKjp-VF.ttf"
    output_dir = BUILD_DIR

    if not input_font.exists():
        logger.error(f"Input file not found: {input_font}")
        logger.error("  Run download first")
        sys.exit(1)

    for weight in NotoWeight:
        output = output_dir / f"Noto-{weight.value}.ttf"

        logger.info(f"Extracting weight={weight.value} from {input_font.name}")
        extract_noto_instance(
            input_font,
            output,
            wght=float(weight.value),
        )
        logger.info(f"Successfully extracted weight {weight.value}")

    logger.info("All Noto weights extracted")
