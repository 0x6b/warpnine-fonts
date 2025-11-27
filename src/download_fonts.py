#!/usr/bin/env python3
"""
Script to download font files and license files

Download Noto Sans Mono CJK JP and copy Recursive VF from submodule to build/
"""

import shutil
import sys
from dataclasses import dataclass
from pathlib import Path

import requests

from src.logger import logger
from src.paths import BUILD_DIR, PROJECT_ROOT


@dataclass(frozen=True)
class DownloadItem:
    url: str
    output_name: str
    description: str


# Files to download
DOWNLOADS = [
    # Noto Sans Mono CJK JP (Variable Font)
    DownloadItem(
        "https://raw.githubusercontent.com/notofonts/noto-cjk/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/Variable/TTF/Mono/NotoSansMonoCJKjp-VF.ttf",
        "NotoSansMonoCJKjp-VF.ttf",
        "Noto Sans Mono CJK JP (Variable)",
    ),
    # Licenses
    DownloadItem(
        "https://raw.githubusercontent.com/notofonts/noto-cjk/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/LICENSE",
        "LICENSE-NotoSansCJK.txt",
        "Noto CJK License",
    ),
    DownloadItem(
        "https://raw.githubusercontent.com/arrowtype/recursive/refs/tags/v1.085/OFL.txt",
        "LICENSE-Recursive.txt",
        "Recursive License (OFL)",
    ),
]


def download_file(item: DownloadItem, output_dir: Path) -> bool:
    """
    Download a file

    Args:
    item: Download item
    output_dir: Output directory

    Returns:
    True if successful, False if failed
    """
    target = output_dir / item.output_name
    logger.info(f"Downloading {item.description}")
    logger.info(f"  {target.name}")

    try:
        response = requests.get(item.url, timeout=60)
        response.raise_for_status()

        target.write_bytes(response.content)

        size = len(response.content) / 1024 / 1024  # MB
        logger.info(f"Downloaded ({size:.2f} MB)")
        return True

    except requests.RequestException as e:
        logger.error(f"Failed to download: {e}")
        return False


def copy_recursive_vf() -> bool:
    """Copy Recursive VF from submodule to build directory"""
    recursive_vf = (
        PROJECT_ROOT
        / "recursive/fonts/ArrowType-Recursive-1.085/Recursive_Desktop/Recursive_VF_1.085.ttf"
    )
    target = BUILD_DIR / "Recursive_VF_1.085.ttf"

    if not recursive_vf.exists():
        logger.error(f"Recursive VF not found: {recursive_vf}")
        logger.error("Run: git submodule update --init --recursive")
        return False

    logger.info("Copying Recursive VF from submodule")
    logger.info(f"  {target.name}")

    try:
        shutil.copy2(recursive_vf, target)
        size = target.stat().st_size / 1024 / 1024  # MB
        logger.info(f"Copied ({size:.2f} MB)")
        return True
    except Exception as e:
        logger.error(f"Failed to copy: {e}")
        return False


def main():
    BUILD_DIR.mkdir(parents=True, exist_ok=True)

    logger.info(f"Downloading fonts to {BUILD_DIR}")

    # Download files
    failures = [item for item in DOWNLOADS if not download_file(item, BUILD_DIR)]

    # Copy Recursive VF from submodule
    if not copy_recursive_vf():
        failures.append("Recursive VF (copy from submodule)")

    success_count = len(DOWNLOADS) + 1 - len(failures)
    fail_count = len(failures)

    # Summary
    logger.info("Download Summary")
    logger.info(f"  Success: {success_count}")
    if fail_count > 0:
        logger.error(f"  Failed:  {fail_count}")

    if fail_count > 0:
        sys.exit(1)

    logger.info(f"All files ready in {BUILD_DIR}/")


if __name__ == "__main__":
    main()
