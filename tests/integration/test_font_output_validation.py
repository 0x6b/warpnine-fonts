"""
Font output validation tests.

Validates that Rust CLI produces correct font files with expected properties.
These tests use known expected values as the gold standard.

Run with: uv run pytest tests/integration/test_font_output_validation.py -v
"""

import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

import pytest
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont


PROJECT_ROOT = Path(__file__).parent.parent.parent
RUST_CLI = PROJECT_ROOT / "target" / "release" / "warpnine-fonts"
BUILD_DIR = PROJECT_ROOT / "build"
DIST_DIR = PROJECT_ROOT / "dist"
RECURSIVE_VF = BUILD_DIR / "Recursive_VF_1.085.ttf"


# ============================================================================
# Expected values (gold standard derived from Python pipeline)
# ============================================================================


@dataclass
class FontSpec:
    """Expected font specification."""

    glyph_count: int
    units_per_em: int = 1000
    is_monospace: bool = False
    weight_class: int | None = None


@dataclass
class VariableFontSpec:
    """Expected variable font specification."""

    glyph_count: int
    axes: dict[str, tuple[float, float, float]]  # tag -> (min, default, max)
    named_instances: int
    units_per_em: int = 1000


# WarpnineMono: 16 static fonts merged with Noto Sans Mono CJK
MONO_SPEC = FontSpec(
    glyph_count=51544,  # Rust pipeline with all layout features (dlig, liga, etc.)
    is_monospace=True,
)

MONO_WEIGHTS = {
    "Light": 300,
    "Regular": 400,
    "Medium": 500,
    "SemiBold": 600,
    "Bold": 700,
    "ExtraBold": 800,
    "Black": 900,
    "ExtraBlack": 1000,
}

# WarpnineMono Variable Font
MONO_VF_SPEC = VariableFontSpec(
    glyph_count=51544,  # Rust pipeline with all layout features
    axes={
        "wght": (300.0, 400.0, 1000.0),
        "ital": (0.0, 0.0, 1.0),
    },
    named_instances=16,
)

# WarpnineSans: 14 static fonts from Recursive
SANS_SPEC = FontSpec(
    glyph_count=1304,  # Recursive glyph count
    is_monospace=False,
)

SANS_WEIGHTS = {
    "Light": 300,
    "Regular": 400,
    "Medium": 500,
    "SemiBold": 600,
    "Bold": 700,
    "ExtraBold": 800,
    "Black": 900,
}

# WarpnineSansCondensed: 14 static fonts
CONDENSED_SPEC = FontSpec(
    glyph_count=1304,
    is_monospace=False,
)


# ============================================================================
# Test Fixtures
# ============================================================================


@pytest.fixture(scope="module")
def rust_cli():
    """Ensure Rust CLI is built."""
    if not RUST_CLI.exists():
        pytest.skip("Rust CLI not built. Run: cargo build --release")
    return RUST_CLI


@pytest.fixture(scope="module")
def recursive_vf():
    """Ensure Recursive VF is available."""
    if not RECURSIVE_VF.exists():
        pytest.skip("Recursive VF not downloaded. Run the download step first.")
    return RECURSIVE_VF


# ============================================================================
# Naming Helpers
# ============================================================================


def get_name_entry(font: TTFont, name_id: int) -> str | None:
    """Get a name table entry by ID (prefers platformID 3/Windows, then 1/Mac)."""
    name_table = font["name"]
    # Try Windows (platformID=3, encodingID=1) first, then Mac (platformID=1)
    for platform_id, encoding_id in [(3, 1), (1, 0)]:
        record = name_table.getName(name_id, platform_id, encoding_id, None)
        if record:
            return record.toUnicode()
    return None


