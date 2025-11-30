#!/usr/bin/env python3
"""
Script to run all build steps in order.

Runs clean → download → extract-duotone → remove-ligatures → extract-weights →
subset → merge → build → copy-gsub → set-monospace → create-condensed →
freeze-features → set-version in order.

An optional --date flag can be passed to stamp a specific version date (YYYY-MM-DD).
If omitted, set-version uses today's date.
"""

import argparse
import sys

from src.build_variable import main as build_main
from src.clean import main as clean_main
from src.copy_gsub_to_vf import main as copy_gsub_main
from src.create_condensed import main as condensed_main
from src.download_fonts import main as download_main
from src.extract_duotone import main as extract_duotone_main
from src.extract_weights import main as extract_weights_main
from src.freeze_features import main as freeze_main
from src.logger import logger
from src.merge import main as merge_main
from src.remove_ligatures import main as remove_ligatures_main
from src.set_monospace import main as monospace_main
from src.set_version import stamp_font, parse_date
from src.paths import DIST_DIR
from src.subset import main as subset_main


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
        ("build", build_main),
        ("copy-gsub", copy_gsub_main),
        ("set-monospace", monospace_main),
        ("create-condensed", condensed_main),
        ("freeze-features", freeze_main),
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
