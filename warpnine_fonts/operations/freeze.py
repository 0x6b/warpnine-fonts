"""
Feature freezing operations.

Permanently applies OpenType features to font glyphs using pyftfeatfreeze.
"""

import sys
from pathlib import Path

from warpnine_fonts.config.features import get_font_configs
from warpnine_fonts.config.paths import DIST_DIR
from warpnine_fonts.utils.logging import logger
from warpnine_fonts.utils.subprocess import run_pyftfeatfreeze


def freeze_features_in_font(font_path: Path, features: list[str]) -> bool:
    """
    Freeze OpenType features into a font file.

    Args:
        font_path: Path to the font file
        features: List of feature tags to freeze

    Returns:
        True if successful, False if failed
    """
    logger.info(f"Freezing features in {font_path.name}")

    # Add rvrn (Required Variation Alternates) to features
    features_with_rvrn = ["rvrn", *features]
    logger.info(f"  Features: {','.join(features_with_rvrn)}")

    temp_path = font_path.with_suffix(".tmp.ttf")

    try:
        run_pyftfeatfreeze(
            font_path,
            temp_path,
            features_with_rvrn,
            exit_on_error=False,
        )
        temp_path.replace(font_path)
        logger.info("  Features frozen successfully")
        return True

    except Exception as e:
        logger.error(f"  Error freezing features: {e}")
        if temp_path.exists():
            temp_path.unlink()
        return False


def freeze_static_mono() -> None:
    """
    Freeze features in static mono fonts only.

    This is called before building the variable font, while GSUB is still present.
    """
    from warpnine_fonts.config.features import MONO_FEATURES

    font_files = sorted(DIST_DIR.glob("WarpnineMono-[!V]*.ttf"))

    if not font_files:
        logger.error("No static mono fonts found in dist/")
        sys.exit(1)

    logger.info(f"Freezing features in {len(font_files)} static mono fonts")
    failures = []

    for font_path in font_files:
        if not freeze_features_in_font(font_path, MONO_FEATURES):
            failures.append(font_path.name)

    if failures:
        logger.error(f"Failed to freeze features in {len(failures)} fonts:")
        for name in failures:
            logger.error(f"  - {name}")
        sys.exit(1)

    logger.info("Static mono feature freezing completed")


def freeze_all() -> None:
    """Freeze features in all font families."""
    total_fonts = 0
    failures: list[str] = []

    for directory, pattern, features in get_font_configs(DIST_DIR):
        font_files = sorted(directory.glob(pattern))

        if not font_files:
            continue

        logger.info(f"Processing {directory}/{pattern}: {len(font_files)} fonts")
        total_fonts += len(font_files)

        for font_path in font_files:
            if not freeze_features_in_font(font_path, features):
                failures.append(font_path.name)

    if total_fonts == 0:
        logger.error("No fonts found in dist/")
        sys.exit(1)

    success_count = total_fonts - len(failures)
    logger.info(f"Feature freezing summary: {success_count}/{total_fonts} successful")

    if failures:
        logger.error(f"Failed to freeze features in {len(failures)} fonts:")
        for name in failures:
            logger.error(f"  - {name}")
        sys.exit(1)

    logger.info("Feature freezing completed successfully")


def freeze_vf_and_sans() -> None:
    """Freeze features in variable font and sans families."""
    from warpnine_fonts.config.features import MONO_FEATURES, SANS_FEATURES

    configs = [
        (DIST_DIR, "WarpnineMono-VF.ttf", MONO_FEATURES),
        (DIST_DIR, "WarpnineSansCondensed-*.ttf", SANS_FEATURES),
        (DIST_DIR, "WarpnineSans-*.ttf", SANS_FEATURES),
    ]

    total_fonts = 0
    failures: list[str] = []

    for directory, pattern, features in configs:
        font_files = sorted(directory.glob(pattern))

        if not font_files:
            continue

        logger.info(f"Processing {pattern}: {len(font_files)} fonts")
        total_fonts += len(font_files)

        for font_path in font_files:
            if not freeze_features_in_font(font_path, features):
                failures.append(font_path.name)

    if total_fonts == 0:
        logger.warning("No VF or Sans fonts found")
        return

    success_count = total_fonts - len(failures)
    logger.info(f"Feature freezing summary: {success_count}/{total_fonts} successful")

    if failures:
        logger.error(f"Failed to freeze features in {len(failures)} fonts:")
        for name in failures:
            logger.error(f"  - {name}")
        sys.exit(1)

    logger.info("VF and Sans feature freezing completed")