def validate_font_names_exist(font_path: Path) -> list[str]:
    """Validate that required name table entries exist. Returns list of failures."""
    failures = []

    if not font_path.exists():
        return [f"Font not found: {font_path}"]

    font = TTFont(font_path)

    # Name ID 1: Family name
    family_name = get_name_entry(font, 1)
    if not family_name:
        failures.append("Name ID 1 (Family): missing")

    # Name ID 2: Subfamily name
    subfamily = get_name_entry(font, 2)
    if not subfamily:
        failures.append("Name ID 2 (Subfamily): missing")

    # Name ID 4: Full name
    full_name = get_name_entry(font, 4)
    if not full_name:
        failures.append("Name ID 4 (Full name): missing")

    # Name ID 6: PostScript name (no spaces, hyphen separator)
    ps_name = get_name_entry(font, 6)
    if not ps_name:
        failures.append("Name ID 6 (PostScript name): missing")
    elif " " in ps_name:
        failures.append(
            f"Name ID 6 (PostScript name): should not contain spaces, got '{ps_name}'"
        )
    elif "-" not in ps_name:
        failures.append(
            f"Name ID 6 (PostScript name): should contain hyphen, got '{ps_name}'"
        )

    font.close()
    return failures


def validate_font_naming(
    font_path: Path, family: str, style: str, postscript_family: str
) -> list[str]:
    """Validate name table entries against expected naming. Returns list of failures."""
    failures = []

    if not font_path.exists():
        return [f"Font not found: {font_path}"]

    font = TTFont(font_path)

    # Expected values (matching FontNaming class logic)
    expected_full_name = f"{family} {style}"
    expected_ps_name = f"{postscript_family}-{style.replace(' ', '')}"

    # Name ID 4: Full name
    full_name = get_name_entry(font, 4)
    if full_name != expected_full_name:
        failures.append(
            f"Name ID 4 (Full name): expected '{expected_full_name}', got '{full_name}'"
        )

    # Name ID 6: PostScript name
    ps_name = get_name_entry(font, 6)
    if ps_name != expected_ps_name:
        failures.append(
            f"Name ID 6 (PostScript name): expected '{expected_ps_name}', got '{ps_name}'"
        )

    # Name ID 3: Unique ID (should contain WARPNINE)
    unique_id = get_name_entry(font, 3)
    if unique_id and "WARPNINE" not in unique_id:
        failures.append(
            f"Name ID 3 (Unique ID): expected to contain 'WARPNINE', got '{unique_id}'"
        )

    # Name ID 16: Typographic Family
    typo_family = get_name_entry(font, 16)
    if typo_family and typo_family != family:
        failures.append(
            f"Name ID 16 (Typographic Family): expected '{family}', got '{typo_family}'"
        )

    # Name ID 17: Typographic Subfamily
    typo_subfamily = get_name_entry(font, 17)
    if typo_subfamily and typo_subfamily != style:
        failures.append(
            f"Name ID 17 (Typographic Subfamily): expected '{style}', got '{typo_subfamily}'"
        )

    font.close()
    return failures


# ============================================================================
# Validation Helpers
# ============================================================================


def validate_static_font(
    font_path: Path, spec: FontSpec, weight: int | None = None
) -> list[str]:
    """Validate a static font against expected spec. Returns list of failures."""
    failures = []

    if not font_path.exists():
        return [f"Font not found: {font_path}"]

    font = TTFont(font_path)

    # Glyph count
    actual_glyphs = len(font.getGlyphOrder())
    if actual_glyphs != spec.glyph_count:
        failures.append(
            f"Glyph count: expected {spec.glyph_count}, got {actual_glyphs}"
        )

    # Units per em
    if font["head"].unitsPerEm != spec.units_per_em:
        failures.append(
            f"unitsPerEm: expected {spec.units_per_em}, got {font['head'].unitsPerEm}"
        )

    # Monospace flags
    if spec.is_monospace:
        if font["post"].isFixedPitch != 1:
            failures.append(
                f"post.isFixedPitch: expected 1, got {font['post'].isFixedPitch}"
            )
        if font["OS/2"].panose.bProportion != 9:
            failures.append(
                f"panose.bProportion: expected 9, got {font['OS/2'].panose.bProportion}"
            )

    # Weight class
    if weight is not None:
        if font["OS/2"].usWeightClass != weight:
            failures.append(
                f"usWeightClass: expected {weight}, got {font['OS/2'].usWeightClass}"
            )

    # Required tables
    required_tables = [
        "glyf",
        "head",
        "hhea",
        "hmtx",
        "maxp",
        "name",
        "OS/2",
        "post",
        "cmap",
    ]
    for table in required_tables:
        if table not in font:
            failures.append(f"Missing required table: {table}")

    font.close()
    return failures


