#!/usr/bin/env python3
"""
Script to freeze OpenType features in the variable font.

After copying GSUB from Recursive VF, we freeze the same features that were
frozen in static fonts to ensure consistent appearance.
"""

import subprocess
import sys
from pathlib import Path

from src.logger import logger
from src.paths import DIST_DIR

# Features to freeze (same as static fonts)
# Note: 'case' feature is excluded as it can cause issues with feature freezing
FEATURES = [
    "dlig",
    "ss01",
    "ss02",
    "ss04",
    "ss05",
    "ss07",
    "ss08",
    "ss10",
    "ss12",
]


def freeze_vf_features(font_path: Path) -> bool:
    """
    Freeze OpenType features into the variable font using pyftfeatfreeze.

    Args:
        font_path: Path to the variable font file

    Returns:
        True if successful, False if failed
    """
    logger.info(f"Freezing features in variable font: {font_path.name}")
    features_str = f"rvrn,{','.join(FEATURES)}"
    logger.info(f"  Features: {features_str}")

    temp_path = font_path.with_suffix(".tmp.ttf")

    try:
        cmd = [
            "pyftfeatfreeze",
            f"--features={features_str}",
            str(font_path),
            str(temp_path),
        ]
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        temp_path.replace(font_path)
        logger.info(f"  ✓ Features frozen successfully")
        return True

    except subprocess.CalledProcessError as e:
        logger.error(f"  Error freezing features: {e}")
        if e.stderr:
            logger.error(f"  {e.stderr}")
        if temp_path.exists():
            temp_path.unlink()
        return False


def main():
    """Freeze features in the variable font"""
    vf_path = DIST_DIR / "WarpnineMono-VF.ttf"

    if not vf_path.exists():
        logger.error(f"Variable font not found: {vf_path}")
        logger.error("  Run copy-gsub first")
        sys.exit(1)

    logger.info("Freezing OpenType features in variable font")

    if freeze_vf_features(vf_path):
        logger.info("Variable font feature freezing completed successfully")
    else:
        logger.error("Failed to freeze features in variable font")
        sys.exit(1)


if __name__ == "__main__":
    main()
