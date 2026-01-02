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
