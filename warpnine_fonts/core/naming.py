"""
Name table manipulation utilities.
"""

from dataclasses import dataclass

from fontTools.ttLib import TTFont

# Copyright text template
COPYRIGHT_TEMPLATE = (
    "Copyright 2020 The Recursive Project Authors (https://github.com/arrowtype/recursive). "
    "Copyright 2014-2021 Adobe (http://www.adobe.com/), with Reserved Font Name 'Source'. "
    "{additional}"
)


@dataclass
class FontNaming:
    """Font naming configuration."""

    family: str  # e.g., "Warpnine Mono"
    style: str  # e.g., "Regular", "Bold"
    postscript_family: str | None = None  # e.g., "WarpnineMono"

    @property
    def full_name(self) -> str:
        """Full font name with family and style."""
        return f"{self.family} {self.style}"

    @property
    def postscript_name(self) -> str:
        """PostScript name (no spaces)."""
        base = self.postscript_family or self.family.replace(" ", "")
        return f"{base}-{self.style.replace(' ', '')}"

    @property
    def unique_id(self) -> str:
        """Unique font identifier."""
        return f"1.0;WARPNINE;{self.postscript_name.replace('-', '')}"


def update_name_table(font: TTFont, naming: FontNaming) -> None:
    """
    Update font name table with consistent naming.

    Args:
        font: TTFont instance to modify
        naming: Naming configuration
    """
    name_table = font["name"]

    for record in name_table.names:
        # nameID 1: Font Family name
        if record.nameID == 1:
            record.string = f"{naming.family} {naming.style}"

        # nameID 3: Unique identifier
        elif record.nameID == 3:
            record.string = naming.unique_id

        # nameID 4: Full font name
        elif record.nameID == 4:
            record.string = naming.full_name

        # nameID 6: PostScript name
        elif record.nameID == 6:
            record.string = naming.postscript_name

        # nameID 16: Typographic Family
        elif record.nameID == 16:
            record.string = naming.family

        # nameID 17: Typographic Subfamily
        elif record.nameID == 17:
            record.string = naming.style


def set_copyright(font: TTFont, additional: str = "") -> None:
    """
    Set copyright notice in name table.

    Args:
        font: TTFont instance to modify
        additional: Additional copyright text to append
    """
    copyright_text = COPYRIGHT_TEMPLATE.format(additional=additional)

    for record in font["name"].names:
        if record.nameID == 0:
            record.string = copyright_text


def update_vf_names(font: TTFont, family: str, postscript_name: str) -> None:
    """
    Update name table for Variable Font.

    Variable fonts don't include style info in some name fields.

    Args:
        font: TTFont instance to modify
        family: Family name (e.g., "Warpnine Mono")
        postscript_name: PostScript name without style (e.g., "WarpnineMono")
    """
    name_table = font["name"]

    for record in name_table.names:
        # nameID 0: Copyright
        if record.nameID == 0:
            copyright_text = COPYRIGHT_TEMPLATE.format(
                additional="Warpnine Mono is based on Recursive Mono Duotone and Noto Sans Mono CJK JP."
            )
            record.string = copyright_text

        # nameID 1: Font Family
        elif record.nameID == 1:
            record.string = family

        # nameID 3: Unique ID
        elif record.nameID == 3:
            record.string = f"1.0;WARPNINE;{postscript_name}"

        # nameID 4: Full Name
        elif record.nameID == 4:
            record.string = family

        # nameID 6: PostScript Name
        elif record.nameID == 6:
            record.string = postscript_name

        # nameID 16: Typographic Family
        elif record.nameID == 16:
            record.string = family

        # nameID 17: Typographic Subfamily
        elif record.nameID == 17:
            record.string = "Regular"
