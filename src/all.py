#!/usr/bin/env python3
"""
Script to run all build steps in order.

Runs clean → download → remove-ligatures → extract → subset → merge → build →
set-monospace → set-version in order.
"""

import sys

from src.build_variable import main as build_main
from src.logger import logger
from src.clean import main as clean_main
from src.download_fonts import main as download_main
from src.extract_weights import main as extract_main
from src.merge import main as merge_main
from src.remove_ligatures import main as remove_ligatures_main
from src.set_monospace import main as monospace_main
from src.set_version import main as version_main
from src.subset import main as subset_main


def main():
    """Run all steps"""
    steps = [
        ("clean", clean_main),
        ("download", download_main),
        ("remove-ligatures", remove_ligatures_main),
        ("extract", extract_main),
        ("subset", subset_main),
        ("merge", merge_main),
        ("build", build_main),
        ("set-monospace", monospace_main),
        ("set-version", version_main),
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

    logger.info("All steps completed successfully")


if __name__ == "__main__":
    main()