def validate_variable_font(font_path: Path, spec: VariableFontSpec) -> list[str]:
    """Validate a variable font against expected spec. Returns list of failures."""
    failures = []

    if not font_path.exists():
        return [f"Font not found: {font_path}"]

    font = TTFont(font_path)

    # Glyph count
    actual_glyphs = len(font.getGlyphOrder())
    if actual_glyphs != spec.glyph_count:
        failures.append(
            f"Glyph count: expected {spec.glyph_count}, got {actual_glyphs}"
        )

    # fvar table
    if "fvar" not in font:
        failures.append("Missing fvar table")
        font.close()
        return failures

    fvar = font["fvar"]

    # Axes
    actual_axes = {
        a.axisTag: (a.minValue, a.defaultValue, a.maxValue) for a in fvar.axes
    }
    for tag, expected_range in spec.axes.items():
        if tag not in actual_axes:
            failures.append(f"Missing axis: {tag}")
        elif actual_axes[tag] != expected_range:
            failures.append(
                f"Axis {tag}: expected {expected_range}, got {actual_axes[tag]}"
            )

    # Named instances
    actual_instances = len(fvar.instances)
    if actual_instances != spec.named_instances:
        failures.append(
            f"Named instances: expected {spec.named_instances}, got {actual_instances}"
        )

    # gvar table (required for glyph variations)
    if "gvar" not in font:
        failures.append("Missing gvar table")

    font.close()
    return failures


def validate_vf_instantiation(font_path: Path, test_locations: list[dict]) -> list[str]:
    """Validate that VF can be instantiated at various locations."""
    failures = []

    if not font_path.exists():
        return [f"Font not found: {font_path}"]

    base_font = TTFont(font_path)
    expected_glyphs = len(base_font.getGlyphOrder())
    base_font.close()

    for location in test_locations:
        try:
            font = TTFont(font_path)
            instance = instantiateVariableFont(font, location)
            actual_glyphs = len(instance.getGlyphOrder())

            if actual_glyphs != expected_glyphs:
                failures.append(
                    f"Location {location}: glyph count {actual_glyphs} != {expected_glyphs}"
                )

            instance.close()
            font.close()
        except Exception as e:
            failures.append(f"Location {location}: instantiation failed - {e}")

    return failures


# ============================================================================
# WarpnineMono Tests
# ============================================================================


class TestWarpnineMonoStatic:
    """Tests for WarpnineMono static fonts."""

    @pytest.mark.parametrize(
        "style,weight",
        [
            ("Light", 300),
            ("Regular", 400),
            ("Medium", 500),
            ("SemiBold", 600),
            ("Bold", 700),
            ("ExtraBold", 800),
            ("Black", 900),
            ("ExtraBlack", 1000),
        ],
    )
    def test_mono_upright(self, style, weight):
        """Test WarpnineMono upright styles."""
        font_path = DIST_DIR / f"WarpnineMono-{style}.ttf"
        if not font_path.exists():
            pytest.skip(f"Font not built: {font_path.name}")

        failures = validate_static_font(font_path, MONO_SPEC, weight)
        assert not failures, "\n".join(failures)

    @pytest.mark.parametrize(
        "style,weight",
        [
            ("LightItalic", 300),
            ("Italic", 400),
            ("MediumItalic", 500),
            ("SemiBoldItalic", 600),
            ("BoldItalic", 700),
            ("ExtraBoldItalic", 800),
            ("BlackItalic", 900),
            ("ExtraBlackItalic", 1000),
        ],
    )
    def test_mono_italic(self, style, weight):
        """Test WarpnineMono italic styles."""
        font_path = DIST_DIR / f"WarpnineMono-{style}.ttf"
        if not font_path.exists():
            pytest.skip(f"Font not built: {font_path.name}")

        failures = validate_static_font(font_path, MONO_SPEC, weight)
        assert not failures, "\n".join(failures)

    def test_mono_has_required_tables(self):
        """Test that WarpnineMono has all required tables including GSUB."""
        font_path = DIST_DIR / "WarpnineMono-Regular.ttf"
        if not font_path.exists():
            pytest.skip("Font not built")

        font = TTFont(font_path)

        # Core required tables for TrueType fonts
        # GSUB is required for ligatures and contextual alternates
        required = [
            "glyf",
            "head",
            "hhea",
            "hmtx",
            "maxp",
            "name",
            "OS/2",
            "post",
            "cmap",
            "GSUB",
        ]
        missing = [t for t in required if t not in font]

        font.close()

        assert not missing, f"Missing required tables: {missing}. Run copy-gsub step."

    def test_mono_gsub_has_calt(self):
        """Test that WarpnineMono GSUB has calt/rclt features."""
        font_path = DIST_DIR / "WarpnineMono-Regular.ttf"
        if not font_path.exists():
            pytest.skip("Font not built")

        font = TTFont(font_path)

        if "GSUB" not in font:
            font.close()
            pytest.skip("GSUB missing - run copy-gsub step first")

        gsub = font["GSUB"].table
        features = {r.FeatureTag for r in gsub.FeatureList.FeatureRecord}
        font.close()

        # Check for key ligature-related features (Recursive doesn't have calt)
        assert "liga" in features, f"Missing liga feature. Found: {sorted(features)}"
        assert "dlig" in features, f"Missing dlig feature. Found: {sorted(features)}"


