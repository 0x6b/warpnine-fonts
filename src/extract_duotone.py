#!/usr/bin/env python3
"""
Script to extract Duotone instances from Recursive Variable Font

RecMonoDuotone uses:
- Regular: MONO=1, CASL=0 (Linear), wght=400, slnt=0, CRSV=0.5
- Bold: MONO=1, CASL=1 (Casual), wght=750, slnt=0, CRSV=0.5
- Italic: MONO=1, CASL=1 (Casual), wght=400, slnt=-15, CRSV=1.0
- BoldItalic: MONO=1, CASL=1 (Casual), wght=750, slnt=-15, CRSV=1.0
"""

import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from src.logger import logger
from src.paths import BUILD_DIR


@dataclass(frozen=True)
class DuotoneInstance:
    style: str
    output_name: str
    # Axis values for extraction
    mono: float
    casl: float
    wght: float
    slnt: float
    crsv: float


# Duotone instances to extract
# For FeatureVariations approach: Extract ALL weights with Casual (CASL=1)
# We'll add FeatureVariations later to switch to Linear glyphs for Light/Regular
INSTANCES = [
    # Light - Casual (will switch to Linear via FeatureVariations)
    DuotoneInstance(
        style="Light",
        output_name="RecMonoDuotone-Light.ttf",
        mono=1.0,
        casl=1.0,  # Casual (compatible base)
        wght=300.0,
        slnt=0.0,
        crsv=0.5,
    ),
    DuotoneInstance(
        style="LightItalic",
        output_name="RecMonoDuotone-LightItalic.ttf",
        mono=1.0,
        casl=1.0,  # Casual (compatible base)
        wght=300.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # Regular - Casual (will switch to Linear via FeatureVariations)
    DuotoneInstance(
        style="Regular",
        output_name="RecMonoDuotone-Regular.ttf",
        mono=1.0,
        casl=1.0,  # Casual (compatible base)
        wght=400.0,
        slnt=0.0,
        crsv=0.5,
    ),
    DuotoneInstance(
        style="Italic",
        output_name="RecMonoDuotone-Italic.ttf",
        mono=1.0,
        casl=1.0,  # Casual (compatible base)
        wght=400.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # Medium - Casual
    DuotoneInstance(
        style="Medium",
        output_name="RecMonoDuotone-Medium.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=500.0,
        slnt=0.0,
        crsv=0.5,
    ),
    DuotoneInstance(
        style="MediumItalic",
        output_name="RecMonoDuotone-MediumItalic.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=500.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # SemiBold - Casual
    DuotoneInstance(
        style="SemiBold",
        output_name="RecMonoDuotone-SemiBold.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=600.0,
        slnt=0.0,
        crsv=0.5,
    ),
    DuotoneInstance(
        style="SemiBoldItalic",
        output_name="RecMonoDuotone-SemiBoldItalic.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=600.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # Bold - Casual
    DuotoneInstance(
        style="Bold",
        output_name="RecMonoDuotone-Bold.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=700.0,
        slnt=0.0,
        crsv=0.5,
    ),
    DuotoneInstance(
        style="BoldItalic",
        output_name="RecMonoDuotone-BoldItalic.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=700.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # ExtraBold - Casual
    DuotoneInstance(
        style="ExtraBold",
        output_name="RecMonoDuotone-ExtraBold.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=800.0,
        slnt=0.0,
        crsv=0.5,
    ),
    DuotoneInstance(
        style="ExtraBoldItalic",
        output_name="RecMonoDuotone-ExtraBoldItalic.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=800.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # Black - Casual
    DuotoneInstance(
        style="Black",
        output_name="RecMonoDuotone-Black.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=900.0,
        slnt=0.0,
        crsv=0.5,
    ),
    DuotoneInstance(
        style="BlackItalic",
        output_name="RecMonoDuotone-BlackItalic.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=900.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # ExtraBlack - Casual
    DuotoneInstance(
        style="ExtraBlack",
        output_name="RecMonoDuotone-ExtraBlack.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=1000.0,
        slnt=0.0,
        crsv=0.5,
    ),
    DuotoneInstance(
        style="ExtraBlackItalic",
        output_name="RecMonoDuotone-ExtraBlackItalic.ttf",
        mono=1.0,
        casl=1.0,  # Casual
        wght=1000.0,
        slnt=-15.0,
        crsv=1.0,
    ),
]


def extract_instance(
    variable_font: Path, instance: DuotoneInstance, output: Path
) -> None:
    """
    Extract a specific instance from Variable Font

    Args:
        variable_font: Path to Variable Font
        instance: Instance configuration
        output: Output file path
    """
    output.parent.mkdir(parents=True, exist_ok=True)

    logger.info(f"Extracting {instance.style}")
    logger.info(
        f"  MONO={instance.mono}, CASL={instance.casl}, wght={instance.wght}, slnt={instance.slnt}, CRSV={instance.crsv}"
    )
    logger.info(f"  Output: {output}")

    cmd = [
        "fonttools",
        "varLib.instancer",
        str(variable_font),
        f"MONO={instance.mono}",
        f"CASL={instance.casl}",
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

    # Extract each instance
    for instance in INSTANCES:
        output = output_dir / instance.output_name
        extract_instance(input_font, instance, output)

    logger.info("All Duotone instances extracted")


if __name__ == "__main__":
    main()
