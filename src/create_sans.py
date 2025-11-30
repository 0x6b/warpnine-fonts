#!/usr/bin/env python3
"""
Create Warpnine Sans (non-condensed proportional) from Recursive Sans Linear Static.

This script:
1. Extracts static instances from Recursive VF (Sans=MONO=0, Linear=CASL=0)
2. Updates font naming for Warpnine Sans
"""

import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from fontTools.ttLib import TTFont

from src.logger import logger
from src.paths import BUILD_DIR, DIST_DIR


@dataclass(frozen=True)
class SansConfig:
    """Configuration for a sans variant."""

    style: str
    output_name: str
    # Axis values for the base extraction
    mono: float  # 0 = Sans (proportional), 1 = Mono
    casl: float  # 0 = Linear, 1 = Casual
    wght: float
    slnt: float
    crsv: float


# Sans instances to create
# Recursive Sans Linear Static - same weights as condensed
INSTANCES = [
    # Light
    SansConfig(
        style="Light",
        output_name="WarpnineSans-Light.ttf",
        mono=0.0,
        casl=0.0,
        wght=300.0,
        slnt=0.0,
        crsv=0.5,
    ),
    SansConfig(
        style="LightItalic",
        output_name="WarpnineSans-LightItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=300.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # Regular
    SansConfig(
        style="Regular",
        output_name="WarpnineSans-Regular.ttf",
        mono=0.0,
        casl=0.0,
        wght=400.0,
        slnt=0.0,
        crsv=0.5,
    ),
    SansConfig(
        style="Italic",
        output_name="WarpnineSans-Italic.ttf",
        mono=0.0,
        casl=0.0,
        wght=400.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # Medium
    SansConfig(
        style="Medium",
        output_name="WarpnineSans-Medium.ttf",
        mono=0.0,
        casl=0.0,
        wght=500.0,
        slnt=0.0,
        crsv=0.5,
    ),
    SansConfig(
        style="MediumItalic",
        output_name="WarpnineSans-MediumItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=500.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # SemiBold
    SansConfig(
        style="SemiBold",
        output_name="WarpnineSans-SemiBold.ttf",
        mono=0.0,
        casl=0.0,
        wght=600.0,
        slnt=0.0,
        crsv=0.5,
    ),
    SansConfig(
        style="SemiBoldItalic",
        output_name="WarpnineSans-SemiBoldItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=600.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # Bold
    SansConfig(
        style="Bold",
        output_name="WarpnineSans-Bold.ttf",
        mono=0.0,
        casl=0.0,
        wght=700.0,
        slnt=0.0,
        crsv=0.5,
    ),
    SansConfig(
        style="BoldItalic",
        output_name="WarpnineSans-BoldItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=700.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # ExtraBold
    SansConfig(
        style="ExtraBold",
        output_name="WarpnineSans-ExtraBold.ttf",
        mono=0.0,
        casl=0.0,
        wght=800.0,
        slnt=0.0,
        crsv=0.5,
    ),
    SansConfig(
        style="ExtraBoldItalic",
        output_name="WarpnineSans-ExtraBoldItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=800.0,
        slnt=-15.0,
        crsv=1.0,
    ),
    # Black
    SansConfig(
        style="Black",
        output_name="WarpnineSans-Black.ttf",
        mono=0.0,
        casl=0.0,
        wght=900.0,
        slnt=0.0,
        crsv=0.5,
    ),
    SansConfig(
        style="BlackItalic",
        output_name="WarpnineSans-BlackItalic.ttf",
        mono=0.0,
        casl=0.0,
        wght=900.0,
        slnt=-15.0,
        crsv=1.0,
    ),
]


def extract_static_instance(
    variable_font: Path, config: SansConfig, output: Path
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


def update_name_table(font: TTFont, style: str) -> None:
    """Update name table for Warpnine Sans."""
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
            record.toUnicode()
        except UnicodeDecodeError:
            continue

        # Update family name
        if record.nameID == 1:
            record.string = f"Warpnine Sans {style}"

        # Update full name
        elif record.nameID == 4:
            record.string = f"Warpnine Sans {style}"

        # Update PostScript name
        elif record.nameID == 6:
            # PostScript names cannot have spaces
            record.string = f"WarpnineSans-{style}"

        # Update typographic family name
        elif record.nameID == 16:
            record.string = "Warpnine Sans"

        # Update typographic subfamily name
        elif record.nameID == 17:
            record.string = style


def create_sans(variable_font: Path, config: SansConfig, output: Path) -> None:
    """Create a sans variant from the variable font."""
    # Step 1: Extract static instance to a temp file
    temp_static = output.with_suffix(".temp.ttf")
    logger.info(f"Extracting {config.style} instance")
    extract_static_instance(variable_font, config, temp_static)

    # Step 2: Load and update names
    font = TTFont(temp_static)
    update_name_table(font, config.style)

    # Step 3: Save
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
        logger.info(f"Creating {config.style}")
        try:
            create_sans(input_font, config, output)
        except Exception as e:
            logger.error(f"Failed to create {config.style}: {e}")
            raise

    logger.info(f"All sans instances created in {output_dir}/")


if __name__ == "__main__":
    main()