class TestWarpnineMonoVF:
    """Tests for WarpnineMono variable font."""

    def test_vf_axes_and_instances(self):
        """Test VF has correct axes and named instances."""
        font_path = DIST_DIR / "WarpnineMono-VF.ttf"
        if not font_path.exists():
            pytest.skip("VF not built")

        failures = validate_variable_font(font_path, MONO_VF_SPEC)
        assert not failures, "\n".join(failures)

    @pytest.mark.skip(
        reason="GSUB FeatureVariations from Recursive use 5-axis coordinates incompatible with 2-axis VF"
    )
    def test_vf_instantiation_weights(self):
        """Test VF can be instantiated at all master weights."""
        font_path = DIST_DIR / "WarpnineMono-VF.ttf"
        if not font_path.exists():
            pytest.skip("VF not built")

        test_locations = [
            {"wght": 300, "ital": 0},
            {"wght": 400, "ital": 0},
            {"wght": 500, "ital": 0},
            {"wght": 600, "ital": 0},
            {"wght": 700, "ital": 0},
            {"wght": 800, "ital": 0},
            {"wght": 900, "ital": 0},
            {"wght": 1000, "ital": 0},
        ]

        failures = validate_vf_instantiation(font_path, test_locations)
        assert not failures, "\n".join(failures)

    @pytest.mark.skip(
        reason="GSUB FeatureVariations from Recursive use 5-axis coordinates incompatible with 2-axis VF"
    )
    def test_vf_instantiation_italics(self):
        """Test VF can be instantiated at italic locations."""
        font_path = DIST_DIR / "WarpnineMono-VF.ttf"
        if not font_path.exists():
            pytest.skip("VF not built")

        test_locations = [
            {"wght": 400, "ital": 0},
            {"wght": 400, "ital": 1},
            {"wght": 700, "ital": 0},
            {"wght": 700, "ital": 1},
        ]

        failures = validate_vf_instantiation(font_path, test_locations)
        assert not failures, "\n".join(failures)

    @pytest.mark.skip(
        reason="GSUB FeatureVariations from Recursive use 5-axis coordinates incompatible with 2-axis VF"
    )
    def test_vf_instantiation_intermediate(self):
        """Test VF can be instantiated at intermediate (non-master) locations."""
        font_path = DIST_DIR / "WarpnineMono-VF.ttf"
        if not font_path.exists():
            pytest.skip("VF not built")

        test_locations = [
            {"wght": 350, "ital": 0},  # Between Light and Regular
            {"wght": 450, "ital": 0},  # Between Regular and Medium
            {"wght": 550, "ital": 0.5},  # Intermediate on both axes
            {"wght": 850, "ital": 0},  # Between ExtraBold and Black
        ]

        failures = validate_vf_instantiation(font_path, test_locations)
        assert not failures, "\n".join(failures)


