#!/usr/bin/env python3
"""
Script to extract each weight from Variable Font

Extract 400, 700 from Noto Sans Mono CJK JP (Variable) to build/
"""

import subprocess
import sys
from pathlib import Path

from src.logger import logger
from src.paths import BUILD_DIR


def extract_weight(variable_font: Path, weight: int, output: Path) -> None:
    """
    Extract a specific weight from Variable Font

    Args:
    variable_font: Path to Variable Font
    weight: Weight value to extract (e.g., 400, 700)
    output: Output file path
    """
    output.parent.mkdir(parents=True, exist_ok=True)

    logger.info(f"Extracting weight={weight} from {variable_font.name}")
    logger.info(f"  Output: {output}")

    cmd = [
        "fonttools",
        "varLib.instancer",
        str(variable_font),
        f"wght={weight}",
        "-o",
        str(output),
    ]

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        logger.info(f"Successfully extracted weight {weight}")
        if result.stdout:
            logger.info(result.stdout)
    except subprocess.CalledProcessError as e:
        logger.error(f"Error extracting weight: {e}")
        if e.stderr:
            logger.error(e.stderr)
        sys.exit(1)


def main():
    input_font = BUILD_DIR / "NotoSansMonoCJKjp-VF.ttf"
    output_dir = BUILD_DIR

    # Noto CJK has 400(Regular), 500(Medium), 700(Bold), but
    # Recursive Duotone only has Regular and Bold, so extract only 400 and 700
    weights = [400, 700]

    # Check input file existence
    if not input_font.exists():
        logger.error(f"Input file not found: {input_font}")
        logger.error("  Run download first")
        sys.exit(1)

    # Extract each weight
    for weight in weights:
        output = output_dir / f"Noto-{weight}.ttf"
        extract_weight(input_font, weight, output)

    logger.info("All weights extracted")


if __name__ == "__main__":
    main()
