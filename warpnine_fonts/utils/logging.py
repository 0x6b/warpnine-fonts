"""
Shared logging configuration for build scripts.
"""

import logging

logging.basicConfig(
    level=logging.INFO,
    format="%(levelname)s: %(message)s",
    handlers=[logging.StreamHandler()],
)

logger = logging.getLogger("warpnine_fonts")
