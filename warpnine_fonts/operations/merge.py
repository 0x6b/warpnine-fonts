"""
Font merging operations.

Merges Recursive Mono Duotone with Noto Sans Mono CJK JP and handles ligature processing.
"""

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from fontTools.ttLib import TTFont

from warpnine_fonts.config.instances import DUOTONE_INSTANCES, get_noto_weight_for_style
from warpnine_fonts.config.paths import BUILD_DIR, DIST_DIR, WARPNINE_MONO
from warpnine_fonts.core.font_io import iter_fonts
from warpnine_fonts.core.gsub import fix_calt_registration
from warpnine_fonts.core.naming import COPYRIGHT_TEMPLATE, FontNaming, update_name_table
from warpnine_fonts.utils.logging import logger


def remove_grave_ligature(font_path: Path) -> None:
    """
    Remove the three-backtick ligature from a font.

    Args:
        font_path: Path to the font file
    """
    logger.info(f"Removing three-backtick ligature from {font_path.name}")

    font = TTFont(font_path)

    if "GSUB" not in font:
        logger.warning(f"No GSUB table found in {font_path.name}")
        font.close()
        return

    gsub = font["GSUB"].table
    modified = False

    for lookup_idx, lookup in enumerate(gsub.LookupList.Lookup):
        # Type 6 is chaining contextual substitution
        if lookup.LookupType == 6:
            for subtable_idx, subtable in enumerate(lookup.SubTable):
                if hasattr(subtable, "Format") and subtable.Format == 3:
                    if (
                        hasattr(subtable, "InputCoverage")
                        and len(subtable.InputCoverage) > 0
                    ):
                        first_input_glyphs = list(subtable.InputCoverage[0].glyphs)
                        if (
                            "grave" in first_input_glyphs
                            and hasattr(subtable, "LookAheadCoverage")
                            and len(subtable.LookAheadCoverage) == 2
                        ):
                            lookahead_0 = list(subtable.LookAheadCoverage[0].glyphs)
                            lookahead_1 = list(subtable.LookAheadCoverage[1].glyphs)
                            if "grave" in lookahead_0 and "grave" in lookahead_1:
                                logger.info(
                                    f"  Found three-backtick pattern in Lookup {lookup_idx}"
                                )
                                subtable.SubstLookupRecord = []
                                modified = True

        # Type 1 is single substitution
        elif lookup.LookupType == 1:
            for subtable in lookup.SubTable:
                if hasattr(subtable, "mapping"):
                    if (
                        "grave" in subtable.mapping
                        and subtable.mapping["grave"] == "grave_grave_grave.code"
                    ):
                        logger.info(f"  Removing grave → grave_grave_grave.code")
                        del subtable.mapping["grave"]
                        modified = True

    if modified:
        font.save(font_path)
        logger.info(f"  Saved modified font")
    else:
        logger.info(f"  No three-backtick ligature found")

    font.close()


def remove_ligatures_from_duotone() -> None:
    """Remove three-backtick ligature from all Recursive Mono Duotone fonts."""
    rec_fonts = list(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))

    if not rec_fonts:
        logger.error("No Recursive Mono Duotone fonts found in build directory")
        logger.error("  Run extract-duotone first")
        sys.exit(1)

    logger.info(f"Processing {len(rec_fonts)} Recursive Mono Duotone fonts")

    for font_path in rec_fonts:
        remove_grave_ligature(font_path)

    logger.info("Ligature removal completed")


def merge_fonts(fonts: list[Path], output: Path, font_name: str) -> None:
    """
    Merge multiple fonts using fontforge.

    Args:
        fonts: List of fonts to merge (Recursive first, Noto second)
        output: Output file path
        font_name: Full font name (e.g., "Warpnine Mono Regular")
    """
    output.parent.mkdir(parents=True, exist_ok=True)

    logger.info("Merging fonts:")
    for font in fonts:
        logger.info(f"  - {font}")
    logger.info(f"  Output: {output}")

    try:
        fontname_clean = font_name.replace(" ", "")
        parts = font_name.split()
        style = parts[-1] if len(parts) > 1 else "Regular"
        family = " ".join(parts[:-1]) if len(parts) > 1 else font_name

        script_lines = [
            "#!/usr/bin/env fontforge",
            "import fontforge",
            "",
            f'font = fontforge.open("{fonts[0]}")',
            f'font.mergeFonts("{fonts[1]}")',
            "",
            f'font.fontname = "{fontname_clean}"',
            f'font.familyname = "{family}"',
            f'font.fullname = "{font_name}"',
            f'font.weight = "{style}"',
            "",
            f'font.generate("{output}")',
            "font.close()",
        ]

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
        font = TTFont(str(output))

        # Update name table
        for record in font["name"].names:
            if record.nameID == 0:
                record.string = COPYRIGHT_TEMPLATE.format(
                    additional="Warpnine Mono is based on Recursive Mono Duotone and Noto Sans Mono CJK JP."
                )
            elif record.nameID == 3 and "RecMono" in str(record.string):
                record.string = f"1.0;WARPNINE;{fontname_clean}"
            elif record.nameID == 16:
                record.string = family
            elif record.nameID == 17:
                record.string = style

        fix_calt_registration(font)
        font.save(str(output))
        font.close()

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


