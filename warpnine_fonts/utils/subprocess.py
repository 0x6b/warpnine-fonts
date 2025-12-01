"""
Subprocess execution utilities with consistent error handling.
"""

import subprocess
import sys
from pathlib import Path

from warpnine_fonts.utils.logging import logger


def run_command(
    cmd: list[str],
    description: str | None = None,
    exit_on_error: bool = True,
) -> subprocess.CompletedProcess:
    """
    Run a subprocess command with consistent logging and error handling.

    Args:
        cmd: Command and arguments to run
        description: Optional description for logging
        exit_on_error: Whether to exit on failure (default True)

    Returns:
        CompletedProcess result

    Raises:
        SystemExit: If exit_on_error is True and command fails
    """
    if description:
        logger.info(description)

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        if result.stdout:
            logger.debug(result.stdout)
        return result
    except subprocess.CalledProcessError as e:
        logger.error(f"Command failed: {' '.join(cmd)}")
        if e.stderr:
            logger.error(e.stderr)
        if exit_on_error:
            sys.exit(1)
        raise


def run_fonttools(
    subcommand: str,
    args: list[str],
    description: str | None = None,
    exit_on_error: bool = True,
) -> subprocess.CompletedProcess:
    """
    Run a fonttools subcommand.

    Args:
        subcommand: fonttools subcommand (e.g., "varLib.instancer")
        args: Additional arguments
        description: Optional description for logging
        exit_on_error: Whether to exit on failure

    Returns:
        CompletedProcess result
    """
    cmd = ["fonttools", subcommand, *args]
    return run_command(cmd, description, exit_on_error)


def run_pyftsubset(
    input_font: Path,
    output_file: Path,
    unicodes: str,
    *,
    keep_layout: bool = True,
    drop_tables: list[str] | None = None,
    exit_on_error: bool = True,
) -> subprocess.CompletedProcess:
    """
    Run pyftsubset to subset a font.

    Args:
        input_font: Input font path
        output_file: Output file path
        unicodes: Comma-separated Unicode ranges
        keep_layout: Whether to preserve layout features
        drop_tables: Tables to drop from the font
        exit_on_error: Whether to exit on failure

    Returns:
        CompletedProcess result
    """
    cmd = [
        "pyftsubset",
        str(input_font),
        f"--unicodes={unicodes}",
        "--glyph-names",
        f"--output-file={output_file}",
    ]

    if drop_tables:
        cmd.append(f"--drop-tables={','.join(drop_tables)}")

    if keep_layout:
        cmd.append("--layout-features=*")

    return run_command(
        cmd,
        f"Subsetting {input_font.name}",
        exit_on_error,
    )


def run_pyftfeatfreeze(
    input_font: Path,
    output_file: Path,
    features: list[str],
    exit_on_error: bool = True,
) -> subprocess.CompletedProcess:
    """
    Run pyftfeatfreeze to freeze OpenType features.

    Args:
        input_font: Input font path
        output_file: Output file path
        features: List of feature tags to freeze
        exit_on_error: Whether to exit on failure

    Returns:
        CompletedProcess result
    """
    features_str = ",".join(features)
    cmd = [
        "pyftfeatfreeze",
        f"--features={features_str}",
        str(input_font),
        str(output_file),
    ]

    return run_command(
        cmd,
        f"Freezing features in {input_font.name}",
        exit_on_error,
    )