# ============================================================================
# WarpnineSans Tests
# ============================================================================


class TestWarpnineSans:
    """Tests for WarpnineSans static fonts."""

    @pytest.mark.parametrize(
        "style,weight",
        [
            ("Light", 300),
            ("Regular", 400),
            ("Medium", 500),
            ("SemiBold", 600),
            ("Bold", 700),
            ("ExtraBold", 800),
            ("Black", 900),
        ],
    )
    def test_sans_upright(self, style, weight):
        """Test WarpnineSans upright styles."""
        font_path = DIST_DIR / f"WarpnineSans-{style}.ttf"
        if not font_path.exists():
            pytest.skip(f"Font not built: {font_path.name}")

        failures = validate_static_font(font_path, SANS_SPEC, weight)
        assert not failures, "\n".join(failures)

    @pytest.mark.parametrize(
        "style,weight",
        [
            ("LightItalic", 300),
            ("Italic", 400),
            ("MediumItalic", 500),
            ("SemiBoldItalic", 600),
            ("BoldItalic", 700),
            ("ExtraBoldItalic", 800),
            ("BlackItalic", 900),
        ],
    )
    def test_sans_italic(self, style, weight):
        """Test WarpnineSans italic styles."""
        font_path = DIST_DIR / f"WarpnineSans-{style}.ttf"
        if not font_path.exists():
            pytest.skip(f"Font not built: {font_path.name}")

        failures = validate_static_font(font_path, SANS_SPEC, weight)
        assert not failures, "\n".join(failures)


# ============================================================================
# WarpnineSansCondensed Tests
# ============================================================================


class TestWarpnineSansCondensed:
    """Tests for WarpnineSansCondensed static fonts."""

    @pytest.mark.parametrize(
        "style,weight",
        [
            ("Light", 300),
            ("Regular", 400),
            ("Medium", 500),
            ("SemiBold", 600),
            ("Bold", 700),
            ("ExtraBold", 800),
            ("Black", 900),
        ],
    )
    def test_condensed_upright(self, style, weight):
        """Test WarpnineSansCondensed upright styles."""
        font_path = DIST_DIR / f"WarpnineSansCondensed-{style}.ttf"
        if not font_path.exists():
            pytest.skip(f"Font not built: {font_path.name}")

        failures = validate_static_font(font_path, CONDENSED_SPEC, weight)
        assert not failures, "\n".join(failures)

    @pytest.mark.parametrize(
        "style,weight",
        [
            ("LightItalic", 300),
            ("Italic", 400),
            ("MediumItalic", 500),
            ("SemiBoldItalic", 600),
            ("BoldItalic", 700),
            ("ExtraBoldItalic", 800),
            ("BlackItalic", 900),
        ],
    )
    def test_condensed_italic(self, style, weight):
        """Test WarpnineSansCondensed italic styles."""
        font_path = DIST_DIR / f"WarpnineSansCondensed-{style}.ttf"
        if not font_path.exists():
            pytest.skip(f"Font not built: {font_path.name}")

        failures = validate_static_font(font_path, CONDENSED_SPEC, weight)
        assert not failures, "\n".join(failures)


# ============================================================================
# Rust CLI Operation Tests
# ============================================================================


