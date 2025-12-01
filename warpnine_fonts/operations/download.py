"""
Download and clean operations.

Downloads source fonts (Recursive VF, Noto CJK) and handles build cleanup.
"""

import shutil
import sys
import zipfile
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path

import requests

from warpnine_fonts.config.paths import BUILD_DIR, DIST_DIR
from warpnine_fonts.utils.logging import logger


@dataclass(frozen=True)
class DownloadItem:
    """Configuration for a file to download."""

    url: str
    output_name: str
    description: str


# Direct file downloads
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


def clean_directory(path: Path) -> None:
    """
    Delete a directory.

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


def clean() -> None:
    """Remove build and dist directories."""
    logger.info("Cleaning build artifacts")

    for directory in (BUILD_DIR, DIST_DIR):
        clean_directory(directory)

    logger.info("Clean complete")


def download_file(item: DownloadItem, output_dir: Path) -> bool:
    """
    Download a file.

    Args:
        item: Download item configuration
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

        size = len(response.content) / 1024 / 1024
        logger.info(f"Downloaded ({size:.2f} MB)")
        return True

    except requests.RequestException as e:
        logger.error(f"Failed to download: {e}")
        return False


def download_recursive_vf(output_dir: Path) -> bool:
    """
    Download Recursive VF from GitHub releases.

    The font is distributed inside a zip file.

    Args:
        output_dir: Output directory

    Returns:
        True if successful, False if failed
    """
    target = output_dir / RECURSIVE_OUTPUT

    logger.info("Downloading Recursive VF")
    logger.info(f"  {target.name}")

    try:
        response = requests.get(RECURSIVE_ZIP_URL, timeout=120)
        response.raise_for_status()

        with zipfile.ZipFile(BytesIO(response.content)) as zf:
            with zf.open(RECURSIVE_ZIP_PATH) as src:
                target.write_bytes(src.read())

        size = target.stat().st_size / 1024 / 1024
        logger.info(f"Downloaded ({size:.2f} MB)")
        return True

    except requests.RequestException as e:
        logger.error(f"Failed to download: {e}")
        return False
    except (zipfile.BadZipFile, KeyError) as e:
        logger.error(f"Failed to extract from zip: {e}")
        return False


def download() -> None:
    """Download all source fonts and licenses."""
    BUILD_DIR.mkdir(parents=True, exist_ok=True)

    logger.info(f"Downloading fonts to {BUILD_DIR}")

    failures: list[str] = []

    # Download direct files
    for item in DOWNLOADS:
        if not download_file(item, BUILD_DIR):
            failures.append(item.description)

    # Download Recursive VF from GitHub
    if not download_recursive_vf(BUILD_DIR):
        failures.append("Recursive VF")

    success_count = len(DOWNLOADS) + 1 - len(failures)
    fail_count = len(failures)

    logger.info("Download Summary")
    logger.info(f"  Success: {success_count}")
    if fail_count > 0:
        logger.error(f"  Failed:  {fail_count}")
        sys.exit(1)

    logger.info(f"All files ready in {BUILD_DIR}/")
