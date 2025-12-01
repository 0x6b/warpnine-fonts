"""
GSUB table operations.
"""

from fontTools.ttLib import TTFont

from warpnine_fonts.utils.logging import logger


def fix_calt_registration(font: TTFont) -> None:
    """
    Ensure calt/rclt features are registered to all scripts.

    After font merging with fontforge, calt is only registered to DFLT script.
    This causes calt to not be applied when displaying Latin characters in browsers.

    Args:
        font: TTFont instance to modify
    """
    if "GSUB" not in font:
        return

    gsub = font["GSUB"].table

    if not hasattr(gsub, "ScriptList") or not hasattr(gsub, "FeatureList"):
        return

    # Search for calt and rclt feature indices
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

        # Add to DefaultLangSys
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

            script.DefaultLangSys.FeatureIndex = features
            script.DefaultLangSys.FeatureCount = len(features)

        # Add to language-specific systems (JAN, KOR, ZHH, ZHS, ZHT, etc.)
        # This makes calt effective even when lang="ja" etc. is specified in browsers
        if hasattr(script, "LangSysRecord") and script.LangSysRecord:
            for lang_record in script.LangSysRecord:
                lang_sys = lang_record.LangSys
                if lang_sys.FeatureIndex is not None:
                    features = list(lang_sys.FeatureIndex)

                    for calt_idx in calt_indices:
                        if calt_idx not in features:
                            insert_pos = 1 if len(features) > 1 else len(features)
                            features.insert(insert_pos, calt_idx)

                    for rclt_idx in rclt_indices:
                        if rclt_idx not in features:
                            features.append(rclt_idx)

                    lang_sys.FeatureIndex = features
                    lang_sys.FeatureCount = len(features)


def remove_gsub_table(font: TTFont) -> bool:
    """
    Remove GSUB table from a font.

    Args:
        font: TTFont instance to modify

    Returns:
        True if GSUB was removed, False if it didn't exist
    """
    if "GSUB" in font:
        del font["GSUB"]
        return True
    return False


def copy_gsub_from(target: TTFont, source: TTFont) -> None:
    """
    Copy GSUB table from source font to target font.

    This is used to restore GSUB with FeatureVariations from the original
    Recursive VF after building a variable font.

    Args:
        target: Target font to receive GSUB
        source: Source font to copy GSUB from
    """
    if "GSUB" not in source:
        logger.warning("Source font has no GSUB table")
        return

    target["GSUB"] = source["GSUB"]
    logger.info("Copied GSUB table from source font")