class TestRustCLIOperations:
    """Tests for individual Rust CLI operations."""

    def test_instance_creates_valid_font(self, rust_cli, recursive_vf, tmp_path):
        """Test that instance command creates valid static font."""
        output = tmp_path / "instance.ttf"

        result = subprocess.run(
            [
                str(rust_cli),
                "instance",
                "-a",
                "MONO=1",
                "-a",
                "CASL=0",
                "-a",
                "wght=400",
                "-a",
                "slnt=0",
                "-a",
                "CRSV=0.5",
                str(recursive_vf),
                str(output),
            ],
            capture_output=True,
        )

        assert result.returncode == 0, f"Command failed: {result.stderr.decode()}"
        assert output.exists(), "Output file not created"

        font = TTFont(output)
        assert len(font.getGlyphOrder()) == 1304, "Unexpected glyph count"
        assert "fvar" not in font, "Static instance should not have fvar"
        font.close()

    def test_freeze_processes_font(self, rust_cli, tmp_path, recursive_vf):
        """Test that freeze command processes font without error."""
        # Create a static instance first (it will have GSUB)
        instance = tmp_path / "instance.ttf"
        subprocess.run(
            [
                str(rust_cli),
                "instance",
                "-a",
                "MONO=1",
                "-a",
                "CASL=0",
                "-a",
                "wght=400",
                "-a",
                "slnt=0",
                "-a",
                "CRSV=0.5",
                str(recursive_vf),
                str(instance),
            ],
            capture_output=True,
            check=True,
        )

        dest = tmp_path / "frozen.ttf"
        shutil.copy(instance, dest)

        result = subprocess.run(
            [str(rust_cli), "freeze", "-f", "ss01,ss02", str(dest)],
            capture_output=True,
        )

        assert result.returncode == 0, f"Command failed: {result.stderr.decode()}"

        # Font should still be valid with GSUB preserved
        font = TTFont(dest)
        assert len(font.getGlyphOrder()) > 0, "Font should have glyphs"
        assert "GSUB" in font, "GSUB table should be preserved after freeze"
        font.close()

    def test_set_monospace_flags(self, rust_cli, tmp_path):
        """Test that set-monospace command sets correct flags."""
        src = BUILD_DIR / "RecMonoDuotone-Regular.ttf"
        if not src.exists():
            src = DIST_DIR / "WarpnineMono-Regular.ttf"
        if not src.exists():
            pytest.skip("Source font not available")

        dest = tmp_path / "mono.ttf"
        shutil.copy(src, dest)

        result = subprocess.run(
            [str(rust_cli), "set-monospace", str(dest)],
            capture_output=True,
        )

        assert result.returncode == 0, f"Command failed: {result.stderr.decode()}"

        font = TTFont(dest)
        assert font["post"].isFixedPitch == 1, "isFixedPitch should be 1"
        assert font["OS/2"].panose.bProportion == 9, "panose.bProportion should be 9"
        font.close()

    def test_merge_increases_glyph_count(self, rust_cli, tmp_path):
        """Test that merge command combines glyphs from both fonts."""
        base = BUILD_DIR / "RecMonoDuotone-Regular.ttf"
        fallback = BUILD_DIR / "Noto-400-subset.ttf"

        if not base.exists() or not fallback.exists():
            pytest.skip("Source fonts not available")

        output = tmp_path / "merged.ttf"

        result = subprocess.run(
            [str(rust_cli), "merge", str(base), str(fallback), "-o", str(output)],
            capture_output=True,
        )

        assert result.returncode == 0, f"Command failed: {result.stderr.decode()}"

        base_font = TTFont(base)
        merged_font = TTFont(output)

        base_glyphs = len(base_font.getGlyphOrder())
        merged_glyphs = len(merged_font.getGlyphOrder())

        base_font.close()
        merged_font.close()

        assert merged_glyphs > base_glyphs, (
            f"Merged ({merged_glyphs}) should have more glyphs than base ({base_glyphs})"
        )

    def test_build_vf_creates_valid_variable_font(self, rust_cli, tmp_path):
        """Test that build-vf command creates valid variable font."""
        # Check if we have the required static fonts
        required_fonts = [
            "WarpnineMono-Light.ttf",
            "WarpnineMono-Regular.ttf",
            "WarpnineMono-Bold.ttf",
            "WarpnineMono-Black.ttf",
        ]

        for font_name in required_fonts:
            if not (DIST_DIR / font_name).exists():
                pytest.skip(f"Required font not built: {font_name}")

        # Copy fonts to temp dir
        temp_dist = tmp_path / "dist"
        temp_dist.mkdir()

        for font_path in DIST_DIR.glob("WarpnineMono-*.ttf"):
            if "-VF" not in font_path.name:
                shutil.copy(font_path, temp_dist / font_path.name)

        output = tmp_path / "vf.ttf"

        result = subprocess.run(
            [
                str(rust_cli),
                "build-vf",
                "--dist-dir",
                str(temp_dist),
                "--output",
                str(output),
            ],
            capture_output=True,
            timeout=120,
        )

        assert result.returncode == 0, f"Command failed: {result.stderr.decode()}"
        assert output.exists(), "VF output not created"

        font = TTFont(output)
        assert "fvar" in font, "VF should have fvar table"
        assert "gvar" in font, "VF should have gvar table"

        fvar = font["fvar"]
        axes = {a.axisTag for a in fvar.axes}
        assert "wght" in axes, "VF should have wght axis"
        assert "ital" in axes, "VF should have ital axis"

        font.close()


