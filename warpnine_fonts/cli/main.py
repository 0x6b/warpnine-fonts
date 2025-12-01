"""
Main CLI entry point for warpnine-fonts.
"""

import click

from warpnine_fonts import __version__


@click.group()
@click.version_option(version=__version__)
def cli():
    """Warpnine Fonts build system."""
    pass


@cli.group()
def build():
    """Font build commands."""
    pass


@build.command()
def clean():
    """Remove build artifacts (build/ and dist/)."""
    from warpnine_fonts.operations.download import clean as do_clean

    do_clean()


@build.command()
def download():
    """Download source fonts (Recursive VF, Noto CJK)."""
    from warpnine_fonts.operations.download import download as do_download

    do_download()


@build.command("extract-duotone")
def extract_duotone():
    """Extract Duotone instances from Recursive VF."""
    from warpnine_fonts.operations.extract import extract_duotone as do_extract

    do_extract()


@build.command("extract-weights")
def extract_weights():
    """Extract weight instances from Noto CJK VF."""
    from warpnine_fonts.operations.extract import extract_noto_weights

    extract_noto_weights()


@build.command()
def subset():
    """Subset Noto fonts to Japanese Unicode ranges."""
    from warpnine_fonts.operations.subset import subset_noto_fonts

    subset_noto_fonts()


@build.command("remove-ligatures")
def remove_ligatures():
    """Remove three-backtick ligature from Duotone fonts."""
    from warpnine_fonts.operations.merge import remove_ligatures_from_duotone

    remove_ligatures_from_duotone()


@build.command()
def merge():
    """Merge Recursive Duotone with Noto CJK."""
    from warpnine_fonts.operations.merge import merge_all_styles

    merge_all_styles()


@build.command("freeze-static-mono")
def freeze_static_mono():
    """Freeze features in static mono fonts."""
    from warpnine_fonts.operations.freeze import freeze_static_mono as do_freeze

    do_freeze()


@build.command("backup-frozen")
def backup_frozen():
    """Backup frozen static fonts before VF build."""
    from warpnine_fonts.operations.variable import backup_frozen_static_fonts

    backup_frozen_static_fonts()


@build.command("vf")
def build_vf():
    """Build WarpnineMono variable font."""
    from warpnine_fonts.operations.variable import build

    build()


@build.command("copy-gsub")
def copy_gsub():
    """Copy GSUB from Recursive VF to WarpnineMono VF."""
    import sys

    from warpnine_fonts.operations.variable import copy_gsub_to_vf

    if not copy_gsub_to_vf():
        sys.exit(1)


@build.command("restore-frozen")
def restore_frozen():
    """Restore frozen static fonts after VF build."""
    from warpnine_fonts.operations.variable import restore_frozen_static_fonts

    restore_frozen_static_fonts()


@build.command("set-monospace")
def set_monospace():
    """Set monospace flags on all dist fonts."""
    from warpnine_fonts.operations.metadata import set_monospace as do_set

    do_set()


@build.command()
def condensed():
    """Create WarpnineSansCondensed fonts."""
    from warpnine_fonts.operations.sans import create_condensed

    create_condensed()


@build.command()
def sans():
    """Create WarpnineSans fonts."""
    from warpnine_fonts.operations.sans import create_sans

    create_sans()


@build.command("freeze-vf-and-sans")
def freeze_vf_and_sans():
    """Freeze features in VF and sans fonts."""
    from warpnine_fonts.operations.freeze import freeze_vf_and_sans as do_freeze

    do_freeze()


@build.command("set-version")
@click.option(
    "--date",
    "date_string",
    type=str,
    default=None,
    help="Date to embed (YYYY-MM-DD). Defaults to today.",
)
def set_version(date_string):
    """Stamp version date into fonts."""
    from warpnine_fonts.operations.metadata import set_version as do_set

    do_set(date_string)


@build.command()
@click.option(
    "--date",
    "date_string",
    type=str,
    default=None,
    help="Version date (YYYY-MM-DD). Defaults to today.",
)
def all(date_string):
    """Run complete build pipeline."""
    from warpnine_fonts.pipeline.runner import run_all

    run_all(date_string)


@cli.group()
def validate():
    """Font validation commands."""
    pass


@validate.command()
def vf():
    """Validate WarpnineMono variable font."""
    from warpnine_fonts.pipeline.validate import validate_vf

    validate_vf()


@validate.command()
def frozen():
    """Validate frozen features in VF and static mono fonts."""
    from warpnine_fonts.pipeline.validate import validate_frozen

    validate_frozen()


@validate.command("all")
def validate_all():
    """Run all validation tests (vf + frozen)."""
    from warpnine_fonts.pipeline.validate import validate_frozen, validate_vf

    validate_vf()
    validate_frozen()


if __name__ == "__main__":
    cli()
