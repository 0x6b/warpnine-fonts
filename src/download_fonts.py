#!/usr/bin/env python3
"""
Script to download font files and license files

Download Noto Sans Mono CJK JP and Recursive VF to build/
"""

import sys
import zipfile
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path

import requests

from src.logger import logger
from src.paths import BUILD_DIR


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

# Recursive VF from GitHub releases (inside zip)
RECURSIVE_ZIP_URL = "https://github.com/arrowtype/recursive/releases/download/v1.085/ArrowType-Recursive-1.085.zip"
RECURSIVE_ZIP_PATH = (
    "ArrowType-Recursive-1.085/Recursive_Desktop/Recursive_VF_1.085.ttf"
)
RECURSIVE_OUTPUT = "Recursive_VF_1.085.ttf"


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


def download_recursive_vf() -> bool:
    """Download Recursive VF from GitHub releases"""
    target = BUILD_DIR / RECURSIVE_OUTPUT

    logger.info("Downloading Recursive VF")
    logger.info(f"  {target.name}")

    try:
        response = requests.get(RECURSIVE_ZIP_URL, timeout=120)
        response.raise_for_status()

        # Extract the font from the zip
        with zipfile.ZipFile(BytesIO(response.content)) as zf:
            with zf.open(RECURSIVE_ZIP_PATH) as src:
                target.write_bytes(src.read())

        size = target.stat().st_size / 1024 / 1024  # MB
        logger.info(f"Downloaded ({size:.2f} MB)")
        return True
    except requests.RequestException as e:
        logger.error(f"Failed to download: {e}")
        return False
    except (zipfile.BadZipFile, KeyError) as e:
        logger.error(f"Failed to extract from zip: {e}")
        return False


def main():
    BUILD_DIR.mkdir(parents=True, exist_ok=True)

    logger.info(f"Downloading fonts to {BUILD_DIR}")

    # Download files
    failures = [item for item in DOWNLOADS if not download_file(item, BUILD_DIR)]

    # Download Recursive VF from GitHub
    if not download_recursive_vf():
        failures.append("Recursive VF")

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
