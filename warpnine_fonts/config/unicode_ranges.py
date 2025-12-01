"""
Unicode range definitions for font subsetting.

Reference: https://www.unicode.org/charts/
Note: pyftsubset requires hex format without U+ prefix.
"""

# Basic Japanese Unicode ranges
JAPANESE_RANGES_BASIC = [
    # Symbols and punctuation
    "3000-303F",  # CJK Symbols and Punctuation
    # Hiragana
    "3041-3096",  # Hiragana (basic)
    "3099-309F",  # Hiragana (combining marks)
    # Katakana
    "30A0-30FF",  # Katakana
    # Kanji (basic)
    "4E00-9FFF",  # CJK Unified Ideographs
    # Fullwidth alphanumeric and symbols
    "FF00-FFEF",  # Halfwidth and Fullwidth Forms
]

# Extended Japanese Unicode ranges
JAPANESE_RANGES_EXTENDED = [
    # Hiragana extension
    "1B100-1B12F",  # Kana Extended-A
    "1AFF0-1AFFF",  # Kana Extended-B
    "1B000-1B0FF",  # Kana Supplement
    "1B130-1B16F",  # Small Kana Extension
    # Kanji extension
    "3400-4DBF",  # CJK Unified Ideographs Extension A
    "20000-2A6DF",  # CJK Unified Ideographs Extension B
    "2A700-2B739",  # CJK Unified Ideographs Extension C
    "2B740-2B81D",  # CJK Unified Ideographs Extension D
    "2B820-2CEA1",  # CJK Unified Ideographs Extension E
    "2CEB0-2EBE0",  # CJK Unified Ideographs Extension F
    "30000-3134A",  # CJK Unified Ideographs Extension G
    "31350-323AF",  # CJK Unified Ideographs Extension H
    "2EBF0-2EE5D",  # CJK Unified Ideographs Extension I
    "F900-FAFF",  # CJK Compatibility Ideographs
    "2F800-2FA1F",  # CJK Compatibility Ideographs Supplement
]

# Full Japanese ranges (basic + extended)
JAPANESE_RANGES = JAPANESE_RANGES_BASIC + JAPANESE_RANGES_EXTENDED

# Variable font tables to drop during subsetting
VF_TABLES_TO_DROP = ["HVAR", "MVAR", "STAT", "avar", "fvar", "gvar", "cvar"]