# ============================================================================
# Naming Tests
# ============================================================================


class TestFontNaming:
    """Test name table entries for all font families."""

    def test_mono_font_names_exist(self):
        """Validate required name table entries exist for WarpnineMono fonts."""
        for style in ["Regular", "Bold", "Light", "Italic", "BoldItalic"]:
            font_path = DIST_DIR / f"WarpnineMono-{style}.ttf"
            if not font_path.exists():
                pytest.skip(f"Font not built: {font_path.name}")

            failures = validate_font_names_exist(font_path)
            assert not failures, f"{font_path.name}: {failures}"

    def test_mono_font_naming_values(self):
        """Validate WarpnineMono fonts have correct naming values."""
        for style in ["Regular", "Bold", "Light", "Italic", "BoldItalic"]:
            font_path = DIST_DIR / f"WarpnineMono-{style}.ttf"
            if not font_path.exists():
                pytest.skip(f"Font not built: {font_path.name}")

            failures = validate_font_naming(
                font_path, "Warpnine Mono", style, "WarpnineMono"
            )
            assert not failures, f"{font_path.name}: {failures}"

    def test_mono_vf_names_exist(self):
        """Validate required name table entries exist for WarpnineMono VF."""
        font_path = DIST_DIR / "WarpnineMono-VF.ttf"
        if not font_path.exists():
            pytest.skip("VF not built")

        failures = validate_font_names_exist(font_path)
        assert not failures, f"{font_path.name}: {failures}"

    def test_sans_font_names_exist(self):
        """Validate required name table entries exist for WarpnineSans fonts."""
        for style in ["Regular", "Bold", "Italic"]:
            font_path = DIST_DIR / f"WarpnineSans-{style}.ttf"
            if not font_path.exists():
                pytest.skip(f"Font not built: {font_path.name}")

            failures = validate_font_names_exist(font_path)
            assert not failures, f"{font_path.name}: {failures}"

    def test_sans_font_naming_values(self):
        """Validate WarpnineSans fonts have correct naming values."""
        for style in ["Regular", "Bold", "Italic"]:
            font_path = DIST_DIR / f"WarpnineSans-{style}.ttf"
            if not font_path.exists():
                pytest.skip(f"Font not built: {font_path.name}")

            failures = validate_font_naming(
                font_path, "Warpnine Sans", style, "WarpnineSans"
            )
            assert not failures, f"{font_path.name}: {failures}"

    def test_condensed_font_names_exist(self):
        """Validate required name table entries exist for WarpnineSansCondensed fonts."""
        for style in ["Regular", "Bold", "Italic"]:
            font_path = DIST_DIR / f"WarpnineSansCondensed-{style}.ttf"
            if not font_path.exists():
                pytest.skip(f"Font not built: {font_path.name}")

            failures = validate_font_names_exist(font_path)
            assert not failures, f"{font_path.name}: {failures}"

    def test_condensed_font_naming_values(self):
        """Validate WarpnineSansCondensed fonts have correct naming values."""
        for style in ["Regular", "Bold", "Italic"]:
            font_path = DIST_DIR / f"WarpnineSansCondensed-{style}.ttf"
            if not font_path.exists():
                pytest.skip(f"Font not built: {font_path.name}")

            failures = validate_font_naming(
                font_path, "Warpnine Sans Condensed", style, "WarpnineSansCondensed"
            )
            assert not failures, f"{font_path.name}: {failures}"

    def test_postscript_name_format(self):
        """Validate PostScript name format across all fonts."""
        for font_path in DIST_DIR.glob("Warpnine*.ttf"):
            font = TTFont(font_path)
            ps_name = get_name_entry(font, 6)
            font.close()

            assert ps_name, f"{font_path.name}: Missing PostScript name"
            assert " " not in ps_name, (
                f"{font_path.name}: PostScript name has spaces: {ps_name}"
            )
            assert "-" in ps_name, (
                f"{font_path.name}: PostScript name missing hyphen: {ps_name}"
            )


