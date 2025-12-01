"""
Variable font instance extraction utilities.
"""

from pathlib import Path
from typing import Protocol

from warpnine_fonts.utils.logging import logger
from warpnine_fonts.utils.subprocess import run_fonttools


class AxisValues(Protocol):
    """Protocol for axis value containers."""

    def to_instancer_args(self) -> list[str]:
        """Convert axis values to fonttools instancer arguments."""
        ...


def extract_instance(
    variable_font: Path,
    output: Path,
    axis_args: list[str],
    *,
    exit_on_error: bool = True,
) -> None:
    """
    Extract a static instance from a variable font.

    Args:
        variable_font: Path to variable font
        output: Output file path
        axis_args: Axis value arguments (e.g., ["wght=400", "MONO=1"])
        exit_on_error: Whether to exit on failure
    """
    output.parent.mkdir(parents=True, exist_ok=True)

    args = [str(variable_font), *axis_args, "-o", str(output)]

    run_fonttools(
        "varLib.instancer",
        args,
        f"Extracting instance to {output.name}",
        exit_on_error,
    )


def extract_recursive_instance(
    variable_font: Path,
    output: Path,
    *,
    mono: float = 1.0,
    casl: float = 0.0,
    wght: float = 400.0,
    slnt: float = 0.0,
    crsv: float = 0.5,
    exit_on_error: bool = True,
) -> None:
    """
    Extract a Recursive font instance with explicit axis values.

    Args:
        variable_font: Path to Recursive VF
        output: Output file path
        mono: MONO axis value (1.0 for monospace)
        casl: CASL axis value (0.0 Linear, 1.0 Casual)
        wght: wght axis value (300-1000)
        slnt: slnt axis value (0.0 upright, -15.0 italic)
        crsv: CRSV axis value (0.5 upright, 1.0 cursive)
        exit_on_error: Whether to exit on failure
    """
    logger.info(
        f"Extracting: MONO={mono}, CASL={casl}, wght={wght}, slnt={slnt}, CRSV={crsv}"
    )

    axis_args = [
        f"MONO={mono}",
        f"CASL={casl}",
        f"wght={wght}",
        f"slnt={slnt}",
        f"CRSV={crsv}",
    ]

    extract_instance(variable_font, output, axis_args, exit_on_error=exit_on_error)


def extract_noto_instance(
    variable_font: Path,
    output: Path,
    *,
    wght: float = 400.0,
    exit_on_error: bool = True,
) -> None:
    """
    Extract a Noto CJK font instance.

    Args:
        variable_font: Path to Noto CJK VF
        output: Output file path
        wght: wght axis value
        exit_on_error: Whether to exit on failure
    """
    logger.info(f"Extracting Noto: wght={wght}")

    axis_args = [f"wght={wght}"]
    extract_instance(variable_font, output, axis_args, exit_on_error=exit_on_error)
