#!/usr/bin/env python3
"""
Script to extract Recursive Mono Casual variable font subset

Extracts a variable font with:
- MONO=1 (fixed to monospace)
- CASL=1 (fixed to Casual)
- wght=300-1000 (variable)
- slnt=-15 to 0 (variable, for italic)
- CRSV=0.5 to 1.0 (variable, for italic cursive forms)
"""

import subprocess
import sys
from pathlib import Path

from src.logger import logger
from src.paths import BUILD_DIR


def extract_mono_casual_vf(variable_font: Path, output: Path) -> None:
    """
    Extract Mono Casual variable font subset

    Args:
        variable_font: Path to Recursive Variable Font
        output: Output file path
    """
    output.parent.mkdir(parents=True, exist_ok=True)

    logger.info("Extracting Mono Casual variable font subset")
    logger.info(f"  MONO=1 (fixed), CASL=1 (fixed)")
    logger.info(
        f"  wght=300-1000 (variable), slnt=-15 to 0 (variable), CRSV=0.5-1.0 (variable)"
    )
    logger.info(f"  Output: {output}")

    # Use fonttools varLib.instancer to pin MONO and CASL while keeping wght, slnt, CRSV variable
    cmd = [
        "fonttools",
        "varLib.instancer",
        str(variable_font),
        "MONO=1",
        "CASL=1",
        "-o",
        str(output),
    ]

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        logger.info("Successfully extracted Mono Casual variable font")
        if result.stdout:
            logger.info(result.stdout)
    except subprocess.CalledProcessError as e:
        logger.error(f"Error extracting variable font: {e}")
        if e.stderr:
            logger.error(e.stderr)
        sys.exit(1)


def main():
    input_font = BUILD_DIR / "Recursive_VF_1.085.ttf"
    output_font = BUILD_DIR / "RecMonoCasual-VF.ttf"

    # Check input file existence
    if not input_font.exists():
        logger.error(f"Input file not found: {input_font}")
        logger.error("  Run download first")
        sys.exit(1)

    extract_mono_casual_vf(input_font, output_font)

    logger.info("Mono Casual variable font extraction completed")


if __name__ == "__main__":
    main()
