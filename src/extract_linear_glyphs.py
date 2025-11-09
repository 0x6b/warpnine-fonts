#!/usr/bin/env python3
"""
Script to extract Linear style fonts for glyph extraction

Extract Light and Regular in Linear style (CASL=0) to get Linear glyphs
that we'll add to the variable font as alternates
"""

import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from src.logger import logger
from src.paths import BUILD_DIR


@dataclass(frozen=True)
class LinearInstance:
    style: str
    output_name: str
    wght: float
    slnt: float
    crsv: float


# Linear instances to extract for glyph extraction
LINEAR_INSTANCES = [
    LinearInstance(
        style="Light-Linear",
        output_name="RecMonoLinear-Light.ttf",
        wght=300.0,
        slnt=0.0,
        crsv=0.5,
    ),
    LinearInstance(
        style="LightItalic-Linear",
        output_name="RecMonoLinear-LightItalic.ttf",
        wght=300.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    LinearInstance(
        style="Regular-Linear",
        output_name="RecMonoLinear-Regular.ttf",
        wght=400.0,
        slnt=0.0,
        crsv=0.5,
    ),
    LinearInstance(
        style="Italic-Linear",
        output_name="RecMonoLinear-Italic.ttf",
        wght=400.0,
        slnt=-15.0,
        crsv=1.0,
    ),
]


def extract_linear_instance(
    variable_font: Path, instance: LinearInstance, output: Path
) -> None:
    """
    Extract a Linear instance from Variable Font

    Args:
        variable_font: Path to Recursive Variable Font
        instance: Instance configuration
        output: Output file path
    """
    output.parent.mkdir(parents=True, exist_ok=True)

    logger.info(f"Extracting {instance.style}")
    logger.info(
        f"  MONO=1, CASL=0 (Linear), wght={instance.wght}, slnt={instance.slnt}, CRSV={instance.crsv}"
    )
    logger.info(f"  Output: {output}")

    cmd = [
        "fonttools",
        "varLib.instancer",
        str(variable_font),
        "MONO=1",
        "CASL=0",  # Linear
        f"wght={instance.wght}",
        f"slnt={instance.slnt}",
        f"CRSV={instance.crsv}",
        "-o",
        str(output),
    ]

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        logger.info(f"Successfully extracted {instance.style}")
        if result.stdout:
            logger.info(result.stdout)
    except subprocess.CalledProcessError as e:
        logger.error(f"Error extracting {instance.style}: {e}")
        if e.stderr:
            logger.error(e.stderr)
        sys.exit(1)


def main():
    input_font = BUILD_DIR / "Recursive_VF_1.085.ttf"
    output_dir = BUILD_DIR

    # Check input file existence
    if not input_font.exists():
        logger.error(f"Input file not found: {input_font}")
        logger.error("  Run download first")
        sys.exit(1)

    # Extract each Linear instance
    for instance in LINEAR_INSTANCES:
        output = output_dir / instance.output_name
        extract_linear_instance(input_font, instance, output)

    logger.info("All Linear instances extracted")


if __name__ == "__main__":
    main()
