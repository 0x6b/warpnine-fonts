#!/usr/bin/env python3
"""
Script to run all build steps in order.

Runs clean → download → extract-duotone → remove-ligatures → extract-weights →
subset → merge → freeze-features (static) → build → copy-gsub → set-monospace →
create-condensed → create-sans → freeze-features (VF & Sans) → set-version in order.

Static mono fonts are frozen after merge (while they still have GSUB), backed up,
then restored after the VF build completes.

An optional --date flag can be passed to stamp a specific version date (YYYY-MM-DD).
If omitted, set-version uses today's date.
"""

import argparse
import shutil
import sys

from src.build_variable import main as build_main
from src.clean import main as clean_main
from src.copy_gsub_to_vf import main as copy_gsub_main
from src.create_condensed import main as condensed_main
from src.create_sans import main as sans_main
from src.download_fonts import main as download_main
from src.extract_duotone import main as extract_duotone_main
from src.extract_weights import main as extract_weights_main
from src.freeze_features import (
    MONO_FEATURES,
    MONO_STATIC_PATTERN,
    MONO_VF_PATTERN,
    SANS_CONDENSED_PATTERN,
    SANS_FEATURES,
    SANS_PATTERN,
    freeze_features_in_font,
)
from src.logger import logger
from src.merge import main as merge_main
from src.paths import BUILD_DIR, DIST_DIR
from src.remove_ligatures import main as remove_ligatures_main
from src.set_monospace import main as monospace_main
from src.set_version import parse_date, stamp_font
from src.subset import main as subset_main

FROZEN_BACKUP_DIR = BUILD_DIR / "frozen"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run all build steps in order.")
    parser.add_argument(
        "--date",
        type=str,
        default=None,
        help="Date to embed in version (YYYY-MM-DD). Defaults to today.",
    )
    return parser.parse_args()


def run_set_version(date_str: str | None) -> None:
    """Run set-version step with optional date argument."""
    target_date = parse_date(date_str)
    fonts = sorted(DIST_DIR.glob("*.ttf"))
    if not fonts:
        logger.warning(f"No .ttf files found in {DIST_DIR}/")
        return
    for font_path in fonts:
        stamp_font(font_path, target_date)
    logger.info("Version stamping complete.")


def freeze_static_mono() -> None:
    """Freeze features in static mono fonts (after merge, before build)."""
    font_files = sorted(DIST_DIR.glob(MONO_STATIC_PATTERN))
    if not font_files:
        logger.warning("No static mono fonts found to freeze")
        return

    logger.info(f"Freezing features in {len(font_files)} static mono fonts")
    failures = []
    for font_path in font_files:
        if not freeze_features_in_font(font_path, MONO_FEATURES):
            failures.append(font_path.name)

    if failures:
        logger.error(f"Failed to freeze {len(failures)} fonts")
        raise RuntimeError(f"Failed to freeze: {failures}")

    logger.info("Static mono fonts frozen successfully")


def backup_frozen_static() -> None:
    """Backup frozen static mono fonts before build removes GSUB."""
    font_files = sorted(DIST_DIR.glob(MONO_STATIC_PATTERN))
    if not font_files:
        logger.warning("No static mono fonts to backup")
        return

    FROZEN_BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    logger.info(
        f"Backing up {len(font_files)} frozen static fonts to {FROZEN_BACKUP_DIR}"
    )

    for font_path in font_files:
        backup_path = FROZEN_BACKUP_DIR / font_path.name
        shutil.copy2(font_path, backup_path)

    logger.info("Backup complete")


def restore_frozen_static() -> None:
    """Restore frozen static mono fonts after build."""
    if not FROZEN_BACKUP_DIR.exists():
        logger.warning("No frozen backup directory found")
        return

    backup_files = sorted(FROZEN_BACKUP_DIR.glob("WarpnineMono-*.ttf"))
    if not backup_files:
        logger.warning("No frozen fonts to restore")
        return

    logger.info(
        f"Restoring {len(backup_files)} frozen static fonts from {FROZEN_BACKUP_DIR}"
    )

    for backup_path in backup_files:
        dest_path = DIST_DIR / backup_path.name
        shutil.copy2(backup_path, dest_path)

    logger.info("Restore complete")


def freeze_vf_and_sans() -> None:
    """Freeze features in VF and Sans fonts (after create-condensed and create-sans)."""
    total = 0
    failures = []

    # Freeze VF
    vf_files = sorted(DIST_DIR.glob(MONO_VF_PATTERN))
    if vf_files:
        logger.info(f"Freezing features in {len(vf_files)} VF font(s)")
        total += len(vf_files)
        for font_path in vf_files:
            if not freeze_features_in_font(font_path, MONO_FEATURES):
                failures.append(font_path.name)

    # Freeze Sans Condensed
    sans_condensed_files = sorted(DIST_DIR.glob(SANS_CONDENSED_PATTERN))
    if sans_condensed_files:
        logger.info(
            f"Freezing features in {len(sans_condensed_files)} Sans Condensed fonts"
        )
        total += len(sans_condensed_files)
        for font_path in sans_condensed_files:
            if not freeze_features_in_font(font_path, SANS_FEATURES):
                failures.append(font_path.name)

    # Freeze Sans (non-condensed)
    sans_files = sorted(DIST_DIR.glob(SANS_PATTERN))
    if sans_files:
        logger.info(f"Freezing features in {len(sans_files)} Sans fonts")
        total += len(sans_files)
        for font_path in sans_files:
            if not freeze_features_in_font(font_path, SANS_FEATURES):
                failures.append(font_path.name)

    if total == 0:
        logger.warning("No VF or Sans fonts found to freeze")
        return

    if failures:
        logger.error(f"Failed to freeze {len(failures)} fonts")
        raise RuntimeError(f"Failed to freeze: {failures}")

    logger.info(f"VF and Sans fonts frozen successfully ({total} fonts)")


def main():
    """Run all steps"""
    args = parse_args()

    steps = [
        ("clean", clean_main),
        ("download", download_main),
        ("extract-duotone", extract_duotone_main),
        ("remove-ligatures", remove_ligatures_main),
        ("extract-weights", extract_weights_main),
        ("subset", subset_main),
        ("merge", merge_main),
        ("freeze-static-mono", freeze_static_mono),
        ("backup-frozen", backup_frozen_static),
        ("build", build_main),
        ("copy-gsub", copy_gsub_main),
        ("restore-frozen", restore_frozen_static),
        ("set-monospace", monospace_main),
        ("create-condensed", condensed_main),
        ("create-sans", sans_main),
        ("freeze-vf-and-sans", freeze_vf_and_sans),
    ]

    logger.info("Running all build steps")

    for i, (name, func) in enumerate(steps, 1):
        logger.info(f"[{i}/{len(steps)}] Running {name}")
        try:
            func()
            logger.info(f"{name} completed")
        except SystemExit as e:
            if e.code != 0:
                logger.error(f"{name} failed")
                sys.exit(e.code)
        except Exception as e:
            logger.error(f"{name} failed: {e}")
            sys.exit(1)

    # Run set-version as final step with optional date
    logger.info(f"[{len(steps) + 1}/{len(steps) + 1}] Running set-version")
    try:
        run_set_version(args.date)
        logger.info("set-version completed")
    except SystemExit as e:
        if e.code != 0:
            logger.error("set-version failed")
            sys.exit(e.code)
    except Exception as e:
        logger.error(f"set-version failed: {e}")
        sys.exit(1)

    logger.info("All steps completed successfully")


if __name__ == "__main__":
    main()
