#!/usr/bin/env python3
"""
Stamp today's date into the version metadata of fonts in dist/.

The script updates the name table's version string (name ID 5) for every
`.ttf` file under the distribution directory and bumps the `head.fontRevision`
value to keep it in sync. An optional `--date` flag allows stamping an arbitrary
YYYY-MM-DD date.
"""

from __future__ import annotations

import argparse
from datetime import date, datetime
from pathlib import Path

from fontTools.ttLib import TTFont

from src.logger import logger
from src.paths import DIST_DIR

NAME_ID_VERSION = 5
NAME_ID_UNIQUE_ID = 3


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Apply a version string to fonts in dist/."
    )
    parser.add_argument(
        "--date",
        type=str,
        help="Date to embed (YYYY-MM-DD). Defaults to today.",
    )
    return parser.parse_args()


def parse_date(value: str | None) -> date:
    if value is None:
        return date.today()
    try:
        return datetime.strptime(value, "%Y-%m-%d").date()
    except ValueError as exc:
        msg = f"Invalid date '{value}'. Expected YYYY-MM-DD."
        raise SystemExit(msg) from exc


def format_version_string(target_date: date) -> str:
    return f"Version {target_date:%Y-%m-%d}"


def compute_font_revision(target_date: date) -> float:
    # Use a YYYY.MMDD fixed-point value that fits in head.fontRevision.
    return float(target_date.strftime("%Y.%m%d"))


def encode_name(record, text: str) -> None:
    try:
        record.string = text.encode(record.getEncoding())
    except (LookupError, UnicodeEncodeError):
        # Fallback for Macintosh Roman or unexpected encodings.
        record.string = text.encode("utf-16-be")


def update_name_records(font: TTFont, version_string: str, version_tag: str) -> int:
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
    return updated


def stamp_font(font_path: Path, target_date: date) -> None:
    version_string = format_version_string(target_date)
    version_tag = target_date.strftime("%Y-%m-%d")
    revision_value = compute_font_revision(target_date)

    logger.info(f"{font_path.name}:")
    logger.info(f"  version string -> {version_string}")
    logger.info(f"  head.fontRevision -> {revision_value}")

    font = TTFont(font_path)
    font["head"].fontRevision = revision_value
    updated = update_name_records(font, version_string, version_tag)
    font.save(font_path)
    font.close()

    logger.info(f"  Updated {updated} name records")


def main() -> None:
    args = parse_args()
    target_date = parse_date(args.date)

    fonts = sorted(DIST_DIR.glob("*.ttf"))
    if not fonts:
        logger.warning(f"No .ttf files found in {DIST_DIR}/")
        return

    for font_path in fonts:
        stamp_font(font_path, target_date)

    logger.info("Version stamping complete.")


if __name__ == "__main__":
    main()
