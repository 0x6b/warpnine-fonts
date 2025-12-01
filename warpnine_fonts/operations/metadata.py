"""
Font metadata operations.

Sets monospace flags and version information.
"""

from datetime import date, datetime
from pathlib import Path

from fontTools.ttLib import TTFont

from warpnine_fonts.config.paths import DIST_DIR
from warpnine_fonts.core.font_io import iter_fonts
from warpnine_fonts.utils.logging import logger

# Monospace settings
TARGET_WIDTH = 600
MONO_PROPORTION = 9  # PANOSE value for monospaced

# Name table IDs
NAME_ID_VERSION = 5
NAME_ID_UNIQUE_ID = 3


def set_monospace_metrics(font_path: Path) -> None:
    """Set monospace flags in a font."""
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


def set_monospace() -> None:
    """Set monospace flags on all dist fonts."""
    fonts = list(iter_fonts(DIST_DIR))

    if not fonts:
        logger.warning(f"No .ttf files found in {DIST_DIR}/")
        return

    for font_path in fonts:
        set_monospace_metrics(font_path)

    logger.info("Monospace metadata applied to all dist fonts.")


def parse_version_string(value: str | None) -> tuple[date, str]:
    """
    Parse a version string or return today's date.

    Accepts formats:
    - YYYY-MM-DD (e.g., 2024-12-01)
    - YYYY-MM-DD.N (e.g., 2024-12-01.1)

    Returns a tuple of (date, full_version_string).
    """
    if value is None:
        today = date.today()
        return today, today.strftime("%Y-%m-%d")

    # Try YYYY-MM-DD.N format first
    if "." in value:
        date_part, build_num = value.rsplit(".", 1)
        try:
            parsed_date = datetime.strptime(date_part, "%Y-%m-%d").date()
            # Validate build number is numeric
            int(build_num)
            return parsed_date, value
        except ValueError:
            pass

    # Try plain YYYY-MM-DD format
    try:
        parsed_date = datetime.strptime(value, "%Y-%m-%d").date()
        return parsed_date, value
    except ValueError as exc:
        msg = f"Invalid version '{value}'. Expected YYYY-MM-DD or YYYY-MM-DD.N."
        raise SystemExit(msg) from exc


def format_version_string(version: str) -> str:
    """Format version string for name table."""
    return f"Version {version}"


def compute_font_revision(target_date: date) -> float:
    """Compute font revision as YYYY.MMDD."""
    return float(target_date.strftime("%Y.%m%d"))


def encode_name(record, text: str) -> None:
    """Encode text for name table record."""
    try:
        record.string = text.encode(record.getEncoding())
    except (LookupError, UnicodeEncodeError):
        record.string = text.encode("utf-16-be")


def stamp_font(font_path: Path, target_date: date, version_tag: str) -> None:
    """Stamp version date into a font."""
    version_string = format_version_string(version_tag)
    revision_value = compute_font_revision(target_date)

    logger.info(f"{font_path.name}:")
    logger.info(f"  version string -> {version_string}")
    logger.info(f"  head.fontRevision -> {revision_value}")

    font = TTFont(font_path)
    font["head"].fontRevision = revision_value

    updated = 0
    name_table = font["name"]
    for record in name_table.names:
        if record.nameID == NAME_ID_VERSION:
            encode_name(record, version_string)
            updated += 1
        elif record.nameID == NAME_ID_UNIQUE_ID:
            current = record.toUnicode()
            parts = [part.strip() for part in current.split(";") if part.strip()]
            if parts:
                parts[-1] = version_tag
            else:
                parts = [version_tag]
            encode_name(record, "; ".join(parts))
            updated += 1

    font.save(font_path)
    font.close()

    logger.info(f"  Updated {updated} name records")


def set_version(version_string: str | None = None) -> None:
    """Set version date on all dist fonts."""
    target_date, version_tag = parse_version_string(version_string)

    fonts = list(iter_fonts(DIST_DIR))

    if not fonts:
        logger.warning(f"No .ttf files found in {DIST_DIR}/")
        return

    for font_path in fonts:
        stamp_font(font_path, target_date, version_tag)

    logger.info("Version stamping complete.")