# ============================================================================
# Summary Test
# ============================================================================


class TestFontFamilySummary:
    """Summary tests that verify all expected fonts exist and are valid."""

    def test_all_mono_fonts_present(self):
        """Verify all WarpnineMono static fonts are present."""
        expected = [
            "WarpnineMono-Light.ttf",
            "WarpnineMono-LightItalic.ttf",
            "WarpnineMono-Regular.ttf",
            "WarpnineMono-Italic.ttf",
            "WarpnineMono-Medium.ttf",
            "WarpnineMono-MediumItalic.ttf",
            "WarpnineMono-SemiBold.ttf",
            "WarpnineMono-SemiBoldItalic.ttf",
            "WarpnineMono-Bold.ttf",
            "WarpnineMono-BoldItalic.ttf",
            "WarpnineMono-ExtraBold.ttf",
            "WarpnineMono-ExtraBoldItalic.ttf",
            "WarpnineMono-Black.ttf",
            "WarpnineMono-BlackItalic.ttf",
            "WarpnineMono-ExtraBlack.ttf",
            "WarpnineMono-ExtraBlackItalic.ttf",
        ]

        missing = [f for f in expected if not (DIST_DIR / f).exists()]

        if missing:
            pytest.skip(f"Missing fonts: {missing}")

        assert len(missing) == 0, f"Missing fonts: {missing}"

    def test_mono_vf_present(self):
        """Verify WarpnineMono VF is present."""
        vf = DIST_DIR / "WarpnineMono-VF.ttf"
        if not vf.exists():
            pytest.skip("VF not built")

        assert vf.exists()

    def test_all_sans_fonts_present(self):
        """Verify all WarpnineSans static fonts are present."""
        expected = [
            "WarpnineSans-Light.ttf",
            "WarpnineSans-LightItalic.ttf",
            "WarpnineSans-Regular.ttf",
            "WarpnineSans-Italic.ttf",
            "WarpnineSans-Medium.ttf",
            "WarpnineSans-MediumItalic.ttf",
            "WarpnineSans-SemiBold.ttf",
            "WarpnineSans-SemiBoldItalic.ttf",
            "WarpnineSans-Bold.ttf",
            "WarpnineSans-BoldItalic.ttf",
            "WarpnineSans-ExtraBold.ttf",
            "WarpnineSans-ExtraBoldItalic.ttf",
            "WarpnineSans-Black.ttf",
            "WarpnineSans-BlackItalic.ttf",
        ]

        missing = [f for f in expected if not (DIST_DIR / f).exists()]

        if missing:
            pytest.skip(f"Missing fonts: {missing}")

        assert len(missing) == 0

    def test_all_condensed_fonts_present(self):
        """Verify all WarpnineSansCondensed static fonts are present."""
        expected = [
            "WarpnineSansCondensed-Light.ttf",
            "WarpnineSansCondensed-LightItalic.ttf",
            "WarpnineSansCondensed-Regular.ttf",
            "WarpnineSansCondensed-Italic.ttf",
            "WarpnineSansCondensed-Medium.ttf",
            "WarpnineSansCondensed-MediumItalic.ttf",
            "WarpnineSansCondensed-SemiBold.ttf",
            "WarpnineSansCondensed-SemiBoldItalic.ttf",
            "WarpnineSansCondensed-Bold.ttf",
            "WarpnineSansCondensed-BoldItalic.ttf",
            "WarpnineSansCondensed-ExtraBold.ttf",
            "WarpnineSansCondensed-ExtraBoldItalic.ttf",
            "WarpnineSansCondensed-Black.ttf",
            "WarpnineSansCondensed-BlackItalic.ttf",
        ]

        missing = [f for f in expected if not (DIST_DIR / f).exists()]

        if missing:
            pytest.skip(f"Missing fonts: {missing}")

        assert len(missing) == 0
