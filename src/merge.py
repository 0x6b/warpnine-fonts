#!/usr/bin/env python3
"""
Script to merge multiple fonts

Merge Recursive Mono Duotone, Noto Sans Mono CJK JP to dist/
"""

import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

from fontTools.ttLib import TTFont

from src.logger import logger
from src.paths import BUILD_DIR, DIST_DIR


def fix_calt_registration(font: TTFont) -> None:
    """
    Ensure calt/rclt features are registered to all scripts (latn, cyrl, etc.) after font merge

    When merging with fontforge, calt is only registered to DFLT script,
    fix the issue where calt is not applied when displaying Latin characters in browsers
    """
    if "GSUB" not in font:
        return

    gsub = font["GSUB"].table

    if not hasattr(gsub, "ScriptList") or not hasattr(gsub, "FeatureList"):
        return

    # Search for calt and rclt indices
    calt_indices = []
    rclt_indices = []

    for i, record in enumerate(gsub.FeatureList.FeatureRecord):
        if record.FeatureTag == "calt":
            calt_indices.append(i)
        elif record.FeatureTag == "rclt":
            rclt_indices.append(i)

    if not calt_indices:
        return

    # Add calt/rclt to all scripts
    for script_record in gsub.ScriptList.ScriptRecord:
        script = script_record.Script

        # Add calt/rclt to DefaultLangSys
        if script.DefaultLangSys:
            features = list(script.DefaultLangSys.FeatureIndex)

            # Add calt (after aalt, insert at position 1)
            for calt_idx in calt_indices:
                if calt_idx not in features:
                    insert_pos = 1 if len(features) > 1 else len(features)
                    features.insert(insert_pos, calt_idx)

                    # Add rclt
            for rclt_idx in rclt_indices:
                if rclt_idx not in features:
                    features.append(rclt_idx)

                    # Update
            script.DefaultLangSys.FeatureIndex = features
            script.DefaultLangSys.FeatureCount = len(features)

        # Add calt/rclt to language-specific systems (JAN, KOR, ZHH, ZHS, ZHT, etc.)
        # This makes calt effective even when lang="ja" etc. is specified in browsers
        if hasattr(script, "LangSysRecord") and script.LangSysRecord:
            for lang_record in script.LangSysRecord:
                lang_sys = lang_record.LangSys
                if lang_sys.FeatureIndex is not None:
                    features = list(lang_sys.FeatureIndex)

                    # Add calt
                    for calt_idx in calt_indices:
                        if calt_idx not in features:
                            insert_pos = 1 if len(features) > 1 else len(features)
                            features.insert(insert_pos, calt_idx)

                            # Add rclt
                    for rclt_idx in rclt_indices:
                        if rclt_idx not in features:
                            features.append(rclt_idx)

                            # Update
                    lang_sys.FeatureIndex = features
                    lang_sys.FeatureCount = len(features)


@dataclass
class StyleMapping:
    recursive: str
    noto_weight: int


# Style mapping table - maps to Noto weights
STYLE_MAPPING = {
    "Light": StyleMapping("RecMonoDuotone-Light.ttf", 400),
    "LightItalic": StyleMapping("RecMonoDuotone-LightItalic.ttf", 400),
    "Regular": StyleMapping("RecMonoDuotone-Regular.ttf", 400),
    "Italic": StyleMapping("RecMonoDuotone-Italic.ttf", 400),
    "Medium": StyleMapping("RecMonoDuotone-Medium.ttf", 400),
    "MediumItalic": StyleMapping("RecMonoDuotone-MediumItalic.ttf", 400),
    "SemiBold": StyleMapping("RecMonoDuotone-SemiBold.ttf", 700),
    "SemiBoldItalic": StyleMapping("RecMonoDuotone-SemiBoldItalic.ttf", 700),
    "Bold": StyleMapping("RecMonoDuotone-Bold.ttf", 700),
    "BoldItalic": StyleMapping("RecMonoDuotone-BoldItalic.ttf", 700),
    "ExtraBold": StyleMapping("RecMonoDuotone-ExtraBold.ttf", 700),
    "ExtraBoldItalic": StyleMapping("RecMonoDuotone-ExtraBoldItalic.ttf", 700),
    "Black": StyleMapping("RecMonoDuotone-Black.ttf", 700),
    "BlackItalic": StyleMapping("RecMonoDuotone-BlackItalic.ttf", 700),
    "ExtraBlack": StyleMapping("RecMonoDuotone-ExtraBlack.ttf", 700),
    "ExtraBlackItalic": StyleMapping("RecMonoDuotone-ExtraBlackItalic.ttf", 700),
}


