"""
Consolidated font instance definitions.

Defines all weight, style, and axis configurations for font extraction.
"""

from dataclasses import dataclass
from enum import IntEnum


class Weight(IntEnum):
    """Font weight values matching OpenType usWeightClass."""

    LIGHT = 300
    REGULAR = 400
    MEDIUM = 500
    SEMIBOLD = 600
    BOLD = 700
    EXTRABOLD = 800
    BLACK = 900
    EXTRABLACK = 1000


class NotoWeight(IntEnum):
    """Noto CJK weights available for merging."""

    REGULAR = 400
    BOLD = 700


@dataclass(frozen=True)
class DuotoneInstance:
    """
    Recursive Mono Duotone instance configuration.

    Duotone style uses:
    - Linear (CASL=0) for Light/Regular weights
    - Casual (CASL=1) for Medium and heavier weights
    """

    style: str
    weight: Weight
    italic: bool = False
    casual: bool = True  # True for Casual (CASL=1), False for Linear (CASL=0)

    @property
    def output_name(self) -> str:
        """Generate output filename."""
        return f"RecMonoDuotone-{self.style}.ttf"

    @property
    def mono(self) -> float:
        """MONO axis value (always 1.0 for monospace)."""
        return 1.0

    @property
    def casl(self) -> float:
        """CASL axis value."""
        return 1.0 if self.casual else 0.0

    @property
    def wght(self) -> float:
        """wght axis value."""
        return float(self.weight)

    @property
    def slnt(self) -> float:
        """slnt axis value."""
        return -15.0 if self.italic else 0.0

    @property
    def crsv(self) -> float:
        """CRSV axis value."""
        return 1.0 if self.italic else 0.5


# All 16 Duotone instances
# Light/Regular: Linear (CASL=0) for traditional monospace appearance
# Medium and heavier: Casual (CASL=1) for better readability at bold weights
DUOTONE_INSTANCES = [
    # Light - Linear
    DuotoneInstance("Light", Weight.LIGHT, casual=False),
    DuotoneInstance("LightItalic", Weight.LIGHT, italic=True, casual=False),
    # Regular - Linear
    DuotoneInstance("Regular", Weight.REGULAR, casual=False),
    DuotoneInstance("Italic", Weight.REGULAR, italic=True, casual=False),
    # Medium - Casual
    DuotoneInstance("Medium", Weight.MEDIUM, casual=True),
    DuotoneInstance("MediumItalic", Weight.MEDIUM, italic=True, casual=True),
    # SemiBold - Casual
    DuotoneInstance("SemiBold", Weight.SEMIBOLD, casual=True),
    DuotoneInstance("SemiBoldItalic", Weight.SEMIBOLD, italic=True, casual=True),
    # Bold - Casual
    DuotoneInstance("Bold", Weight.BOLD, casual=True),
    DuotoneInstance("BoldItalic", Weight.BOLD, italic=True, casual=True),
    # ExtraBold - Casual
    DuotoneInstance("ExtraBold", Weight.EXTRABOLD, casual=True),
    DuotoneInstance("ExtraBoldItalic", Weight.EXTRABOLD, italic=True, casual=True),
    # Black - Casual
    DuotoneInstance("Black", Weight.BLACK, casual=True),
    DuotoneInstance("BlackItalic", Weight.BLACK, italic=True, casual=True),
    # ExtraBlack - Casual
    DuotoneInstance("ExtraBlack", Weight.EXTRABLACK, casual=True),
    DuotoneInstance("ExtraBlackItalic", Weight.EXTRABLACK, italic=True, casual=True),
]


@dataclass(frozen=True)
class SansInstance:
    """
    Recursive Sans instance configuration.

    Sans uses MONO=0 (proportional) and CASL=0 (Linear).
    """

    style: str
    weight: Weight
    italic: bool = False

    @property
    def output_name(self) -> str:
        """Generate output filename."""
        return f"RecSansLinear-{self.style}.ttf"

    @property
    def mono(self) -> float:
        """MONO axis value (0 for proportional)."""
        return 0.0

    @property
    def casl(self) -> float:
        """CASL axis value (0 for Linear)."""
        return 0.0

    @property
    def wght(self) -> float:
        """wght axis value."""
        return float(self.weight)

    @property
    def slnt(self) -> float:
        """slnt axis value."""
        return -15.0 if self.italic else 0.0

    @property
    def crsv(self) -> float:
        """CRSV axis value."""
        return 1.0 if self.italic else 0.5


# Sans instances (14 total: 7 weights x 2 styles)
SANS_INSTANCES = [
    SansInstance("Light", Weight.LIGHT),
    SansInstance("LightItalic", Weight.LIGHT, italic=True),
    SansInstance("Regular", Weight.REGULAR),
    SansInstance("Italic", Weight.REGULAR, italic=True),
    SansInstance("Medium", Weight.MEDIUM),
    SansInstance("MediumItalic", Weight.MEDIUM, italic=True),
    SansInstance("SemiBold", Weight.SEMIBOLD),
    SansInstance("SemiBoldItalic", Weight.SEMIBOLD, italic=True),
    SansInstance("Bold", Weight.BOLD),
    SansInstance("BoldItalic", Weight.BOLD, italic=True),
    SansInstance("ExtraBold", Weight.EXTRABOLD),
    SansInstance("ExtraBoldItalic", Weight.EXTRABOLD, italic=True),
    SansInstance("Black", Weight.BLACK),
    SansInstance("BlackItalic", Weight.BLACK, italic=True),
]


def get_noto_weight_for_style(style: str) -> NotoWeight:
    """
    Get the Noto CJK weight to use for a given style.

    Lighter weights (Light, Regular, Medium) use Noto 400.
    Heavier weights (SemiBold+) use Noto 700.
    """
    # Extract base weight from style name (remove "Italic" suffix)
    base_style = style.replace("Italic", "")

    light_styles = {"Light", "Regular", "Medium", ""}
    return NotoWeight.REGULAR if base_style in light_styles else NotoWeight.BOLD
