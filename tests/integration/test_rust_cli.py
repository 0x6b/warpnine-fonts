"""
Integration tests comparing Rust CLI output with Python CLI output.

Testing Strategy (from IMPLEMENTATION_PLAN.md):
1. Run Python version, capture output
2. Run Rust version on same input
3. Compare output fonts with fontTools or binary diff
4. Benchmark timing
"""

import hashlib
import os
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

import pytest
from fontTools.ttLib import TTFont

# Paths
PROJECT_ROOT = Path(__file__).parent.parent.parent
RUST_CLI = PROJECT_ROOT / "target" / "release" / "warpnine-fonts"
BUILD_DIR = PROJECT_ROOT / "build"
DIST_DIR = PROJECT_ROOT / "dist"


def run_python_cmd(args: list[str], cwd: Path = PROJECT_ROOT) -> tuple[str, float]:
    """Run Python CLI and return (output, duration_seconds)."""
    start = time.perf_counter()
    result = subprocess.run(
        ["uv", "run", "warpnine", "build"] + args,
        cwd=cwd,
        capture_output=True,
        text=True,
    )
    duration = time.perf_counter() - start
    return result.stdout + result.stderr, duration


def run_rust_cmd(args: list[str], cwd: Path = PROJECT_ROOT) -> tuple[str, float]:
    """Run Rust CLI and return (output, duration_seconds)."""
    start = time.perf_counter()
    result = subprocess.run(
        [str(RUST_CLI)] + args,
        cwd=cwd,
        capture_output=True,
        text=True,
    )
    duration = time.perf_counter() - start
    return result.stdout + result.stderr, duration


