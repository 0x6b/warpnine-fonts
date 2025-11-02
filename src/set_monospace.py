#!/usr/bin/env python3
"""
Ensure dist fonts advertise themselves as monospaced.

The script updates:
- post.isFixedPitch -> 1
- OS/2.panose.bProportion -> 9 (Monospaced)
- OS/2.xAvgCharWidth -> TARGET_WIDTH (default 600)

It targets every .ttf file under dist/.
"""

from __future__ import annotations

from pathlib import Path

from fontTools.ttLib import TTFont

from src.logger import logger
from src.paths import DIST_DIR

TARGET_WIDTH = 600
MONO_PROPORTION = 9  # PANOSE value for monospaced


def set_monospace_metrics(font_path: Path) -> None:
    font = TTFont(font_path)

    if "post" in font:
        font["post"].isFixedPitch = 1

    if "OS/2" in font:
        os2 = font["OS/2"]
        os2.panose.bProportion = MONO_PROPORTION
        os2.xAvgCharWidth = TARGET_WIDTH

    font.save(font_path)
    font.close()

    logger.info(f"Updated monospace metadata: {font_path.name}")


def main() -> None:
    fonts = sorted(DIST_DIR.glob("*.ttf"))
    if not fonts:
        logger.warning(f"No .ttf files found in {DIST_DIR}/")
        return

    for font_path in fonts:
        set_monospace_metrics(font_path)

    logger.info("Monospace metadata applied to all dist fonts.")


if __name__ == "__main__":
    main()
