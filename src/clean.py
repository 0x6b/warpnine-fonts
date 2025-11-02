#!/usr/bin/env python3
"""
Script to remove build artifacts

Delete build/ and dist/ directories
"""

import shutil
import sys
from pathlib import Path

from src.logger import logger
from src.paths import BUILD_DIR, DIST_DIR


def clean_directory(path: Path) -> None:
    """
    Delete a directory

    Args:
        path: Path of the directory to delete
    """
    if path.exists():
        logger.info(f"Removing {path}/")
        try:
            shutil.rmtree(path)
            logger.info(f"Removed {path}/")
        except Exception as e:
            logger.error(f"Failed to remove {path}/: {e}")
            sys.exit(1)
    else:
        logger.info(f"{path}/ does not exist (skipped)")


def main():
    logger.info("Cleaning build artifacts")

    for directory in (BUILD_DIR, DIST_DIR):
        clean_directory(directory)

    logger.info("Clean complete")


if __name__ == "__main__":
    main()