def merge_fonts(fonts: list[Path], output: Path, font_name: str | None = None) -> None:
    """
    Merge multiple fonts

    Args:
    fonts: List of fonts to merge (Recursive, Noto order)
    output: Output file path
    font_name: Font name (if not specified, inherit from first font)
    """
    output.parent.mkdir(parents=True, exist_ok=True)

    logger.info("Merging fonts:")
    for font in fonts:
        logger.info(f"  - {font}")
    logger.info(f"  Output: {output}")

    try:
        # Merge Recursive and Noto with fontforge
        script_lines = [
            "#!/usr/bin/env fontforge",
            "import fontforge",
            "",
            f'font = fontforge.open("{fonts[0]}")',
            f'font.mergeFonts("{fonts[1]}")',  # Merge Noto
            "",
        ]

        if font_name:
            fontname_clean = font_name.replace(" ", "")
            parts = font_name.split()
            style = parts[-1] if len(parts) > 1 else "Regular"
            family = " ".join(parts[:-1]) if len(parts) > 1 else font_name

            script_lines.extend(
                [
                    f'font.fontname = "{fontname_clean}"',
                    f'font.familyname = "{family}"',
                    f'font.fullname = "{font_name}"',
                    f'font.weight = "{style}"',
                    "",
                ]
            )

        script_lines.extend(
            [
                f'font.generate("{output}")',
                "font.close()",
            ]
        )

        script_content = "\n".join(script_lines)

        with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False) as f:
            f.write(script_content)
            script_path = f.name

        try:
            cmd = ["fontforge", "-lang=py", "-script", script_path]
            subprocess.run(cmd, check=True, capture_output=True, text=True)
        finally:
            Path(script_path).unlink(missing_ok=True)

        # Fix metadata with fontTools
        base = TTFont(str(output))

        # Fix name table
        if font_name:
            name_table = base["name"]
            for record in name_table.names:
                # nameID 0 (Copyright)
                if record.nameID == 0:
                    copyright_text = (
                        "Copyright 2020 The Recursive Project Authors (https://github.com/arrowtype/recursive). "
                        "Copyright 2014-2021 Adobe (http://www.adobe.com/), with Reserved Font Name 'Source'. "
                        "Warpnine Mono is based on Recursive Mono Duotone and Noto Sans Mono CJK JP."
                    )
                    record.string = copyright_text

                # nameID 3 (Unique ID)
                elif record.nameID == 3 and "RecMono" in str(record.string):
                    record.string = f"1.0;WARPNINE;{font_name.replace(' ', '')}"

                # nameID 16 (Typographic Family)
                elif record.nameID == 16:
                    parts = font_name.split()
                    family = " ".join(parts[:-1]) if len(parts) > 1 else font_name
                    record.string = family

                # nameID 17 (Typographic Subfamily)
                elif record.nameID == 17:
                    parts = font_name.split()
                    style = parts[-1] if len(parts) > 1 else "Regular"
                    record.string = style

        # Register calt feature to all scripts (for browser compatibility)
        fix_calt_registration(base)

        # Save
        base.save(str(output))
        base.close()

        logger.info("Successfully merged")

    except subprocess.CalledProcessError as e:
        logger.error(f"Error merging fonts: {e}")
        if e.stderr:
            logger.error(e.stderr)
        sys.exit(1)
    except Exception as e:
        logger.error(f"Error: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)


def merge_style(
    style: str,
    recursive_dir: Path,
    noto_dir: Path,
    output_dir: Path,
    output_filename: str,
    display_name: str,
) -> None:
    """
    Merge a specific style

    Args:
    style: Style name (Regular, Bold, Italic, BoldItalic)
    recursive_dir: Directory for Recursive fonts
    noto_dir: Directory for Noto fonts
    output_dir: Output directory
    output_filename: Output filename (no spaces, e.g. "SBMono")
    display_name: Display font name (with spaces, e.g. "SB Mono")
    """
    if style not in STYLE_MAPPING:
        logger.error(f"Unknown style: {style}")
        logger.error(f"  Available: {', '.join(STYLE_MAPPING.keys())}")
        sys.exit(1)

    mapping = STYLE_MAPPING[style]

    # File paths
    recursive_font = recursive_dir / mapping.recursive
    noto_font = noto_dir / f"Noto-{mapping.noto_weight}-subset.ttf"
    output_font = output_dir / f"{output_filename}-{style}.ttf"

    # Check existence
    if not recursive_font.exists():
        logger.error(f"Recursive font not found: {recursive_font}")
        sys.exit(1)
    if not noto_font.exists():
        logger.error(f"Noto font not found: {noto_font}")
        sys.exit(1)

    # Merge
    merge_fonts(
        [recursive_font, noto_font],
        output_font,
        f"{display_name} {style}",
    )


def main():
    recursive_dir = BUILD_DIR
    noto_dir = BUILD_DIR
    output_dir = DIST_DIR
    output_filename = "WarpnineMono"  # For filename (no spaces)
    display_name = "Warpnine Mono"  # For display name (with spaces)

    # Merge all styles
    for style in STYLE_MAPPING.keys():
        logger.info(f"Processing {style}")
        merge_style(
            style,
            recursive_dir,
            noto_dir,
            output_dir,
            output_filename,
            display_name,
        )

    logger.info("All styles merged")
    logger.info(f"Output fonts in {output_dir}:")
    for font in sorted(output_dir.glob(f"{output_filename}-*.ttf")):
        size = font.stat().st_size / 1024 / 1024  # MB
        logger.info(f"  {font.name:30} {size:6.2f} MB")


if __name__ == "__main__":
    main()