def merge_all_styles() -> None:
    """Merge all Recursive Duotone styles with Noto CJK."""
    logger.info("Merging all font styles")

    for instance in DUOTONE_INSTANCES:
        style = instance.style
        noto_weight = get_noto_weight_for_style(style)

        recursive_font = BUILD_DIR / instance.output_name
        noto_font = BUILD_DIR / f"Noto-{noto_weight.value}-subset.ttf"
        output_font = DIST_DIR / f"{WARPNINE_MONO}-{style}.ttf"

        if not recursive_font.exists():
            logger.error(f"Recursive font not found: {recursive_font}")
            sys.exit(1)
        if not noto_font.exists():
            logger.error(f"Noto font not found: {noto_font}")
            sys.exit(1)

        logger.info(f"Processing {style}")
        merge_fonts(
            [recursive_font, noto_font],
            output_font,
            f"Warpnine Mono {style}",
        )

    logger.info("All styles merged")
    logger.info(f"Output fonts in {DIST_DIR}:")
    for font in sorted(DIST_DIR.glob(f"{WARPNINE_MONO}-*.ttf")):
        size = font.stat().st_size / 1024 / 1024
        logger.info(f"  {font.name:30} {size:6.2f} MB")


def create_ligature_only_gsub(source_font_path: Path):
    """Create a modified GSUB table with only ligature features."""
    font = TTFont(source_font_path)

    if "GSUB" not in font:
        logger.error(f"No GSUB table in {source_font_path}")
        return None

    gsub = font["GSUB"]

    # Features to KEEP (only pure ligatures)
    features_to_keep = {"liga", "dlig"}

    if hasattr(gsub.table, "FeatureList") and gsub.table.FeatureList:
        feature_records = gsub.table.FeatureList.FeatureRecord

        indices_to_remove = []
        kept_features = []

        for i, feature_record in enumerate(feature_records):
            if feature_record.FeatureTag not in features_to_keep:
                indices_to_remove.append(i)
            else:
                kept_features.append(feature_record.FeatureTag)

        logger.info(f"  Keeping features: {', '.join(sorted(kept_features))}")
        logger.info(f"  Removing {len(indices_to_remove)} other features")

        for i in reversed(indices_to_remove):
            feature_records.pop(i)

        gsub.table.FeatureList.FeatureCount = len(feature_records)

        # Update script references
        if hasattr(gsub.table, "ScriptList") and gsub.table.ScriptList:
            for script_record in gsub.table.ScriptList.ScriptRecord:
                script = script_record.Script

                if hasattr(script, "DefaultLangSys") and script.DefaultLangSys:
                    lang_sys = script.DefaultLangSys
                    new_feature_indices = []

                    for feature_index in lang_sys.FeatureIndex:
                        offset = sum(
                            1
                            for removed_idx in indices_to_remove
                            if removed_idx < feature_index
                        )
                        new_index = feature_index - offset

                        if feature_index not in indices_to_remove:
                            new_feature_indices.append(new_index)

                    lang_sys.FeatureIndex = new_feature_indices
                    lang_sys.FeatureCount = len(new_feature_indices)

                if hasattr(script, "LangSysRecord"):
                    for lang_sys_record in script.LangSysRecord:
                        lang_sys = lang_sys_record.LangSys
                        new_feature_indices = []

                        for feature_index in lang_sys.FeatureIndex:
                            offset = sum(
                                1
                                for removed_idx in indices_to_remove
                                if removed_idx < feature_index
                            )
                            new_index = feature_index - offset

                            if feature_index not in indices_to_remove:
                                new_feature_indices.append(new_index)

                        lang_sys.FeatureIndex = new_feature_indices
                        lang_sys.FeatureCount = len(new_feature_indices)

    return gsub


def preserve_ligatures() -> bool:
    """Preserve ligatures from Recursive fonts in WarpnineMono fonts."""
    logger.info("Preserving ligatures from Recursive fonts...")

    recursive_regular = BUILD_DIR / "RecMonoDuotone-Regular.ttf"
    recursive_italic = BUILD_DIR / "RecMonoDuotone-Italic.ttf"

    if not recursive_regular.exists():
        logger.error(f"RecMonoDuotone-Regular.ttf not found in {BUILD_DIR}")
        logger.error("Run extract-duotone first")
        return False

    logger.info("Creating ligature GSUB from upright font...")
    gsub_upright = create_ligature_only_gsub(recursive_regular)
    if not gsub_upright:
        return False

    logger.info("Creating ligature GSUB from italic font...")
    gsub_italic = create_ligature_only_gsub(recursive_italic)
    if not gsub_italic:
        return False

    logger.info("Applying ligature GSUB to WarpnineMono fonts...")

    warpnine_fonts = [
        f
        for f in iter_fonts(DIST_DIR, f"{WARPNINE_MONO}-*.ttf")
        if not f.stem.endswith("-VF") and "Test" not in f.stem
    ]

    # Restore backups first
    for font_path in warpnine_fonts:
        backup_path = font_path.with_suffix(".ttf.bak")
        if backup_path.exists():
            shutil.copy2(backup_path, font_path)
            logger.info(f"Restored backup: {font_path.name}")

    success_count = 0
    for font_path in warpnine_fonts:
        is_italic = "Italic" in font_path.stem
        gsub_table = gsub_italic if is_italic else gsub_upright

        logger.info(
            f"Applying {'italic' if is_italic else 'upright'} GSUB to {font_path.name}..."
        )

        try:
            target_font = TTFont(font_path)
            target_font["GSUB"] = gsub_table
            target_font.save(font_path)
            success_count += 1
        except Exception as e:
            logger.error(f"Error applying GSUB to {font_path}: {e}")

    logger.info(f"Applied ligature GSUB to {success_count}/{len(warpnine_fonts)} fonts")
    return success_count == len(warpnine_fonts)