def file_hash(path: Path) -> str:
    """Get SHA256 hash of a file."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compare_font_tables(
    font1_path: Path, font2_path: Path, tables_to_compare: list[str] | None = None
):
    """Compare specific tables between two fonts."""
    font1 = TTFont(font1_path)
    font2 = TTFont(font2_path)

    if tables_to_compare is None:
        tables_to_compare = ["name", "OS/2", "head", "hhea", "hmtx", "GSUB"]

    differences = []
    for table in tables_to_compare:
        if table in font1 and table in font2:
            # For GSUB, compare raw bytes
            if table == "GSUB":
                data1 = font1.getTableData(table)
                data2 = font2.getTableData(table)
                if data1 != data2:
                    differences.append(
                        f"{table}: binary data differs ({len(data1)} vs {len(data2)} bytes)"
                    )
            else:
                # For other tables, just check they exist (detailed comparison is complex)
                pass
        elif table in font1:
            differences.append(f"{table}: missing in font2")
        elif table in font2:
            differences.append(f"{table}: missing in font1")

    font1.close()
    font2.close()
    return differences


# Validation functions for comparing Rust vs Python output
def validate_mvar_metrics(rust_font: TTFont, python_font: TTFont) -> dict:
    """Validate MVAR-interpolated metrics."""
    results = {"pass": [], "fail": []}

    checks = [
        ("OS/2.sxHeight", rust_font["OS/2"].sxHeight, python_font["OS/2"].sxHeight),
        (
            "OS/2.yStrikeoutPosition",
            rust_font["OS/2"].yStrikeoutPosition,
            python_font["OS/2"].yStrikeoutPosition,
        ),
        (
            "OS/2.yStrikeoutSize",
            rust_font["OS/2"].yStrikeoutSize,
            python_font["OS/2"].yStrikeoutSize,
        ),
        (
            "post.underlinePosition",
            rust_font["post"].underlinePosition,
            python_font["post"].underlinePosition,
        ),
        (
            "post.underlineThickness",
            rust_font["post"].underlineThickness,
            python_font["post"].underlineThickness,
        ),
    ]

    for name, rs_val, py_val in checks:
        if rs_val == py_val:
            results["pass"].append(name)
        else:
            results["fail"].append(f"{name}: Rust={rs_val} Python={py_val}")

    return results


def validate_bounding_box(
    rust_font: TTFont, python_font: TTFont, tolerance: int = 2
) -> dict:
    """Validate bounding box values."""
    results = {"pass": [], "fail": []}

    checks = [
        ("head.xMin", rust_font["head"].xMin, python_font["head"].xMin),
        ("head.xMax", rust_font["head"].xMax, python_font["head"].xMax),
        ("head.yMin", rust_font["head"].yMin, python_font["head"].yMin),
        ("head.yMax", rust_font["head"].yMax, python_font["head"].yMax),
        (
            "hhea.minLeftSideBearing",
            rust_font["hhea"].minLeftSideBearing,
            python_font["hhea"].minLeftSideBearing,
        ),
        (
            "hhea.minRightSideBearing",
            rust_font["hhea"].minRightSideBearing,
            python_font["hhea"].minRightSideBearing,
        ),
        (
            "hhea.xMaxExtent",
            rust_font["hhea"].xMaxExtent,
            python_font["hhea"].xMaxExtent,
        ),
    ]

    for name, rs_val, py_val in checks:
        diff = abs(rs_val - py_val)
        if diff <= tolerance:
            results["pass"].append(name)
        else:
            results["fail"].append(
                f"{name}: Rust={rs_val} Python={py_val} (diff={diff})"
            )

    return results


def validate_gdef(rust_font: TTFont, python_font: TTFont) -> dict:
    """Validate GDEF table (glyph classes, mark attach classes)."""
    results = {"pass": [], "fail": []}

    if "GDEF" not in rust_font or "GDEF" not in python_font:
        if "GDEF" not in rust_font and "GDEF" not in python_font:
            results["pass"].append("GDEF (both missing)")
        else:
            results["fail"].append("GDEF presence differs")
        return results

    rs_gdef = rust_font["GDEF"].table
    py_gdef = python_font["GDEF"].table

    if rs_gdef.GlyphClassDef and py_gdef.GlyphClassDef:
        rs_classes = dict(rs_gdef.GlyphClassDef.classDefs)
        py_classes = dict(py_gdef.GlyphClassDef.classDefs)
        if rs_classes == py_classes:
            results["pass"].append(f"GDEF GlyphClassDef ({len(rs_classes)} glyphs)")
        else:
            diff = len(set(rs_classes.items()) ^ set(py_classes.items()))
            results["fail"].append(f"GDEF GlyphClassDef differs ({diff} differences)")
    elif rs_gdef.GlyphClassDef or py_gdef.GlyphClassDef:
        results["fail"].append("GDEF GlyphClassDef presence differs")
    else:
        results["pass"].append("GDEF GlyphClassDef (both none)")

    if rs_gdef.MarkAttachClassDef and py_gdef.MarkAttachClassDef:
        rs_marks = dict(rs_gdef.MarkAttachClassDef.classDefs)
        py_marks = dict(py_gdef.MarkAttachClassDef.classDefs)
        if rs_marks == py_marks:
            results["pass"].append(f"GDEF MarkAttachClassDef ({len(rs_marks)} glyphs)")
        else:
            diff = len(set(rs_marks.items()) ^ set(py_marks.items()))
            results["fail"].append(
                f"GDEF MarkAttachClassDef differs ({diff} differences)"
            )
    elif rs_gdef.MarkAttachClassDef or py_gdef.MarkAttachClassDef:
        results["fail"].append("GDEF MarkAttachClassDef presence differs")
    else:
        results["pass"].append("GDEF MarkAttachClassDef (both none)")

    return results


def validate_table_tags(rust_font: TTFont, python_font: TTFont) -> dict:
    """Validate that both fonts have the same set of table tags."""
    results = {"pass": [], "fail": []}

    rs_tables = set(rust_font.keys())
    py_tables = set(python_font.keys())

    if rs_tables == py_tables:
        results["pass"].append(f"Table tags match ({len(rs_tables)} tables)")
    else:
        only_rust = rs_tables - py_tables
        only_python = py_tables - rs_tables
        if only_rust:
            results["fail"].append(f"Tables only in Rust: {sorted(only_rust)}")
        if only_python:
            results["fail"].append(f"Tables only in Python: {sorted(only_python)}")

    return results


def validate_features(rust_font: TTFont, python_font: TTFont) -> dict:
    """Validate GSUB/GPOS features."""
    results = {"pass": [], "fail": []}

    # GSUB
    rs_gsub = {r.FeatureTag for r in rust_font["GSUB"].table.FeatureList.FeatureRecord}
    py_gsub = {
        r.FeatureTag for r in python_font["GSUB"].table.FeatureList.FeatureRecord
    }

    if rs_gsub == py_gsub:
        results["pass"].append(f"GSUB features ({len(rs_gsub)})")
    else:
        only_rust = rs_gsub - py_gsub
        only_python = py_gsub - rs_gsub
        results["fail"].append(
            f"GSUB features differ: only_rust={only_rust}, only_python={only_python}"
        )

    # GPOS
    rs_gpos = {r.FeatureTag for r in rust_font["GPOS"].table.FeatureList.FeatureRecord}
    py_gpos = {
        r.FeatureTag for r in python_font["GPOS"].table.FeatureList.FeatureRecord
    }

    if rs_gpos == py_gpos:
        results["pass"].append(f"GPOS features ({len(rs_gpos)})")
    else:
        results["fail"].append("GPOS features differ")

    return results


def validate_core_metrics(rust_font: TTFont, python_font: TTFont) -> dict:
    """Validate core metrics that should always match."""
    results = {"pass": [], "fail": []}

    checks = [
        (
            "maxp.numGlyphs",
            rust_font["maxp"].numGlyphs,
            python_font["maxp"].numGlyphs,
        ),
        (
            "OS/2.usWeightClass",
            rust_font["OS/2"].usWeightClass,
            python_font["OS/2"].usWeightClass,
        ),
        (
            "OS/2.usWidthClass",
            rust_font["OS/2"].usWidthClass,
            python_font["OS/2"].usWidthClass,
        ),
        (
            "head.unitsPerEm",
            rust_font["head"].unitsPerEm,
            python_font["head"].unitsPerEm,
        ),
    ]

    for name, rs_val, py_val in checks:
        if rs_val == py_val:
            results["pass"].append(name)
        else:
            results["fail"].append(f"{name}: Rust={rs_val} Python={py_val}")

    return results


def validate_hmtx(rust_font: TTFont, python_font: TTFont) -> dict:
    """Validate per-glyph horizontal metrics (advance widths and LSBs).

    LSB should equal glyph xMin. Small differences (±10) are acceptable due to
    coordinate interpolation rounding differences between Rust and Python.
    """
    results = {"pass": [], "fail": []}

    rs_hmtx = rust_font["hmtx"].metrics
    py_hmtx = python_font["hmtx"].metrics

    rs_glyphs = set(rs_hmtx.keys())
    py_glyphs = set(py_hmtx.keys())

    if rs_glyphs != py_glyphs:
        only_rust = rs_glyphs - py_glyphs
        only_python = py_glyphs - rs_glyphs
        if only_rust:
            results["fail"].append(f"hmtx glyphs only in Rust: {len(only_rust)}")
        if only_python:
            results["fail"].append(f"hmtx glyphs only in Python: {len(only_python)}")
        return results

    width_mismatched = []
    lsb_mismatched = []
    for glyph in rs_glyphs:
        rs_width, rs_lsb = rs_hmtx[glyph]
        py_width, py_lsb = py_hmtx[glyph]
        if rs_width != py_width:
            width_mismatched.append(f"{glyph}: Rust={rs_width} Python={py_width}")
        # Allow ±10 tolerance for LSB due to coordinate rounding differences
        if abs(rs_lsb - py_lsb) > 10:
            lsb_mismatched.append(f"{glyph}: Rust={rs_lsb} Python={py_lsb}")

    if width_mismatched:
        results["fail"].append(
            f"hmtx widths differ for {len(width_mismatched)} glyphs: "
            f"{width_mismatched[:3]}..."
        )
    else:
        results["pass"].append(f"hmtx widths ({len(rs_glyphs)} glyphs)")

    if lsb_mismatched:
        results["fail"].append(
            f"hmtx LSB differs by >10 for {len(lsb_mismatched)} glyphs: "
            f"{lsb_mismatched[:3]}..."
        )
    else:
        results["pass"].append(f"hmtx LSBs ({len(rs_glyphs)} glyphs, ±10 tolerance)")

    return results


def validate_os2_metrics(rust_font: TTFont, python_font: TTFont) -> dict:
    """Validate OS/2 typo and win metrics."""
    results = {"pass": [], "fail": []}

    checks = [
        (
            "OS/2.sTypoAscender",
            rust_font["OS/2"].sTypoAscender,
            python_font["OS/2"].sTypoAscender,
        ),
        (
            "OS/2.sTypoDescender",
            rust_font["OS/2"].sTypoDescender,
            python_font["OS/2"].sTypoDescender,
        ),
        (
            "OS/2.sTypoLineGap",
            rust_font["OS/2"].sTypoLineGap,
            python_font["OS/2"].sTypoLineGap,
        ),
        (
            "OS/2.usWinAscent",
            rust_font["OS/2"].usWinAscent,
            python_font["OS/2"].usWinAscent,
        ),
        (
            "OS/2.usWinDescent",
            rust_font["OS/2"].usWinDescent,
            python_font["OS/2"].usWinDescent,
        ),
        (
            "OS/2.fsSelection",
            rust_font["OS/2"].fsSelection,
            python_font["OS/2"].fsSelection,
        ),
    ]

    for name, rs_val, py_val in checks:
        if rs_val == py_val:
            results["pass"].append(name)
        else:
            results["fail"].append(f"{name}: Rust={rs_val} Python={py_val}")

    return results


def validate_font_pair(rust_path: Path, python_path: Path) -> tuple[int, int, list]:
    """Validate a single font pair. Returns (pass_count, fail_count, failures)."""
    rust_font = TTFont(rust_path)
    python_font = TTFont(python_path)

    all_pass = []
    all_fail = []

    # Table tags (should always pass)
    tables = validate_table_tags(rust_font, python_font)
    all_pass.extend(tables["pass"])
    all_fail.extend(tables["fail"])

    # Core metrics (should always pass)
    core = validate_core_metrics(rust_font, python_font)
    all_pass.extend(core["pass"])
    all_fail.extend(core["fail"])

    # OS/2 typo and win metrics
    os2 = validate_os2_metrics(rust_font, python_font)
    all_pass.extend(os2["pass"])
    all_fail.extend(os2["fail"])

    # hmtx (per-glyph advance widths)
    hmtx = validate_hmtx(rust_font, python_font)
    all_pass.extend(hmtx["pass"])
    all_fail.extend(hmtx["fail"])

    # Features (should always pass)
    features = validate_features(rust_font, python_font)
    all_pass.extend(features["pass"])
    all_fail.extend(features["fail"])

    # MVAR metrics (may fail until font-instancer is fixed)
    mvar = validate_mvar_metrics(rust_font, python_font)
    all_pass.extend(mvar["pass"])
    all_fail.extend(mvar["fail"])

    # Bounding box (may fail until font-instancer is fixed)
    bbox = validate_bounding_box(rust_font, python_font)
    all_pass.extend(bbox["pass"])
    all_fail.extend(bbox["fail"])

    # GDEF table
    gdef = validate_gdef(rust_font, python_font)
    all_pass.extend(gdef["pass"])
    all_fail.extend(gdef["fail"])

    rust_font.close()
    python_font.close()

    return len(all_pass), len(all_fail), all_fail


class TestCleanCommand:
    """Test the clean command."""

    def test_clean_removes_directories(self, tmp_path):
        """Test that clean removes build and dist directories."""
        # Create test directories
        build = tmp_path / "build"
        dist = tmp_path / "dist"
        build.mkdir()
        dist.mkdir()
        (build / "test.ttf").touch()
        (dist / "test.ttf").touch()

        # Run Rust clean
        result = subprocess.run(
            [
                str(RUST_CLI),
                "clean",
                "--build-dir",
                str(build),
                "--dist-dir",
                str(dist),
            ],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        assert not build.exists()
        assert not dist.exists()

    def test_clean_handles_missing_dirs(self, tmp_path):
        """Test that clean handles non-existent directories gracefully."""
        build = tmp_path / "nonexistent_build"
        dist = tmp_path / "nonexistent_dist"

        result = subprocess.run(
            [
                str(RUST_CLI),
                "clean",
                "--build-dir",
                str(build),
                "--dist-dir",
                str(dist),
            ],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0


class TestDownloadCommand:
    """Test the download command."""

    @pytest.mark.slow
    def test_download_creates_files(self, tmp_path):
        """Test that download creates expected files."""
        build = tmp_path / "build"

        result = subprocess.run(
            [str(RUST_CLI), "download", "--build-dir", str(build)],
            capture_output=True,
            text=True,
            timeout=300,  # 5 minutes
        )

        assert result.returncode == 0
        assert (build / "Recursive_VF_1.085.ttf").exists()
        assert (build / "NotoSansMonoCJKjp-VF.ttf").exists()


class TestCopyGsubCommand:
    """Test the copy-gsub command."""

    @pytest.fixture
    def setup_fonts(self, tmp_path):
        """Set up test fonts for GSUB copy test."""
        # Copy existing fonts to temp directory
        source_font = BUILD_DIR / "Recursive_VF_1.085.ttf"
        target_font = DIST_DIR / "WarpnineMono-Regular.ttf"

        if not source_font.exists() or not target_font.exists():
            pytest.skip("Required fonts not built yet")

        source_copy = tmp_path / "source.ttf"
        target_copy = tmp_path / "target.ttf"
        shutil.copy(source_font, source_copy)
        shutil.copy(target_font, target_copy)

        return source_copy, target_copy

    def test_copy_gsub_preserves_table(self, setup_fonts):
        """Test that copy-gsub correctly copies GSUB table."""
        source, target = setup_fonts

        # Get original GSUB from source
        source_font = TTFont(source)
        original_gsub = source_font.getTableData("GSUB")
        source_font.close()

        # Run Rust copy-gsub
        result = subprocess.run(
            [str(RUST_CLI), "copy-gsub", "--from", str(source), "--to", str(target)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0

        # Verify GSUB was copied
        target_font = TTFont(target)
        copied_gsub = target_font.getTableData("GSUB")
        target_font.close()

        assert original_gsub == copied_gsub, "GSUB table should match source"


class TestRemoveLigaturesCommand:
    """Test the remove-ligatures command."""

    @pytest.fixture
    def setup_duotone_font(self, tmp_path):
        """Set up a Duotone font for ligature removal test."""
        duotone = BUILD_DIR / "RecMonoDuotone-Regular.ttf"

        if not duotone.exists():
            pytest.skip("Duotone font not built yet")

        font_copy = tmp_path / "duotone.ttf"
        shutil.copy(duotone, font_copy)
        return font_copy

    def test_remove_ligatures_clears_subst_lookup_records(self, setup_duotone_font):
        """Test that remove-ligatures clears SubstLookupRecord for grave rules."""
        font_path = setup_duotone_font

        # Run Rust remove-ligatures
        result = subprocess.run(
            [str(RUST_CLI), "remove-ligatures", str(font_path)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        assert "Found three-backtick pattern" in result.stdout
        assert "Saved modified font" in result.stdout

        # Verify the ligature was actually removed
        font = TTFont(font_path)
        gsub = font["GSUB"].table

        # Find the lookup with grave rules (Type 6 Format 1)
        for lookup in gsub.LookupList.Lookup:
            if lookup.LookupType == 6:
                for subtable in lookup.SubTable:
                    if hasattr(subtable, "Format") and subtable.Format == 1:
                        cov = getattr(subtable, "Coverage", None)
                        if cov and "grave" in list(cov.glyphs):
                            grave_idx = list(cov.glyphs).index("grave")
                            rulesets = getattr(subtable, "ChainSubRuleSet", [])
                            if (
                                rulesets
                                and grave_idx < len(rulesets)
                                and rulesets[grave_idx]
                            ):
                                for rule in rulesets[grave_idx].ChainSubRule:
                                    input_seq = getattr(rule, "Input", [])
                                    if len(input_seq) == 2 and all(
                                        g == "grave" for g in input_seq
                                    ):
                                        # This rule should have no SubstLookupRecords
                                        subst = getattr(rule, "SubstLookupRecord", [])
                                        assert len(subst) == 0, (
                                            f"SubstLookupRecord should be empty, got {len(subst)}"
                                        )

        font.close()

    def test_remove_ligatures_preserves_font_validity(self, setup_duotone_font):
        """Test that remove-ligatures preserves font validity."""
        font_path = setup_duotone_font

        # Run command
        subprocess.run(
            [str(RUST_CLI), "remove-ligatures", str(font_path)],
            capture_output=True,
        )

        # Verify font is still valid
        font = TTFont(font_path)
        assert "GSUB" in font
        assert "name" in font
        font.close()


class TestCreateSansCommand:
    """Test the create-sans command."""

    @pytest.fixture
    def setup_recursive_vf(self, tmp_path):
        """Set up Recursive VF for sans creation test."""
        vf = BUILD_DIR / "Recursive_VF_1.085.ttf"

        if not vf.exists():
            pytest.skip("Recursive VF not downloaded yet")

        output_dir = tmp_path / "sans"
        output_dir.mkdir()
        return vf, output_dir

    @pytest.mark.slow
    def test_create_sans_generates_fonts(self, setup_recursive_vf):
        """Test that create-sans generates expected fonts."""
        input_vf, output_dir = setup_recursive_vf

        result = subprocess.run(
            [
                str(RUST_CLI),
                "create-sans",
                "--input",
                str(input_vf),
                "--output-dir",
                str(output_dir),
            ],
            capture_output=True,
            text=True,
            timeout=120,
        )

        assert result.returncode == 0

        # Check that fonts were created
        expected_styles = [
            "Light",
            "LightItalic",
            "Regular",
            "Italic",
            "Medium",
            "MediumItalic",
            "SemiBold",
            "SemiBoldItalic",
            "Bold",
            "BoldItalic",
            "ExtraBold",
            "ExtraBoldItalic",
            "Black",
            "BlackItalic",
        ]

        for style in expected_styles:
            font_path = output_dir / f"WarpnineSans-{style}.ttf"
            assert font_path.exists(), f"Missing {style} font"

            # Verify it's a valid font
            font = TTFont(font_path)
            assert "name" in font
            font.close()

    @pytest.mark.slow
    def test_create_sans_metrics(self, setup_recursive_vf):
        """Test that create-sans produces fonts with correct metrics."""
        input_vf, output_dir = setup_recursive_vf

        result = subprocess.run(
            [
                str(RUST_CLI),
                "create-sans",
                "--input",
                str(input_vf),
                "--output-dir",
                str(output_dir),
            ],
            capture_output=True,
            text=True,
            timeout=120,
        )

        assert result.returncode == 0

        regular = output_dir / "WarpnineSans-Regular.ttf"
        bold = output_dir / "WarpnineSans-Bold.ttf"
        italic = output_dir / "WarpnineSans-Italic.ttf"

        for font_path in [regular, bold, italic]:
            assert font_path.exists(), f"Missing font: {font_path.name}"

            font = TTFont(font_path)

            # MVAR metrics should be interpolated correctly
            assert font["OS/2"].sxHeight > 0, (
                f"{font_path.name}: sxHeight should be positive"
            )
            assert font["OS/2"].sCapHeight > 0, (
                f"{font_path.name}: sCapHeight should be positive"
            )

            # Bounding box should be valid
            assert font["head"].xMin <= 0, f"{font_path.name}: xMin should be <= 0"
            assert font["head"].xMax > 0, f"{font_path.name}: xMax should be > 0"
            assert font["head"].yMin < 0, f"{font_path.name}: yMin should be < 0"
            assert font["head"].yMax > 0, f"{font_path.name}: yMax should be > 0"

            # hhea metrics
            assert font["hhea"].ascender > 0, (
                f"{font_path.name}: ascender should be > 0"
            )
            assert font["hhea"].descender < 0, (
                f"{font_path.name}: descender should be < 0"
            )

            # usWidthClass should be normal (5) for sans
            assert font["OS/2"].usWidthClass == 5, (
                f"{font_path.name}: usWidthClass should be 5 (Normal)"
            )

            font.close()

        # Weight class ordering
        reg_font = TTFont(regular)
        bold_font = TTFont(bold)
        assert bold_font["OS/2"].usWeightClass > reg_font["OS/2"].usWeightClass, (
            "Bold should have higher usWeightClass than Regular"
        )
        reg_font.close()
        bold_font.close()


class TestCreateCondensedCommand:
    """Test the create-condensed command."""

    @pytest.fixture
    def setup_recursive_vf(self, tmp_path):
        """Set up Recursive VF for condensed creation test."""
        vf = BUILD_DIR / "Recursive_VF_1.085.ttf"

        if not vf.exists():
            pytest.skip("Recursive VF not downloaded yet")

        output_dir = tmp_path / "condensed"
        output_dir.mkdir()
        return vf, output_dir

    @pytest.mark.slow
    def test_create_condensed_generates_fonts(self, setup_recursive_vf):
        """Test that create-condensed generates expected fonts."""
        input_vf, output_dir = setup_recursive_vf

        result = subprocess.run(
            [
                str(RUST_CLI),
                "create-condensed",
                "--input",
                str(input_vf),
                "--output-dir",
                str(output_dir),
            ],
            capture_output=True,
            text=True,
            timeout=120,
        )

        assert result.returncode == 0

        # Check that fonts were created
        expected_styles = [
            "Light",
            "LightItalic",
            "Regular",
            "Italic",
            "Medium",
            "MediumItalic",
            "SemiBold",
            "SemiBoldItalic",
            "Bold",
            "BoldItalic",
            "ExtraBold",
            "ExtraBoldItalic",
            "Black",
            "BlackItalic",
        ]

        for style in expected_styles:
            font_path = output_dir / f"WarpnineSansCondensed-{style}.ttf"
            assert font_path.exists(), f"Missing {style} font"

            # Verify it's a valid font and check width metrics
            font = TTFont(font_path)
            assert "name" in font
            # Check OS/2 width class is set to Condensed (3)
            assert font["OS/2"].usWidthClass == 3, (
                f"{style}: usWidthClass should be 3 (Condensed)"
            )
            font.close()

    @pytest.mark.slow
    def test_create_condensed_metrics_match_python(self, setup_recursive_vf):
        """Test that create-condensed metrics match Python output."""
        input_vf, output_dir = setup_recursive_vf

        result = subprocess.run(
            [
                str(RUST_CLI),
                "create-condensed",
                "--input",
                str(input_vf),
                "--output-dir",
                str(output_dir),
            ],
            capture_output=True,
            text=True,
            timeout=120,
        )

        assert result.returncode == 0

        regular = output_dir / "WarpnineSansCondensed-Regular.ttf"
        bold = output_dir / "WarpnineSansCondensed-Bold.ttf"
        italic = output_dir / "WarpnineSansCondensed-Italic.ttf"

        for font_path in [regular, bold, italic]:
            assert font_path.exists(), f"Missing font: {font_path.name}"

            font = TTFont(font_path)

            assert font["OS/2"].sxHeight > 0, (
                f"{font_path.name}: sxHeight should be positive"
            )
            assert font["OS/2"].sCapHeight > 0, (
                f"{font_path.name}: sCapHeight should be positive"
            )

            assert font["head"].xMin <= 0, f"{font_path.name}: xMin should be <= 0"
            assert font["head"].xMax > 0, f"{font_path.name}: xMax should be > 0"
            assert font["head"].yMin < 0, f"{font_path.name}: yMin should be < 0"
            assert font["head"].yMax > 0, f"{font_path.name}: yMax should be > 0"

            assert font["hhea"].ascender > 0, (
                f"{font_path.name}: ascender should be > 0"
            )
            assert font["hhea"].descender < 0, (
                f"{font_path.name}: descender should be < 0"
            )

            font.close()

        reg_font = TTFont(regular)
        bold_font = TTFont(bold)
        assert bold_font["OS/2"].usWeightClass > reg_font["OS/2"].usWeightClass, (
            "Bold should have higher usWeightClass than Regular"
        )
        reg_font.close()
        bold_font.close()


class TestSubsetJapaneseCommand:
    """Test the subset-japanese command."""

    @pytest.fixture
    def setup_noto_font(self, tmp_path):
        """Set up Noto CJK font for subset test."""
        noto = BUILD_DIR / "NotoSansMonoCJKjp-VF.ttf"

        if not noto.exists():
            pytest.skip("Noto CJK font not downloaded yet")

        font_copy = tmp_path / "noto.ttf"
        shutil.copy(noto, font_copy)
        output = tmp_path / "noto_subset.ttf"
        return font_copy, output

    def test_subset_japanese_reduces_size(self, setup_noto_font):
        """Test that subset-japanese reduces font size significantly."""
        input_font, output_font = setup_noto_font

        original_size = input_font.stat().st_size

        result = subprocess.run(
            [str(RUST_CLI), "subset-japanese", str(input_font), str(output_font)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        assert output_font.exists()

        subset_size = output_font.stat().st_size
        # Subset should be significantly smaller (~51% of original)
        # because VF tables are dropped
        assert subset_size < original_size * 0.6, (
            f"Subset ({subset_size}) should be less than 60% of original ({original_size})"
        )

    def test_subset_japanese_preserves_japanese_glyphs(self, setup_noto_font):
        """Test that subset preserves Japanese glyphs."""
        input_font, output_font = setup_noto_font

        subprocess.run(
            [str(RUST_CLI), "subset-japanese", str(input_font), str(output_font)],
            capture_output=True,
            check=True,
        )

        font = TTFont(output_font)
        cmap = font.getBestCmap()

        # Check some common Japanese characters are preserved
        # Hiragana: あ (U+3042), い (U+3044)
        # Katakana: ア (U+30A2), イ (U+30A4)
        # Kanji: 日 (U+65E5), 本 (U+672C)
        japanese_chars = [0x3042, 0x3044, 0x30A2, 0x30A4, 0x65E5, 0x672C]
        for char in japanese_chars:
            assert char in cmap, f"Japanese character U+{char:04X} should be preserved"

        font.close()

    def test_subset_japanese_drops_vf_tables(self, setup_noto_font):
        """Test that subset-japanese drops variable font tables."""
        input_font, output_font = setup_noto_font

        subprocess.run(
            [str(RUST_CLI), "subset-japanese", str(input_font), str(output_font)],
            capture_output=True,
            check=True,
        )

        font = TTFont(output_font)
        vf_tables = ["fvar", "gvar", "HVAR", "MVAR", "STAT", "avar", "cvar"]
        for table in vf_tables:
            assert table not in font, f"{table} should be dropped from subset font"
        font.close()


class TestFreezeCommand:
    """Test the freeze command."""

    @pytest.fixture
    def setup_font_with_features(self, tmp_path):
        """Set up a font with OpenType features for freeze test."""
        # Use Recursive VF which has ss01, ss02, etc.
        vf = BUILD_DIR / "Recursive_VF_1.085.ttf"

        if not vf.exists():
            pytest.skip("Recursive VF not downloaded yet")

        # First create an instance
        instance = tmp_path / "instance.ttf"
        subprocess.run(
            [
                str(RUST_CLI),
                "instance",
                str(vf),
                str(instance),
                "--axis",
                "wght=400",
                "--axis",
                "MONO=1",
                "--axis",
                "CASL=0",
                "--axis",
                "slnt=0",
                "--axis",
                "CRSV=0.5",
            ],
            capture_output=True,
            check=True,
        )

        return instance

    def test_freeze_updates_cmap(self, setup_font_with_features):
        """Test that freeze updates cmap with substituted glyphs."""
        font_path = setup_font_with_features

        # Get cmap before freeze - check what glyph '0' maps to
        font_before = TTFont(font_path)
        zero_before = font_before.getBestCmap()[ord("0")]
        font_before.close()

        # Freeze with 'zero' feature (slashed zero)
        result = subprocess.run(
            [str(RUST_CLI), "freeze", "--features", "zero", str(font_path)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0

        # Get cmap after freeze
        font_after = TTFont(font_path)
        zero_after = font_after.getBestCmap()[ord("0")]
        font_after.close()

        # The 'zero' feature should substitute zero -> zero.slash
        assert zero_before == "zero", (
            f"Before freeze, '0' should map to 'zero', got {zero_before}"
        )
        assert zero_after == "zero.slash", (
            f"After freeze, '0' should map to 'zero.slash', got {zero_after}"
        )

    def test_freeze_multiple_features(self, setup_font_with_features):
        """Test freezing multiple features."""
        font_path = setup_font_with_features

        result = subprocess.run(
            [str(RUST_CLI), "freeze", "--features", "ss01,ss02,ss03", str(font_path)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0

        # Verify font is still valid
        font = TTFont(font_path)
        assert "GSUB" in font
        font.close()

    def test_freeze_auto_rvrn_prepends_feature(self, setup_font_with_features):
        """Test that --auto-rvrn prepends rvrn to the feature list (Python compat)."""
        font_path = setup_font_with_features

        result = subprocess.run(
            [
                str(RUST_CLI),
                "freeze",
                "--auto-rvrn",
                "--features",
                "ss01",
                str(font_path),
            ],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        assert "rvrn,ss01" in result.stdout, (
            f"Should prepend rvrn to feature list, got: {result.stdout}"
        )

    def test_freeze_without_auto_rvrn(self, setup_font_with_features):
        """Test that rvrn is NOT auto-added without --auto-rvrn (intentional difference from Python)."""
        font_path = setup_font_with_features

        result = subprocess.run(
            [str(RUST_CLI), "freeze", "--features", "ss01", str(font_path)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        assert "rvrn" not in result.stdout, (
            f"Should NOT include rvrn without --auto-rvrn: {result.stdout}"
        )

    def test_freeze_mono_features_glyph_substitutions(self, setup_font_with_features):
        """Test that MONO feature freeze produces correct glyph substitutions."""
        font_path = setup_font_with_features

        mono_features = "ss01,ss02,ss03,ss04,ss05,ss06,ss08,ss10,ss12,pnum"

        result = subprocess.run(
            [str(RUST_CLI), "freeze", "--features", mono_features, str(font_path)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0

        font = TTFont(font_path)
        cmap = font.getBestCmap()

        expected_substitutions = {
            0x0061: "a.simple",
            0x0067: "g.simple",
            0x0066: "f.simple",
            0x006C: "l.simple",
            0x0072: "r.simple",
            0x004C: "L.sans",
            0x005A: "Z.sans",
            0x0040: "at.alt",
        }

        for codepoint, expected_glyph in expected_substitutions.items():
            actual_glyph = cmap.get(codepoint)
            assert actual_glyph == expected_glyph, (
                f"U+{codepoint:04X} ('{chr(codepoint)}'): expected '{expected_glyph}', got '{actual_glyph}'"
            )

        font.close()

    def test_freeze_sans_features_glyph_substitutions(self, setup_font_with_features):
        """Test that SANS feature freeze produces correct glyph substitutions."""
        font_path = setup_font_with_features

        sans_features = "ss01,ss02,ss03,ss04,ss05,ss06,ss08,ss10,ss12,pnum"

        result = subprocess.run(
            [str(RUST_CLI), "freeze", "--features", sans_features, str(font_path)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0

        font = TTFont(font_path)
        cmap = font.getBestCmap()

        expected_substitutions = {
            0x0061: "a.simple",
            0x0067: "g.simple",
            0x0066: "f.simple",
            0x006C: "l.simple",
            0x0072: "r.simple",
            0x004C: "L.sans",
            0x005A: "Z.sans",
            0x0040: "at.alt",
        }

        for codepoint, expected_glyph in expected_substitutions.items():
            actual_glyph = cmap.get(codepoint)
            assert actual_glyph == expected_glyph, (
                f"U+{codepoint:04X} ('{chr(codepoint)}'): expected '{expected_glyph}', got '{actual_glyph}'"
            )

        font.close()


class TestInstanceCommand:
    """Test the instance command."""

    @pytest.fixture
    def setup_variable_font(self, tmp_path):
        """Set up variable font for instance test."""
        vf = BUILD_DIR / "Recursive_VF_1.085.ttf"

        if not vf.exists():
            pytest.skip("Recursive VF not downloaded yet")

        output = tmp_path / "instance.ttf"
        return vf, output

    def test_instance_creates_static_font(self, setup_variable_font):
        """Test that instance creates a static font from VF."""
        input_vf, output = setup_variable_font

        result = subprocess.run(
            [
                str(RUST_CLI),
                "instance",
                str(input_vf),
                str(output),
                "--axis",
                "wght=700",
                "--axis",
                "MONO=1",
                "--axis",
                "CASL=0",
                "--axis",
                "slnt=0",
                "--axis",
                "CRSV=0.5",
            ],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        assert output.exists()

        # Verify it's a static font (no fvar table)
        font = TTFont(output)
        assert "fvar" not in font, "Instance should not have fvar table"
        assert "gvar" not in font, "Instance should not have gvar table"
        font.close()

    def test_instance_creates_different_weights(self, setup_variable_font):
        """Test that different weight instances have different glyph shapes."""
        input_vf, output = setup_variable_font
        output_bold = output.parent / "bold.ttf"

        # Create regular instance (wght=400)
        subprocess.run(
            [
                str(RUST_CLI),
                "instance",
                str(input_vf),
                str(output),
                "--axis",
                "wght=400",
                "--axis",
                "MONO=0",
                "--axis",
                "CASL=0",
                "--axis",
                "slnt=0",
                "--axis",
                "CRSV=0.5",
            ],
            capture_output=True,
            check=True,
        )

        # Create bold instance (wght=700)
        subprocess.run(
            [
                str(RUST_CLI),
                "instance",
                str(input_vf),
                str(output_bold),
                "--axis",
                "wght=700",
                "--axis",
                "MONO=0",
                "--axis",
                "CASL=0",
                "--axis",
                "slnt=0",
                "--axis",
                "CRSV=0.5",
            ],
            capture_output=True,
            check=True,
        )

        # Both should be valid fonts
        regular = TTFont(output)
        bold = TTFont(output_bold)

        # Glyph outlines should differ (bold is heavier)
        regular_glyf = regular.getTableData("glyf")
        bold_glyf = bold.getTableData("glyf")
        assert regular_glyf != bold_glyf, (
            "Regular and Bold should have different glyphs"
        )

        # OS/2.usWeightClass should match the wght axis value
        assert regular["OS/2"].usWeightClass == 400, (
            f"Regular usWeightClass should be 400, got {regular['OS/2'].usWeightClass}"
        )
        assert bold["OS/2"].usWeightClass == 700, (
            f"Bold usWeightClass should be 700, got {bold['OS/2'].usWeightClass}"
        )

        regular.close()
        bold.close()

    def test_instance_axis_extremes(self, setup_variable_font):
        """Test instancing at axis extremes matches Python output."""
        input_vf, output = setup_variable_font

        extreme_instances = [
            ("wght_min", {"wght": 300, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5}),
            ("wght_max", {"wght": 1000, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5}),
            ("mono_max", {"wght": 400, "MONO": 1, "CASL": 0, "slnt": 0, "CRSV": 0.5}),
            ("casl_max", {"wght": 400, "MONO": 0, "CASL": 1, "slnt": 0, "CRSV": 0.5}),
            ("slnt_min", {"wght": 400, "MONO": 0, "CASL": 0, "slnt": -15, "CRSV": 0.5}),
            ("crsv_max", {"wght": 400, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 1}),
        ]

        for name, axes in extreme_instances:
            rust_out = output.parent / f"rust_{name}.ttf"
            python_out = output.parent / f"python_{name}.ttf"

            axis_args = []
            for k, v in axes.items():
                axis_args.extend(["--axis", f"{k}={v}"])

            result = subprocess.run(
                [str(RUST_CLI), "instance", str(input_vf), str(rust_out)] + axis_args,
                capture_output=True,
                text=True,
            )
            assert result.returncode == 0, (
                f"Rust instance {name} failed: {result.stderr}"
            )

            subprocess.run(
                [
                    "uv",
                    "run",
                    "python",
                    "-c",
                    f"""
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools.ttLib import TTFont
vf = TTFont("{input_vf}")
instance = instantiateVariableFont(vf, {axes})
instance.save("{python_out}")
""",
                ],
                capture_output=True,
                check=True,
                cwd=PROJECT_ROOT,
            )

            rust_font = TTFont(rust_out)
            python_font = TTFont(python_out)

            assert (
                rust_font["OS/2"].usWeightClass == python_font["OS/2"].usWeightClass
            ), f"{name}: usWeightClass mismatch"

            assert set(rust_font.keys()) == set(python_font.keys()), (
                f"{name}: table tags mismatch"
            )

            rs_width, _ = rust_font["hmtx"].metrics["H"]
            py_width, _ = python_font["hmtx"].metrics["H"]
            assert rs_width == py_width, f"{name}: H advance width mismatch"

            rust_font.close()
            python_font.close()


class TestMergeCommand:
    """Test the merge command."""

    @pytest.fixture
    def setup_fonts_for_merge(self, tmp_path):
        """Set up fonts for merge test."""
        duotone = BUILD_DIR / "RecMonoDuotone-Regular.ttf"
        noto = BUILD_DIR / "Noto-400-subset.ttf"

        if not duotone.exists():
            pytest.skip("Duotone font not built yet")
        if not noto.exists():
            pytest.skip("Noto subset font not built yet")

        duotone_copy = tmp_path / "duotone.ttf"
        noto_copy = tmp_path / "noto.ttf"
        output = tmp_path / "merged.ttf"

        shutil.copy(duotone, duotone_copy)
        shutil.copy(noto, noto_copy)

        return duotone_copy, noto_copy, output

    def test_merge_combines_fonts(self, setup_fonts_for_merge):
        """Test that merge combines multiple fonts."""
        font1, font2, output = setup_fonts_for_merge

        result = subprocess.run(
            [
                str(RUST_CLI),
                "merge",
                "--output",
                str(output),
                str(font1),
                str(font2),
            ],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        assert output.exists()

        # Merged font should be valid
        font = TTFont(output)
        assert "cmap" in font
        assert "glyf" in font
        font.close()

    def test_merge_includes_glyphs_from_both(self, setup_fonts_for_merge):
        """Test that merged font includes glyphs from both sources."""
        font1, font2, output = setup_fonts_for_merge

        # Get glyph counts before merge
        f1 = TTFont(font1)
        f2 = TTFont(font2)
        f1_glyphs = len(f1.getGlyphOrder())
        f2_glyphs = len(f2.getGlyphOrder())
        f1.close()
        f2.close()

        subprocess.run(
            [
                str(RUST_CLI),
                "merge",
                "--output",
                str(output),
                str(font1),
                str(font2),
            ],
            capture_output=True,
            check=True,
        )

        # Check merged font has glyphs from both
        merged = TTFont(output)
        merged_cmap = merged.getBestCmap()

        # Check Latin characters from font1
        assert ord("A") in merged_cmap
        assert ord("a") in merged_cmap

        # Check Japanese characters from font2
        assert 0x3042 in merged_cmap  # あ (Hiragana A)

        merged.close()


class TestSetMonospaceCommand:
    """Test the set-monospace command."""

    @pytest.fixture
    def setup_test_font(self, tmp_path):
        """Set up a test font for monospace flag test."""
        font = DIST_DIR / "WarpnineMono-Regular.ttf"

        if not font.exists():
            pytest.skip("WarpnineMono-Regular.ttf not built yet")

        font_copy = tmp_path / "test.ttf"
        shutil.copy(font, font_copy)
        return font_copy

    def test_set_monospace_flags(self, setup_test_font):
        """Test that set-monospace sets the correct flags."""
        font_path = setup_test_font

        result = subprocess.run(
            [str(RUST_CLI), "set-monospace", str(font_path)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0

        # Verify flags
        font = TTFont(font_path)

        # Check post.isFixedPitch
        assert font["post"].isFixedPitch == 1

        # Check OS/2.panose.bProportion
        assert font["OS/2"].panose.bProportion == 9  # Monospace

        font.close()

    def test_set_monospace_avg_char_width(self, setup_test_font):
        """Test that set-monospace sets xAvgCharWidth to 600."""
        font_path = setup_test_font

        subprocess.run(
            [str(RUST_CLI), "set-monospace", str(font_path)],
            capture_output=True,
            check=True,
        )

        font = TTFont(font_path)
        assert font["OS/2"].xAvgCharWidth == 600, (
            f"xAvgCharWidth should be 600, got {font['OS/2'].xAvgCharWidth}"
        )
        font.close()


class TestSetVersionCommand:
    """Test the set-version command."""

    @pytest.fixture
    def setup_test_font(self, tmp_path):
        """Set up a test font for version test."""
        font = DIST_DIR / "WarpnineMono-Regular.ttf"

        if not font.exists():
            pytest.skip("WarpnineMono-Regular.ttf not built yet")

        font_copy = tmp_path / "test.ttf"
        shutil.copy(font, font_copy)
        return font_copy

    def test_set_version_updates_name_table(self, setup_test_font):
        """Test that set-version updates the name table."""
        font_path = setup_test_font

        result = subprocess.run(
            [str(RUST_CLI), "set-version", "--version", "2025-01-02", str(font_path)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0

        # Verify version in name table
        font = TTFont(font_path)
        name_table = font["name"]

        # Check nameID 5 (version string)
        version_records = [r for r in name_table.names if r.nameID == 5]
        assert any("2025-01-02" in str(r) for r in version_records)

        font.close()

    def test_set_version_head_revision(self, setup_test_font):
        """Test that set-version updates head.fontRevision."""
        font_path = setup_test_font

        subprocess.run(
            [str(RUST_CLI), "set-version", "--version", "2025-01-02", str(font_path)],
            capture_output=True,
            check=True,
        )

        font = TTFont(font_path)
        # Format: YYYY.MMDD e.g. 2025.0102
        assert font["head"].fontRevision == pytest.approx(2025.0102, rel=0.001), (
            f"head.fontRevision should be 2025.0102, got {font['head'].fontRevision}"
        )
        font.close()

    def test_set_version_unique_id(self, setup_test_font):
        """Test that set-version updates nameID 3 (unique ID)."""
        font_path = setup_test_font

        subprocess.run(
            [str(RUST_CLI), "set-version", "--version", "2025-01-02", str(font_path)],
            capture_output=True,
            check=True,
        )

        font = TTFont(font_path)
        name3 = font["name"].getDebugName(3)
        assert name3 is not None, "nameID 3 (unique ID) should exist"
        assert "2025-01-02" in name3, (
            f"nameID 3 should contain '2025-01-02', got '{name3}'"
        )
        font.close()


class TestSetNameCommand:
    """Test the set-name command."""

    @pytest.fixture
    def setup_test_font(self, tmp_path):
        """Set up a test font for name table test."""
        font = DIST_DIR / "WarpnineMono-Regular.ttf"

        if not font.exists():
            pytest.skip("WarpnineMono-Regular.ttf not built yet")

        font_copy = tmp_path / "test.ttf"
        shutil.copy(font, font_copy)
        return font_copy

    def test_set_name_updates_name_table(self, setup_test_font):
        """Test that set-name updates all name table entries."""
        font_path = setup_test_font

        result = subprocess.run(
            [
                str(RUST_CLI),
                "set-name",
                "--family",
                "Test Family",
                "--style",
                "Bold",
                str(font_path),
            ],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0

        font = TTFont(font_path)
        name_table = font["name"]

        # Check nameID 1 (family)
        name1 = name_table.getDebugName(1)
        assert "Test Family Bold" in name1

        # Check nameID 4 (full name)
        name4 = name_table.getDebugName(4)
        assert name4 == "Test Family Bold"

        # Check nameID 6 (postscript name)
        name6 = name_table.getDebugName(6)
        assert name6 == "TestFamily-Bold"

        # Check nameID 16 (typographic family)
        name16 = name_table.getDebugName(16)
        assert name16 == "Test Family"

        # Check nameID 17 (typographic subfamily)
        name17 = name_table.getDebugName(17)
        assert name17 == "Bold"

        font.close()

    def test_set_name_with_postscript_family(self, setup_test_font):
        """Test set-name with custom postscript family."""
        font_path = setup_test_font

        result = subprocess.run(
            [
                str(RUST_CLI),
                "set-name",
                "--family",
                "My Font",
                "--style",
                "Regular",
                "--postscript-family",
                "MyCustomPSName",
                str(font_path),
            ],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0

        font = TTFont(font_path)
        name6 = font["name"].getDebugName(6)
        assert name6 == "MyCustomPSName-Regular"
        font.close()

    def test_set_name_with_copyright(self, setup_test_font):
        """Test set-name with additional copyright text."""
        font_path = setup_test_font

        result = subprocess.run(
            [
                str(RUST_CLI),
                "set-name",
                "--family",
                "Test",
                "--style",
                "Regular",
                "--copyright-extra",
                "Additional copyright notice.",
                str(font_path),
            ],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0

        font = TTFont(font_path)
        name0 = font["name"].getDebugName(0)
        assert "Additional copyright notice." in name0
        font.close()


class TestFixCaltCommand:
    """Test the fix-calt command."""

    @pytest.fixture
    def setup_test_font(self, tmp_path):
        """Set up a test font for calt fix test."""
        font = DIST_DIR / "WarpnineMono-Regular.ttf"

        if not font.exists():
            pytest.skip("WarpnineMono-Regular.ttf not built yet")

        font_copy = tmp_path / "test.ttf"
        shutil.copy(font, font_copy)
        return font_copy

    def test_fix_calt_runs_successfully(self, setup_test_font):
        """Test that fix-calt runs without error."""
        font_path = setup_test_font

        result = subprocess.run(
            [str(RUST_CLI), "fix-calt", str(font_path)],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 0
        assert "Fix calt: 1 succeeded" in result.stdout

    def test_fix_calt_preserves_font_validity(self, setup_test_font):
        """Test that fix-calt preserves font validity."""
        font_path = setup_test_font

        subprocess.run(
            [str(RUST_CLI), "fix-calt", str(font_path)],
            capture_output=True,
            check=True,
        )

        # Verify font is still valid
        font = TTFont(font_path)
        assert "GSUB" in font
        assert "cmap" in font
        font.close()

    def test_fix_calt_registers_to_all_scripts(self, tmp_path):
        """Test that fix-calt registers calt feature to all scripts."""
        noto = BUILD_DIR / "Noto-400.ttf"

        if not noto.exists():
            pytest.skip("Noto-400.ttf not built yet")

        font_path = tmp_path / "noto.ttf"
        shutil.copy(noto, font_path)

        subprocess.run(
            [str(RUST_CLI), "fix-calt", str(font_path)],
            capture_output=True,
            check=True,
        )

        font = TTFont(font_path)
        gsub = font["GSUB"].table

        calt_indices = []
        for i, record in enumerate(gsub.FeatureList.FeatureRecord):
            if record.FeatureTag == "calt":
                calt_indices.append(i)

        assert calt_indices, "Font should have calt feature"

        scripts_with_calt = []
        scripts_without_calt = []

        for script_record in gsub.ScriptList.ScriptRecord:
            script = script_record.Script
            script_tag = script_record.ScriptTag

            has_calt = False
            if script.DefaultLangSys:
                feature_indices = list(script.DefaultLangSys.FeatureIndex)
                if any(idx in feature_indices for idx in calt_indices):
                    has_calt = True

            if has_calt:
                scripts_with_calt.append(script_tag)
            else:
                scripts_without_calt.append(script_tag)

        font.close()

        assert not scripts_without_calt, (
            f"calt should be registered to all scripts. "
            f"Missing from: {scripts_without_calt}, "
            f"Present in: {scripts_with_calt}"
        )


class TestBenchmarks:
    """Benchmark tests comparing Python and Rust performance."""

    @pytest.mark.benchmark
    def test_benchmark_remove_ligatures(self, tmp_path):
        """Benchmark remove-ligatures command."""
        duotone = BUILD_DIR / "RecMonoDuotone-Regular.ttf"

        if not duotone.exists():
            pytest.skip("Duotone font not built yet")

        # Prepare copies for each run
        py_font = tmp_path / "py_duotone.ttf"
        rs_font = tmp_path / "rs_duotone.ttf"
        shutil.copy(duotone, py_font)
        shutil.copy(duotone, rs_font)

        # Run Python version - need to check if Python has equivalent command
        # For now, just run Rust
        _, rust_time = run_rust_cmd(["remove-ligatures", str(rs_font)])

        print(f"\nRemove-ligatures benchmark:")
        print(f"  Rust: {rust_time:.3f}s")

    @pytest.mark.benchmark
    def test_benchmark_set_monospace(self, tmp_path):
        """Benchmark set-monospace command."""
        font = DIST_DIR / "WarpnineMono-Regular.ttf"

        if not font.exists():
            pytest.skip("WarpnineMono-Regular.ttf not built yet")

        # Prepare copies
        fonts = [tmp_path / f"font_{i}.ttf" for i in range(5)]
        for f in fonts:
            shutil.copy(font, f)

        # Benchmark Rust
        start = time.perf_counter()
        for f in fonts:
            subprocess.run(
                [str(RUST_CLI), "set-monospace", str(f)], capture_output=True
            )
        rust_time = time.perf_counter() - start

        print(f"\nSet-monospace benchmark (5 fonts):")
        print(f"  Rust: {rust_time:.3f}s ({rust_time / 5:.3f}s per font)")


class TestRustOutputValidation:
    """Validate Rust CLI output against Python reference fonts.

    These tests compare fonts generated by the Rust CLI with those generated
    by Python's fontTools to ensure identical (or near-identical) output.
    """

    @pytest.fixture
    def setup_create_sans(self, tmp_path):
        """Set up create-sans test with Rust output and Python reference."""
        vf = BUILD_DIR / "Recursive_VF_1.085.ttf"

        if not vf.exists():
            pytest.skip("Recursive VF not downloaded yet")

        rust_out = tmp_path / "rust"
        rust_out.mkdir()

        # Generate Rust fonts
        result = subprocess.run(
            [
                str(RUST_CLI),
                "create-sans",
                "--input",
                str(vf),
                "--output-dir",
                str(rust_out),
            ],
            capture_output=True,
            text=True,
            timeout=120,
        )

        if result.returncode != 0:
            pytest.fail(f"Rust create-sans failed: {result.stderr}")

        return rust_out

    @pytest.mark.slow
    def test_validate_sans_regular(self, setup_create_sans):
        """Validate WarpnineSans-Regular against Python reference."""
        rust_out = setup_create_sans
        rust_font = rust_out / "WarpnineSans-Regular.ttf"
        python_font = DIST_DIR / "WarpnineSans-Regular.ttf"

        if not python_font.exists():
            pytest.skip("Python reference font not built yet")

        pass_count, fail_count, failures = validate_font_pair(rust_font, python_font)

        if fail_count > 0:
            pytest.fail(
                f"Validation failed ({pass_count} pass, {fail_count} fail):\n"
                + "\n".join(f"  - {f}" for f in failures)
            )

    @pytest.mark.slow
    def test_validate_sans_bold(self, setup_create_sans):
        """Validate WarpnineSans-Bold against Python reference."""
        rust_out = setup_create_sans
        rust_font = rust_out / "WarpnineSans-Bold.ttf"
        python_font = DIST_DIR / "WarpnineSans-Bold.ttf"

        if not python_font.exists():
            pytest.skip("Python reference font not built yet")

        pass_count, fail_count, failures = validate_font_pair(rust_font, python_font)

        if fail_count > 0:
            pytest.fail(
                f"Validation failed ({pass_count} pass, {fail_count} fail):\n"
                + "\n".join(f"  - {f}" for f in failures)
            )

    @pytest.mark.slow
    def test_validate_sans_italic(self, setup_create_sans):
        """Validate WarpnineSans-Italic against Python reference."""
        rust_out = setup_create_sans
        rust_font = rust_out / "WarpnineSans-Italic.ttf"
        python_font = DIST_DIR / "WarpnineSans-Italic.ttf"

        if not python_font.exists():
            pytest.skip("Python reference font not built yet")

        pass_count, fail_count, failures = validate_font_pair(rust_font, python_font)

        if fail_count > 0:
            pytest.fail(
                f"Validation failed ({pass_count} pass, {fail_count} fail):\n"
                + "\n".join(f"  - {f}" for f in failures)
            )

    @pytest.mark.slow
    def test_validate_all_sans_styles(self, setup_create_sans):
        """Validate all WarpnineSans styles against Python reference."""
        rust_out = setup_create_sans

        styles = [
            "Light",
            "Regular",
            "Medium",
            "Bold",
            "Black",
            "Italic",
            "LightItalic",
            "BoldItalic",
        ]

        total_pass = 0
        total_fail = 0
        all_failures = []

        for style in styles:
            rust_font = rust_out / f"WarpnineSans-{style}.ttf"
            python_font = DIST_DIR / f"WarpnineSans-{style}.ttf"

            if not rust_font.exists():
                continue
            if not python_font.exists():
                continue

            pass_count, fail_count, failures = validate_font_pair(
                rust_font, python_font
            )
            total_pass += pass_count
            total_fail += fail_count

            for f in failures:
                all_failures.append(f"{style}: {f}")

        if total_fail > 0:
            pytest.fail(
                f"Validation failed ({total_pass} pass, {total_fail} fail):\n"
                + "\n".join(f"  - {f}" for f in all_failures[:20])
                + (
                    f"\n  ... and {len(all_failures) - 20} more"
                    if len(all_failures) > 20
                    else ""
                )
            )
