"""
Filesystem path constants for build scripts.

Centralizes path definitions to avoid magic strings in individual scripts.
"""

from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.parent
BUILD_DIR = Path("build")
DIST_DIR = Path("dist")

# Source font filenames
RECURSIVE_VF = "Recursive_VF_1.085.ttf"
NOTO_CJK_VF = "NotoSansMonoCJKjp-VF.ttf"

# Output font prefixes
WARPNINE_MONO = "WarpnineMono"
WARPNINE_SANS = "WarpnineSans"
WARPNINE_SANS_CONDENSED = "WarpnineSansCondensed"
