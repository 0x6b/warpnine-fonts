"""
OpenType feature freeze configurations.

Defines which features to permanently apply to each font family.
"""

from pathlib import Path

# Features to freeze for WarpnineMono (static and variable)
MONO_FEATURES = [
    "dlig",  # Discretionary ligatures
    "ss01",  # Stylistic set 1 (Single-story a)
    "ss02",  # Stylistic set 2 (Single-story g)
    "ss03",  # Stylistic set 3 (Simplified f)
    "ss04",  # Stylistic set 4 (Simplified i)
    "ss05",  # Stylistic set 5 (Simplified l)
    "ss06",  # Stylistic set 6 (Simplified r)
    "ss07",  # Stylistic set 7 (Simplified italic diagonals)
    "ss08",  # Stylistic set 8 (No-serif L and Z)
    "ss10",  # Stylistic set 10 (Dotted zero)
    "ss11",  # Stylistic set 11 (Simplified 1)
    "ss12",  # Stylistic set 12 (Simplified @)
    "pnum",  # Proportional numerals
    "liga",  # Standard ligatures
]

# Features to freeze for WarpnineSans and WarpnineSansCondensed
SANS_FEATURES = [
    "ss01",  # Stylistic set 1
    "ss02",  # Stylistic set 2
    "ss03",  # Stylistic set 3
    "ss04",  # Stylistic set 4
    "ss05",  # Stylistic set 5
    "ss06",  # Stylistic set 6
    "ss07",  # Stylistic set 7
    "ss08",  # Stylistic set 8
    "ss10",  # Stylistic set 10
    "ss12",  # Stylistic set 12
    "case",  # Case-sensitive forms
    "pnum",  # Proportional numerals
    "liga",  # Standard ligatures
]


def get_font_configs(dist_dir: Path) -> list[tuple[Path, str, list[str]]]:
    """
    Get font freeze configurations.

    Returns list of (directory, glob_pattern, features) tuples.
    """
    return [
        # Static mono fonts (exclude VF)
        (dist_dir, "WarpnineMono-[!V]*.ttf", MONO_FEATURES),
        # Variable font
        (dist_dir, "WarpnineMono-VF.ttf", MONO_FEATURES),
        # Sans condensed
        (dist_dir, "WarpnineSansCondensed-*.ttf", SANS_FEATURES),
        # Sans (non-condensed, exclude condensed)
        (dist_dir, "WarpnineSans-*.ttf", SANS_FEATURES),
    ]
