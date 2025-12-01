"""
Font I/O utilities for loading, saving, and traversing font files.
"""

from collections.abc import Callable, Iterator
from contextlib import contextmanager
from pathlib import Path

from fontTools.ttLib import TTFont


def iter_fonts(
    directory: Path,
    pattern: str = "*.ttf",
    exclude_patterns: list[str] | None = None,
) -> Iterator[Path]:
    """
    Iterate over font files matching pattern, sorted by name.

    Args:
        directory: Directory to search
        pattern: Glob pattern to match
        exclude_patterns: Substrings to exclude from filenames

    Yields:
        Paths to matching font files
    """
    fonts = sorted(directory.glob(pattern))
    if exclude_patterns:
        fonts = [f for f in fonts if not any(p in f.name for p in exclude_patterns)]
    return iter(fonts)


def process_fonts(
    directory: Path,
    processor: Callable[[TTFont, Path], None],
    pattern: str = "*.ttf",
    *,
    save: bool = True,
) -> int:
    """
    Process all fonts in directory with given function.

    Args:
        directory: Directory containing fonts
        processor: Function to apply to each font
        pattern: Glob pattern for font files
        save: Whether to save fonts after processing

    Returns:
        Number of fonts processed
    """
    count = 0
    for font_path in iter_fonts(directory, pattern):
        font = TTFont(font_path)
        processor(font, font_path)
        if save:
            font.save(font_path)
        font.close()
        count += 1
    return count


@contextmanager
def open_font(path: Path, *, save_on_exit: bool = True) -> Iterator[TTFont]:
    """
    Context manager for font operations with automatic save.

    Args:
        path: Path to font file
        save_on_exit: Whether to save on successful exit

    Yields:
        TTFont instance
    """
    font = TTFont(path)
    try:
        yield font
        if save_on_exit:
            font.save(path)
    finally:
        font.close()


def get_font_size_mb(path: Path) -> float:
    """Get font file size in megabytes."""
    return path.stat().st_size / 1024 / 1024
