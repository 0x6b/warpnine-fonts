"""
Build pipeline orchestration.

Runs all build steps in the correct order.
"""

import shutil
import sys
from collections.abc import Callable

from warpnine_fonts.config.paths import BUILD_DIR, DIST_DIR
from warpnine_fonts.operations.download import clean, download
from warpnine_fonts.operations.extract import extract_duotone, extract_noto_weights
from warpnine_fonts.operations.freeze import freeze_static_mono, freeze_vf_and_sans
from warpnine_fonts.operations.merge import (
    merge_all_styles,
    remove_ligatures_from_duotone,
)
from warpnine_fonts.operations.metadata import (
    parse_date_string,
    set_monospace,
    stamp_font,
)
from warpnine_fonts.operations.sans import create_condensed, create_sans
from warpnine_fonts.operations.subset import subset_noto_fonts
from warpnine_fonts.operations.variable import (
    build_variable_font,
    copy_gsub_to_vf,
)
from warpnine_fonts.utils.logging import logger

FROZEN_BACKUP_DIR = BUILD_DIR / "frozen"


def backup_frozen_static() -> None:
    """Backup frozen static mono fonts before build removes GSUB."""
    font_files = sorted(DIST_DIR.glob("WarpnineMono-[!V]*.ttf"))
    if not font_files:
        logger.warning("No static mono fonts to backup")
        return

    FROZEN_BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    logger.info(f"Backing up {len(font_files)} frozen static fonts")

    for font_path in font_files:
        backup_path = FROZEN_BACKUP_DIR / font_path.name
        shutil.copy2(font_path, backup_path)

    logger.info("Backup complete")


def restore_frozen_static() -> None:
    """Restore frozen static mono fonts after build."""
    if not FROZEN_BACKUP_DIR.exists():
        logger.warning("No frozen backup directory found")
        return

    backup_files = sorted(FROZEN_BACKUP_DIR.glob("WarpnineMono-*.ttf"))
    if not backup_files:
        logger.warning("No frozen fonts to restore")
        return

    logger.info(f"Restoring {len(backup_files)} frozen static fonts")

    for backup_path in backup_files:
        dest_path = DIST_DIR / backup_path.name
        shutil.copy2(backup_path, dest_path)

    logger.info("Restore complete")


def run_set_version(date_str: str | None) -> None:
    """Run set-version step with optional date argument."""
    target_date = parse_date_string(date_str)
    fonts = sorted(DIST_DIR.glob("*.ttf"))
    if not fonts:
        logger.warning(f"No .ttf files found in {DIST_DIR}/")
        return
    for font_path in fonts:
        stamp_font(font_path, target_date)
    logger.info("Version stamping complete.")


def build_vf_wrapper() -> None:
    """Wrapper for build_variable_font that exits on failure."""
    if not build_variable_font():
        raise RuntimeError("Variable font build failed")


def copy_gsub_wrapper() -> None:
    """Wrapper for copy_gsub_to_vf that raises on failure."""
    if not copy_gsub_to_vf():
        raise RuntimeError("GSUB copy failed")


def run_all(date_string: str | None = None) -> None:
    """
    Run all build steps in order.

    Build pipeline:
      1. clean            - Remove build/ and dist/ directories
      2. download         - Download Recursive VF and Noto CJK VF
      3. extract-duotone  - Extract 16 static instances from Recursive VF
      4. remove-ligatures - Remove triple-backtick ligature from Duotone fonts
      5. extract-weights  - Extract Regular (400) and Bold (700) from Noto CJK
      6. subset           - Subset Noto fonts to Japanese Unicode ranges
      7. merge            - Merge Duotone + Noto CJK into WarpnineMono static fonts
      8. freeze-static    - Freeze OpenType features (ss01-ss12, liga, dlig, pnum)
      9. backup-frozen    - Backup frozen static fonts (VF build will modify them)
      10. build           - Build WarpnineMono-VF.ttf using fontTools varLib
      11. copy-gsub       - Copy GSUB table from Recursive VF to WarpnineMono VF
      12. restore-frozen  - Restore frozen static fonts from backup
      13. set-monospace   - Set monospace flags on all fonts in dist/
      14. create-condensed - Create WarpnineSansCondensed (85% width)
      15. create-sans     - Create WarpnineSans (proportional, Latin only)
      16. freeze-vf-sans  - Freeze features in VF and Sans fonts
      17. set-version     - Stamp version date into all fonts

    Args:
        date_string: Optional date to stamp (YYYY-MM-DD). Defaults to today.
    """
    steps: list[tuple[str, Callable[[], None]]] = [
        ("clean", clean),
        ("download", download),
        ("extract-duotone", extract_duotone),
        ("remove-ligatures", remove_ligatures_from_duotone),
        ("extract-weights", extract_noto_weights),
        ("subset", subset_noto_fonts),
        ("merge", merge_all_styles),
        ("freeze-static-mono", freeze_static_mono),
        ("backup-frozen", backup_frozen_static),
        ("build", build_vf_wrapper),
        ("copy-gsub", copy_gsub_wrapper),
        ("restore-frozen", restore_frozen_static),
        ("set-monospace", set_monospace),
        ("create-condensed", create_condensed),
        ("create-sans", create_sans),
        ("freeze-vf-and-sans", freeze_vf_and_sans),
    ]

    logger.info("Running all build steps")
    total_steps = len(steps) + 1  # +1 for set-version

    for i, (name, func) in enumerate(steps, 1):
        logger.info(f"[{i}/{total_steps}] Running {name}")
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

    # Run set-version as final step
    logger.info(f"[{total_steps}/{total_steps}] Running set-version")
    try:
        run_set_version(date_string)
        logger.info("set-version completed")
    except SystemExit as e:
        if e.code != 0:
            logger.error("set-version failed")
            sys.exit(e.code)
    except Exception as e:
        logger.error(f"set-version failed: {e}")
        sys.exit(1)

    logger.info("All steps completed successfully")


def run_mono() -> None:
    """Run only mono font build steps (no sans variants)."""
    steps: list[tuple[str, Callable[[], None]]] = [
        ("clean", clean),
        ("download", download),
        ("extract-duotone", extract_duotone),
        ("remove-ligatures", remove_ligatures_from_duotone),
        ("extract-weights", extract_noto_weights),
        ("subset", subset_noto_fonts),
        ("merge", merge_all_styles),
        ("freeze-static-mono", freeze_static_mono),
        ("backup-frozen", backup_frozen_static),
        ("build", build_vf_wrapper),
        ("copy-gsub", copy_gsub_wrapper),
        ("restore-frozen", restore_frozen_static),
        ("set-monospace", set_monospace),
    ]

    logger.info("Running mono font build steps")

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

    logger.info("Mono build completed successfully")
