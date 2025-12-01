"""Tests for instance configuration."""

from warpnine_fonts.config.instances import (
    DUOTONE_INSTANCES,
    SANS_INSTANCES,
    DuotoneInstance,
    NotoWeight,
    Weight,
    get_noto_weight_for_style,
)


def test_weight_enum_values():
    """Test Weight enum has correct values."""
    assert Weight.LIGHT == 300
    assert Weight.REGULAR == 400
    assert Weight.BOLD == 700
    assert Weight.EXTRABLACK == 1000


def test_duotone_instance_count():
    """Test we have all 16 Duotone instances."""
    assert len(DUOTONE_INSTANCES) == 16


def test_duotone_instance_axis_values():
    """Test Duotone instance axis value computation."""
    # Regular upright
    regular = next(i for i in DUOTONE_INSTANCES if i.style == "Regular")
    assert regular.mono == 1.0
    assert regular.casl == 0.0  # Linear for Regular
    assert regular.wght == 400.0
    assert regular.slnt == 0.0
    assert regular.crsv == 0.5

    # Bold italic
    bold_italic = next(i for i in DUOTONE_INSTANCES if i.style == "BoldItalic")
    assert bold_italic.mono == 1.0
    assert bold_italic.casl == 1.0  # Casual for Bold
    assert bold_italic.wght == 700.0
    assert bold_italic.slnt == -15.0
    assert bold_italic.crsv == 1.0


def test_duotone_output_name():
    """Test Duotone output filename generation."""
    instance = DuotoneInstance("Regular", Weight.REGULAR, casual=False)
    assert instance.output_name == "RecMonoDuotone-Regular.ttf"

    instance = DuotoneInstance("BoldItalic", Weight.BOLD, italic=True, casual=True)
    assert instance.output_name == "RecMonoDuotone-BoldItalic.ttf"


def test_sans_instance_count():
    """Test we have all 14 Sans instances."""
    assert len(SANS_INSTANCES) == 14


def test_noto_weight_mapping():
    """Test Noto weight mapping for styles."""
    assert get_noto_weight_for_style("Light") == NotoWeight.REGULAR
    assert get_noto_weight_for_style("Regular") == NotoWeight.REGULAR
    assert get_noto_weight_for_style("Medium") == NotoWeight.REGULAR
    assert get_noto_weight_for_style("SemiBold") == NotoWeight.BOLD
    assert get_noto_weight_for_style("Bold") == NotoWeight.BOLD
    assert get_noto_weight_for_style("BoldItalic") == NotoWeight.BOLD
