"""
Common filesystem locations and shared constants for build scripts.

Keeping these in one place helps the command-line utilities stay consistent
and keeps magic strings out of the individual scripts.
"""

from pathlib import Path

BUILD_DIR = Path("build")
DIST_DIR = Path("dist")
