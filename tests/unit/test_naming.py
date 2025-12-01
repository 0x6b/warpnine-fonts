"""Tests for naming utilities."""

from warpnine_fonts.core.naming import FontNaming


def test_font_naming_full_name():
    """Test FontNaming.full_name property."""
    naming = FontNaming("Warpnine Mono", "Regular")
    assert naming.full_name == "Warpnine Mono Regular"


def test_font_naming_postscript_name():
    """Test FontNaming.postscript_name property."""
    naming = FontNaming("Warpnine Mono", "Regular")
    assert naming.postscript_name == "WarpnineMono-Regular"

    naming = FontNaming("Warpnine Mono", "Bold Italic")
    assert naming.postscript_name == "WarpnineMono-BoldItalic"


def test_font_naming_postscript_family_override():
    """Test FontNaming with custom postscript_family."""
    naming = FontNaming("Warpnine Mono", "Regular", postscript_family="WMono")
    assert naming.postscript_name == "WMono-Regular"


def test_font_naming_unique_id():
    """Test FontNaming.unique_id property."""
    naming = FontNaming("Warpnine Mono", "Regular")
    assert naming.unique_id == "1.0;WARPNINE;WarpnineMonoRegular"
