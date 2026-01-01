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
RUST_CLI = (
    PROJECT_ROOT / "rust" / "warpnine-fonts" / "target" / "release" / "warpnine-fonts"
)
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
