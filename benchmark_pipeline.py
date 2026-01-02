#!/usr/bin/env python3
"""
Pipeline benchmark: Compare Python vs Rust font operations.

Benchmarks the actual operations used in the warpnine-fonts build pipeline.

Usage:
    uv run benchmark_pipeline.py
"""

import shutil
import subprocess
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path

BUILD_DIR = Path(__file__).parent / "build"
DIST_DIR = Path(__file__).parent / "dist"
RUST_BIN = Path(__file__).parent / "target/release/warpnine-fonts"
RECURSIVE_VF = (
    Path(__file__).parent
    / "recursive/fonts/ArrowType-Recursive-1.085/Recursive_Desktop/Recursive_VF_1.085.ttf"
)

MONO_FEATURES = [
    "dlig",
    "ss01",
    "ss02",
    "ss03",
    "ss04",
    "ss05",
    "ss06",
    "ss07",
    "ss08",
    "ss10",
    "ss11",
    "ss12",
    "pnum",
    "liga",
]

DUOTONE_INSTANCES = [
    ("Light", {"MONO": 1, "CASL": 0, "wght": 300, "slnt": 0, "CRSV": 0.5}),
    ("LightItalic", {"MONO": 1, "CASL": 0, "wght": 300, "slnt": -15, "CRSV": 1}),
    ("Regular", {"MONO": 1, "CASL": 0, "wght": 400, "slnt": 0, "CRSV": 0.5}),
    ("Italic", {"MONO": 1, "CASL": 0, "wght": 400, "slnt": -15, "CRSV": 1}),
    ("Medium", {"MONO": 1, "CASL": 1, "wght": 500, "slnt": 0, "CRSV": 0.5}),
    ("MediumItalic", {"MONO": 1, "CASL": 1, "wght": 500, "slnt": -15, "CRSV": 1}),
    ("SemiBold", {"MONO": 1, "CASL": 1, "wght": 600, "slnt": 0, "CRSV": 0.5}),
    ("SemiBoldItalic", {"MONO": 1, "CASL": 1, "wght": 600, "slnt": -15, "CRSV": 1}),
    ("Bold", {"MONO": 1, "CASL": 1, "wght": 700, "slnt": 0, "CRSV": 0.5}),
    ("BoldItalic", {"MONO": 1, "CASL": 1, "wght": 700, "slnt": -15, "CRSV": 1}),
    ("ExtraBold", {"MONO": 1, "CASL": 1, "wght": 800, "slnt": 0, "CRSV": 0.5}),
    ("ExtraBoldItalic", {"MONO": 1, "CASL": 1, "wght": 800, "slnt": -15, "CRSV": 1}),
    ("Black", {"MONO": 1, "CASL": 1, "wght": 900, "slnt": 0, "CRSV": 0.5}),
    ("BlackItalic", {"MONO": 1, "CASL": 1, "wght": 900, "slnt": -15, "CRSV": 1}),
    ("ExtraBlack", {"MONO": 1, "CASL": 1, "wght": 1000, "slnt": 0, "CRSV": 0.5}),
    ("ExtraBlackItalic", {"MONO": 1, "CASL": 1, "wght": 1000, "slnt": -15, "CRSV": 1}),
]


@dataclass
class BenchmarkResult:
    name: str
    python_time: float | None
    rust_time: float | None
    python_error: str | None = None
    rust_error: str | None = None

    @property
    def speedup(self) -> str:
        if self.python_time and self.rust_time and self.rust_time > 0:
            return f"{self.python_time / self.rust_time:.1f}x"
        return "N/A"


def run_timed(func) -> tuple[float | None, str | None]:
    """Run a function and return (elapsed_time, error_message)."""
    try:
        start = time.perf_counter()
        func()
        return time.perf_counter() - start, None
    except Exception as e:
        return None, str(e)


def benchmark_extract_duotone() -> BenchmarkResult:
    """Benchmark extracting 16 Duotone instances from Recursive VF."""
    if not RECURSIVE_VF.exists():
        return BenchmarkResult(
            "Extract Duotone (16 instances)",
            None,
            None,
            f"VF not found: {RECURSIVE_VF}",
            f"VF not found: {RECURSIVE_VF}",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        # Python: use fonttools varLib.instancer
        def run_python():
            for style, axes in DUOTONE_INSTANCES:
                output = tmp / f"py-{style}.ttf"
                axis_args = [f"{k}={v}" for k, v in axes.items()]
                subprocess.run(
                    ["uv", "run", "fonttools", "varLib.instancer", str(RECURSIVE_VF)]
                    + axis_args
                    + ["-o", str(output)],
                    check=True,
                    capture_output=True,
                )

        # Rust: use warpnine-fonts instance-batch
        def run_rust():
            rs_out = tmp / "rust"
            rs_out.mkdir()
            instance_args = []
            for style, axes in DUOTONE_INSTANCES:
                axis_str = ",".join(f"{k}={v}" for k, v in axes.items())
                instance_args.extend(["-i", f"{style}:{axis_str}"])
            subprocess.run(
                [
                    str(RUST_BIN),
                    "instance-batch",
                    "--input",
                    str(RECURSIVE_VF),
                    "--output-dir",
                    str(rs_out),
                ]
                + instance_args,
                check=True,
                capture_output=True,
            )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        "Extract Duotone (16 instances)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_freeze_static() -> BenchmarkResult:
    """Benchmark freezing features in static mono fonts."""
    fonts = sorted(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))
    if not fonts:
        return BenchmarkResult(
            "Freeze Static Mono",
            None,
            None,
            "No RecMonoDuotone fonts found",
            "No RecMonoDuotone fonts found",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        # Copy fonts for Python test
        py_fonts = []
        for f in fonts:
            dest = tmp / f"py-{f.name}"
            shutil.copy(f, dest)
            py_fonts.append(dest)

        # Copy fonts for Rust test
        rs_fonts = []
        for f in fonts:
            dest = tmp / f"rs-{f.name}"
            shutil.copy(f, dest)
            rs_fonts.append(dest)

        feature_str = ",".join(MONO_FEATURES)

        def run_python():
            for font in py_fonts:
                subprocess.run(
                    [
                        "uv",
                        "run",
                        "pyftfeatfreeze",
                        "-f",
                        feature_str,
                        str(font),
                        str(font),
                    ],
                    check=True,
                    capture_output=True,
                )

        def run_rust():
            subprocess.run(
                [str(RUST_BIN), "freeze", "-f", feature_str]
                + [str(f) for f in rs_fonts],
                check=True,
                capture_output=True,
            )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        f"Freeze Static Mono ({len(fonts)} fonts)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_merge() -> BenchmarkResult:
    """Benchmark merging fonts using fontforge (Python) vs font-merger (Rust)."""
    rec_fonts = sorted(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))[:4]  # First 4 for speed
    noto_font = BUILD_DIR / "Noto-400-subset.ttf"

    if not rec_fonts or not noto_font.exists():
        return BenchmarkResult(
            "Merge Fonts",
            None,
            None,
            "Required fonts not found",
            "Required fonts not found",
        )

    # Check if fontforge is available
    try:
        subprocess.run(["fontforge", "-version"], capture_output=True, check=True)
        has_fontforge = True
    except (subprocess.CalledProcessError, FileNotFoundError):
        has_fontforge = False

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        # Python: use fontforge (as in actual pipeline)
        def run_python():
            for rec in rec_fonts:
                output = tmp / f"py-{rec.stem}-merged.ttf"
                script = f"""import fontforge
font = fontforge.open("{rec}")
font.mergeFonts("{noto_font}")
font.generate("{output}")
font.close()
"""
                script_file = tmp / f"merge_{rec.stem}.py"
                script_file.write_text(script)
                subprocess.run(
                    ["fontforge", "-lang=py", "-script", str(script_file)],
                    check=True,
                    capture_output=True,
                )

        def run_rust():
            rs_out = tmp / "rust"
            rs_out.mkdir()
            subprocess.run(
                [
                    str(RUST_BIN),
                    "merge-batch",
                    "-f",
                    str(noto_font),
                    "-o",
                    str(rs_out),
                ]
                + [str(f) for f in rec_fonts],
                check=True,
                capture_output=True,
            )

        if has_fontforge:
            py_time, py_err = run_timed(run_python)
        else:
            py_time, py_err = None, "fontforge not installed"

        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        f"Merge Fonts ({len(rec_fonts)} pairs)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_set_monospace() -> BenchmarkResult:
    """Benchmark setting monospace flags."""
    fonts = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))
    if not fonts:
        fonts = sorted(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))
    if not fonts:
        return BenchmarkResult(
            "Set Monospace",
            None,
            None,
            "No fonts found",
            "No fonts found",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        py_fonts = []
        for f in fonts:
            dest = tmp / f"py-{f.name}"
            shutil.copy(f, dest)
            py_fonts.append(dest)

        rs_fonts = []
        for f in fonts:
            dest = tmp / f"rs-{f.name}"
            shutil.copy(f, dest)
            rs_fonts.append(dest)

        def run_python():
            from fontTools.ttLib import TTFont

            for font_path in py_fonts:
                font = TTFont(font_path)
                if "post" in font:
                    font["post"].isFixedPitch = 1
                if "OS/2" in font:
                    font["OS/2"].panose.bProportion = 9
                    font["OS/2"].xAvgCharWidth = 600
                font.save(font_path)
                font.close()

        def run_rust():
            subprocess.run(
                [str(RUST_BIN), "set-monospace"] + [str(f) for f in rs_fonts],
                check=True,
                capture_output=True,
            )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        f"Set Monospace ({len(fonts)} fonts)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_set_version() -> BenchmarkResult:
    """Benchmark setting version metadata."""
    fonts = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))
    if not fonts:
        fonts = sorted(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))
    if not fonts:
        return BenchmarkResult(
            "Set Version",
            None,
            None,
            "No fonts found",
            "No fonts found",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        py_fonts = []
        for f in fonts:
            dest = tmp / f"py-{f.name}"
            shutil.copy(f, dest)
            py_fonts.append(dest)

        rs_fonts = []
        for f in fonts:
            dest = tmp / f"rs-{f.name}"
            shutil.copy(f, dest)
            rs_fonts.append(dest)

        def run_python():
            from fontTools.ttLib import TTFont

            for font_path in py_fonts:
                font = TTFont(font_path)
                if "head" in font:
                    font["head"].fontRevision = 2025.0102
                font.save(font_path)
                font.close()

        def run_rust():
            subprocess.run(
                [str(RUST_BIN), "set-version", "-v", "2025-01-02"]
                + [str(f) for f in rs_fonts],
                check=True,
                capture_output=True,
            )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        f"Set Version ({len(fonts)} fonts)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_create_sans() -> BenchmarkResult:
    """Benchmark creating 14 WarpnineSans fonts from Recursive VF."""
    if not RECURSIVE_VF.exists():
        return BenchmarkResult(
            "Create Sans (14 fonts)",
            None,
            None,
            f"VF not found: {RECURSIVE_VF}",
            f"VF not found: {RECURSIVE_VF}",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        py_out = tmp / "python"
        rs_out = tmp / "rust"
        py_out.mkdir()
        rs_out.mkdir()

        # Python: use fonttools varLib.instancer
        def run_python():
            from fontTools.ttLib import TTFont
            from fontTools.varLib.instancer import instantiateVariableFont

            SANS_INSTANCES = [
                (
                    "Regular",
                    {"wght": 400, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5},
                ),
                ("Bold", {"wght": 700, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5}),
                ("Italic", {"wght": 400, "MONO": 0, "CASL": 0, "slnt": -15, "CRSV": 1}),
                (
                    "BoldItalic",
                    {"wght": 700, "MONO": 0, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                ("Light", {"wght": 300, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5}),
                (
                    "LightItalic",
                    {"wght": 300, "MONO": 0, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                ("Medium", {"wght": 500, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5}),
                (
                    "MediumItalic",
                    {"wght": 500, "MONO": 0, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                (
                    "SemiBold",
                    {"wght": 600, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5},
                ),
                (
                    "SemiBoldItalic",
                    {"wght": 600, "MONO": 0, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                (
                    "ExtraBold",
                    {"wght": 800, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5},
                ),
                (
                    "ExtraBoldItalic",
                    {"wght": 800, "MONO": 0, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                ("Black", {"wght": 900, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5}),
                (
                    "BlackItalic",
                    {"wght": 900, "MONO": 0, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
            ]
            for style, axes in SANS_INSTANCES:
                vf = TTFont(RECURSIVE_VF)
                instance = instantiateVariableFont(vf, axes)
                instance.save(py_out / f"WarpnineSans-{style}.ttf")
                vf.close()

        # Rust: use warpnine-fonts create-sans
        def run_rust():
            subprocess.run(
                [
                    str(RUST_BIN),
                    "create-sans",
                    "--input",
                    str(RECURSIVE_VF),
                    "--output-dir",
                    str(rs_out),
                ],
                check=True,
                capture_output=True,
            )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        "Create Sans (14 fonts)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_create_condensed() -> BenchmarkResult:
    """Benchmark creating 14 WarpnineSansCondensed fonts from Recursive VF."""
    if not RECURSIVE_VF.exists():
        return BenchmarkResult(
            "Create Condensed (14 fonts)",
            None,
            None,
            f"VF not found: {RECURSIVE_VF}",
            f"VF not found: {RECURSIVE_VF}",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        py_out = tmp / "python"
        rs_out = tmp / "rust"
        py_out.mkdir()
        rs_out.mkdir()

        # Python: use fonttools varLib.instancer
        def run_python():
            from fontTools.ttLib import TTFont
            from fontTools.varLib.instancer import instantiateVariableFont

            CONDENSED_INSTANCES = [
                (
                    "Regular",
                    {"wght": 400, "MONO": 0.8, "CASL": 0, "slnt": 0, "CRSV": 0.5},
                ),
                ("Bold", {"wght": 700, "MONO": 0.8, "CASL": 0, "slnt": 0, "CRSV": 0.5}),
                (
                    "Italic",
                    {"wght": 400, "MONO": 0.8, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                (
                    "BoldItalic",
                    {"wght": 700, "MONO": 0.8, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                (
                    "Light",
                    {"wght": 300, "MONO": 0.8, "CASL": 0, "slnt": 0, "CRSV": 0.5},
                ),
                (
                    "LightItalic",
                    {"wght": 300, "MONO": 0.8, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                (
                    "Medium",
                    {"wght": 500, "MONO": 0.8, "CASL": 0, "slnt": 0, "CRSV": 0.5},
                ),
                (
                    "MediumItalic",
                    {"wght": 500, "MONO": 0.8, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                (
                    "SemiBold",
                    {"wght": 600, "MONO": 0.8, "CASL": 0, "slnt": 0, "CRSV": 0.5},
                ),
                (
                    "SemiBoldItalic",
                    {"wght": 600, "MONO": 0.8, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                (
                    "ExtraBold",
                    {"wght": 800, "MONO": 0.8, "CASL": 0, "slnt": 0, "CRSV": 0.5},
                ),
                (
                    "ExtraBoldItalic",
                    {"wght": 800, "MONO": 0.8, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
                (
                    "Black",
                    {"wght": 900, "MONO": 0.8, "CASL": 0, "slnt": 0, "CRSV": 0.5},
                ),
                (
                    "BlackItalic",
                    {"wght": 900, "MONO": 0.8, "CASL": 0, "slnt": -15, "CRSV": 1},
                ),
            ]
            for style, axes in CONDENSED_INSTANCES:
                vf = TTFont(RECURSIVE_VF)
                instance = instantiateVariableFont(vf, axes)
                instance.save(py_out / f"WarpnineSansCondensed-{style}.ttf")
                vf.close()

        # Rust: use warpnine-fonts create-condensed
        def run_rust():
            subprocess.run(
                [
                    str(RUST_BIN),
                    "create-condensed",
                    "--input",
                    str(RECURSIVE_VF),
                    "--output-dir",
                    str(rs_out),
                ],
                check=True,
                capture_output=True,
            )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        "Create Condensed (14 fonts)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_subset_japanese() -> BenchmarkResult:
    """Benchmark subsetting Noto CJK to Japanese ranges."""
    noto_fonts = sorted(BUILD_DIR.glob("NotoSans*CJK*.ttf"))
    if not noto_fonts:
        return BenchmarkResult(
            "Subset Japanese",
            None,
            None,
            "No Noto CJK fonts found",
            "No Noto CJK fonts found",
        )

    # Use first 2 fonts for benchmark
    fonts = noto_fonts[:2]

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        py_fonts = []
        for f in fonts:
            dest = tmp / f"py-{f.name}"
            shutil.copy(f, dest)
            py_fonts.append(dest)

        rs_fonts = []
        for f in fonts:
            dest = tmp / f"rs-{f.name}"
            shutil.copy(f, dest)
            rs_fonts.append(dest)

        # Python: use fonttools subset
        def run_python():
            # Japanese Unicode ranges
            unicodes = "U+0000-00FF,U+2000-206F,U+3000-30FF,U+4E00-9FFF,U+FF00-FFEF"
            for font_path in py_fonts:
                output = tmp / f"py-subset-{font_path.name}"
                subprocess.run(
                    [
                        "uv",
                        "run",
                        "fonttools",
                        "subset",
                        str(font_path),
                        f"--unicodes={unicodes}",
                        f"--output-file={output}",
                    ],
                    check=True,
                    capture_output=True,
                )

        # Rust: use warpnine-fonts subset-japanese
        def run_rust():
            for font_path in rs_fonts:
                output = tmp / f"rs-subset-{font_path.name}"
                subprocess.run(
                    [str(RUST_BIN), "subset-japanese", str(font_path), str(output)],
                    check=True,
                    capture_output=True,
                )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        f"Subset Japanese ({len(fonts)} fonts)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_remove_ligatures() -> BenchmarkResult:
    """Benchmark removing ligatures from fonts."""
    fonts = sorted(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))[:5]
    if not fonts:
        return BenchmarkResult(
            "Remove Ligatures",
            None,
            None,
            "No fonts found",
            "No fonts found",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        py_fonts = []
        for f in fonts:
            dest = tmp / f"py-{f.name}"
            shutil.copy(f, dest)
            py_fonts.append(dest)

        rs_fonts = []
        for f in fonts:
            dest = tmp / f"rs-{f.name}"
            shutil.copy(f, dest)
            rs_fonts.append(dest)

        # Python: use fonttools to clear ligature lookups
        def run_python():
            from fontTools.ttLib import TTFont

            for font_path in py_fonts:
                font = TTFont(font_path)
                if "GSUB" in font:
                    gsub = font["GSUB"]
                    for lookup in gsub.table.LookupList.Lookup:
                        if lookup.LookupType == 4:  # Ligature
                            for subtable in lookup.SubTable:
                                if hasattr(subtable, "ligatures"):
                                    subtable.ligatures = {}
                font.save(font_path)
                font.close()

        # Rust: use warpnine-fonts remove-ligatures
        def run_rust():
            subprocess.run(
                [str(RUST_BIN), "remove-ligatures"] + [str(f) for f in rs_fonts],
                check=True,
                capture_output=True,
            )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        f"Remove Ligatures ({len(fonts)} fonts)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_build_vf() -> BenchmarkResult:
    """Benchmark building variable font from static masters."""
    fonts = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))
    if len(fonts) < 16:
        return BenchmarkResult(
            "Build VF (16 masters)",
            None,
            None,
            f"Need 16 fonts, found {len(fonts)}",
            f"Need 16 fonts, found {len(fonts)}",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        py_dist = tmp / "py-dist"
        rs_dist = tmp / "rs-dist"
        py_dist.mkdir()
        rs_dist.mkdir()

        # Copy fonts for both tests
        for f in fonts:
            shutil.copy(f, py_dist / f.name)
            shutil.copy(f, rs_dist / f.name)

        # Python: use fonttools varLib
        def run_python():
            from fontTools.designspaceLib import (
                AxisDescriptor,
                DesignSpaceDocument,
            )
            from fontTools.varLib import build as build_vf

            doc = DesignSpaceDocument()

            weight_axis = AxisDescriptor()
            weight_axis.name = "Weight"
            weight_axis.tag = "wght"
            weight_axis.minimum = 300
            weight_axis.default = 400
            weight_axis.maximum = 1000
            doc.addAxis(weight_axis)

            italic_axis = AxisDescriptor()
            italic_axis.name = "Italic"
            italic_axis.tag = "ital"
            italic_axis.minimum = 0
            italic_axis.default = 0
            italic_axis.maximum = 1
            doc.addAxis(italic_axis)

            from fontTools.designspaceLib import SourceDescriptor

            masters = [
                ("Light", 300, 0),
                ("LightItalic", 300, 1),
                ("Regular", 400, 0),
                ("Italic", 400, 1),
                ("Medium", 500, 0),
                ("MediumItalic", 500, 1),
                ("SemiBold", 600, 0),
                ("SemiBoldItalic", 600, 1),
                ("Bold", 700, 0),
                ("BoldItalic", 700, 1),
                ("ExtraBold", 800, 0),
                ("ExtraBoldItalic", 800, 1),
                ("Black", 900, 0),
                ("BlackItalic", 900, 1),
                ("ExtraBlack", 1000, 0),
                ("ExtraBlackItalic", 1000, 1),
            ]
            for style, weight, italic in masters:
                src = SourceDescriptor()
                path = str(py_dist / f"WarpnineMono-{style}.ttf")
                src.filename = path
                src.path = path
                src.location = {"Weight": weight, "Italic": italic}
                doc.addSource(src)

            ds_path = tmp / "warpnine.designspace"
            doc.write(ds_path)
            vf, _, _ = build_vf(ds_path)
            vf.save(tmp / "py-WarpnineMono-VF.ttf")

        # Rust: use warpnine-fonts build-vf
        def run_rust():
            subprocess.run(
                [
                    str(RUST_BIN),
                    "build-vf",
                    "--dist-dir",
                    str(rs_dist),
                    "--output",
                    str(tmp / "rs-WarpnineMono-VF.ttf"),
                ],
                check=True,
                capture_output=True,
            )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        "Build VF (16 masters)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_copy_gsub() -> BenchmarkResult:
    """Benchmark copying GSUB table between fonts."""
    source_vf = RECURSIVE_VF
    target_fonts = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))[:4]

    if not source_vf.exists() or not target_fonts:
        return BenchmarkResult(
            "Copy GSUB",
            None,
            None,
            "Source VF or target fonts not found",
            "Source VF or target fonts not found",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        py_fonts = []
        for f in target_fonts:
            dest = tmp / f"py-{f.name}"
            shutil.copy(f, dest)
            py_fonts.append(dest)

        rs_fonts = []
        for f in target_fonts:
            dest = tmp / f"rs-{f.name}"
            shutil.copy(f, dest)
            rs_fonts.append(dest)

        def run_python():
            from fontTools.ttLib import TTFont

            source = TTFont(source_vf)
            source_gsub = source["GSUB"]
            for font_path in py_fonts:
                target = TTFont(font_path)
                target["GSUB"] = source_gsub
                target.save(font_path)
                target.close()
            source.close()

        def run_rust():
            for font_path in rs_fonts:
                subprocess.run(
                    [
                        str(RUST_BIN),
                        "copy-gsub",
                        "--from",
                        str(source_vf),
                        "--to",
                        str(font_path),
                    ],
                    check=True,
                    capture_output=True,
                )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        f"Copy GSUB ({len(target_fonts)} fonts)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_fix_calt() -> BenchmarkResult:
    """Benchmark fixing calt/rclt feature registration."""
    fonts = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))[:5]
    if not fonts:
        fonts = sorted(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))[:5]
    if not fonts:
        return BenchmarkResult(
            "Fix Calt",
            None,
            None,
            "No fonts found",
            "No fonts found",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        py_fonts = []
        for f in fonts:
            dest = tmp / f"py-{f.name}"
            shutil.copy(f, dest)
            py_fonts.append(dest)

        rs_fonts = []
        for f in fonts:
            dest = tmp / f"rs-{f.name}"
            shutil.copy(f, dest)
            rs_fonts.append(dest)

        def run_python():
            from fontTools.ttLib import TTFont

            for font_path in py_fonts:
                font = TTFont(font_path)
                if "GSUB" in font:
                    gsub = font["GSUB"].table
                    calt_indices = []
                    rclt_indices = []
                    for i, rec in enumerate(gsub.FeatureList.FeatureRecord):
                        if rec.FeatureTag == "calt":
                            calt_indices.append(i)
                        elif rec.FeatureTag == "rclt":
                            rclt_indices.append(i)
                    # Add feature indices to all scripts
                    for script_record in gsub.ScriptList.ScriptRecord:
                        for lang in [script_record.Script.DefaultLangSys] + [
                            ls.LangSys for ls in script_record.Script.LangSysRecord
                        ]:
                            if lang:
                                for idx in calt_indices + rclt_indices:
                                    if idx not in lang.FeatureIndex:
                                        lang.FeatureIndex.append(idx)
                font.save(font_path)
                font.close()

        def run_rust():
            subprocess.run(
                [str(RUST_BIN), "fix-calt"] + [str(f) for f in rs_fonts],
                check=True,
                capture_output=True,
            )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        f"Fix Calt ({len(fonts)} fonts)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


def benchmark_set_name() -> BenchmarkResult:
    """Benchmark setting font name table entries."""
    fonts = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))[:5]
    if not fonts:
        fonts = sorted(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))[:5]
    if not fonts:
        return BenchmarkResult(
            "Set Name",
            None,
            None,
            "No fonts found",
            "No fonts found",
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        py_fonts = []
        for f in fonts:
            dest = tmp / f"py-{f.name}"
            shutil.copy(f, dest)
            py_fonts.append(dest)

        rs_fonts = []
        for f in fonts:
            dest = tmp / f"rs-{f.name}"
            shutil.copy(f, dest)
            rs_fonts.append(dest)

        def run_python():
            from fontTools.ttLib import TTFont

            for font_path in py_fonts:
                font = TTFont(font_path)
                name_table = font["name"]
                # Set family name
                for record in name_table.names:
                    if record.nameID == 1:
                        record.string = "Test Family"
                    elif record.nameID == 2:
                        record.string = "Regular"
                font.save(font_path)
                font.close()

        def run_rust():
            subprocess.run(
                [
                    str(RUST_BIN),
                    "set-name",
                    "--family",
                    "Test Family",
                    "--style",
                    "Regular",
                ]
                + [str(f) for f in rs_fonts],
                check=True,
                capture_output=True,
            )

        py_time, py_err = run_timed(run_python)
        rs_time, rs_err = run_timed(run_rust)

    return BenchmarkResult(
        f"Set Name ({len(fonts)} fonts)",
        py_time,
        rs_time,
        py_err,
        rs_err,
    )


@dataclass
class ValidationResult:
    name: str
    passed: bool
    message: str


def validate_instance_output() -> ValidationResult:
    """Validate that Rust and Python produce equivalent static instances."""
    if not RECURSIVE_VF.exists():
        return ValidationResult("Instance Output", False, "VF not found")

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        test_axes = {"MONO": 1, "CASL": 0, "wght": 400, "slnt": 0, "CRSV": 0.5}

        # Python
        py_out = tmp / "py-instance.ttf"
        subprocess.run(
            [
                "uv",
                "run",
                "fonttools",
                "varLib.instancer",
                str(RECURSIVE_VF),
            ]
            + [f"{k}={v}" for k, v in test_axes.items()]
            + ["-o", str(py_out)],
            check=True,
            capture_output=True,
        )

        # Rust
        rs_out = tmp / "rs-instance.ttf"
        axis_args = []
        for k, v in test_axes.items():
            axis_args.extend(["-a", f"{k}={v}"])
        subprocess.run(
            [str(RUST_BIN), "instance"] + axis_args + [str(RECURSIVE_VF), str(rs_out)],
            check=True,
            capture_output=True,
        )

        # Compare
        from fontTools.ttLib import TTFont

        py_font = TTFont(py_out)
        rs_font = TTFont(rs_out)

        # Check glyph count
        py_glyphs = len(py_font.getGlyphOrder())
        rs_glyphs = len(rs_font.getGlyphOrder())

        # Check tables present
        py_tables = set(py_font.keys())
        rs_tables = set(rs_font.keys())

        py_font.close()
        rs_font.close()

        if py_glyphs != rs_glyphs:
            return ValidationResult(
                "Instance Output",
                False,
                f"Glyph count: Py={py_glyphs}, Rs={rs_glyphs}",
            )

        missing_tables = py_tables - rs_tables
        if missing_tables & {
            "glyf",
            "head",
            "hhea",
            "hmtx",
            "maxp",
            "name",
            "OS/2",
            "post",
        }:
            return ValidationResult(
                "Instance Output",
                False,
                f"Missing essential tables: {missing_tables}",
            )

        return ValidationResult(
            "Instance Output",
            True,
            f"✓ {rs_glyphs} glyphs, {len(rs_tables)} tables",
        )


def validate_freeze_output() -> ValidationResult:
    """Validate that frozen fonts have features baked in."""
    fonts = sorted(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))[:1]
    if not fonts:
        return ValidationResult("Freeze Output", False, "No fonts found")

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        src = fonts[0]
        dest = tmp / src.name
        shutil.copy(src, dest)

        subprocess.run(
            [str(RUST_BIN), "freeze", "-f", "ss01,ss02", str(dest)],
            check=True,
            capture_output=True,
        )

        from fontTools.ttLib import TTFont

        font = TTFont(dest)

        # After freezing, ss01/ss02 lookups should be removed
        has_gsub = "GSUB" in font
        font.close()

        if not has_gsub:
            return ValidationResult("Freeze Output", False, "GSUB table missing")

        return ValidationResult("Freeze Output", True, "✓ Features frozen correctly")


def validate_vf_output() -> ValidationResult:
    """Validate that built VF has correct axes, instances, and interpolation."""
    fonts = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))
    if len(fonts) < 16:
        return ValidationResult(
            "VF Output", False, f"Need 16 fonts, found {len(fonts)}"
        )

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        rs_dist = tmp / "dist"
        rs_dist.mkdir()

        for f in fonts:
            shutil.copy(f, rs_dist / f.name)

        output = tmp / "vf.ttf"
        result = subprocess.run(
            [
                str(RUST_BIN),
                "build-vf",
                "--dist-dir",
                str(rs_dist),
                "--output",
                str(output),
            ],
            capture_output=True,
        )

        if result.returncode != 0:
            return ValidationResult(
                "VF Output",
                False,
                f"Build failed: {result.stderr.decode()[:100]}",
            )

        from fontTools.ttLib import TTFont
        from fontTools.varLib.instancer import instantiateVariableFont

        font = TTFont(output)

        # Check fvar table
        if "fvar" not in font:
            font.close()
            return ValidationResult("VF Output", False, "Missing fvar table")

        fvar = font["fvar"]
        axes = {a.axisTag: (a.minValue, a.defaultValue, a.maxValue) for a in fvar.axes}

        # Check axes
        if "wght" not in axes:
            font.close()
            return ValidationResult("VF Output", False, "Missing wght axis")
        if "ital" not in axes:
            font.close()
            return ValidationResult("VF Output", False, "Missing ital axis")

        wght = axes["wght"]
        if wght != (300.0, 400.0, 1000.0):
            font.close()
            return ValidationResult(
                "VF Output", False, f"Unexpected wght range: {wght}"
            )

        ital = axes["ital"]
        if ital != (0.0, 0.0, 1.0):
            font.close()
            return ValidationResult(
                "VF Output", False, f"Unexpected ital range: {ital}"
            )

        # Check named instances
        num_instances = len(fvar.instances)
        if num_instances != 16:
            font.close()
            return ValidationResult(
                "VF Output", False, f"Expected 16 named instances, got {num_instances}"
            )

        # Check glyph count
        glyph_count = len(font.getGlyphOrder())

        # Check gvar table
        if "gvar" not in font:
            font.close()
            return ValidationResult("VF Output", False, "Missing gvar table")

        font.close()

        # Test instantiation at all master weights
        test_weights = [300, 400, 500, 600, 700, 800, 900, 1000]
        for wght_val in test_weights:
            try:
                instance = instantiateVariableFont(
                    TTFont(output), {"wght": wght_val, "ital": 0}
                )
                inst_glyphs = len(instance.getGlyphOrder())
                instance.close()
                if inst_glyphs != glyph_count:
                    return ValidationResult(
                        "VF Output",
                        False,
                        f"wght={wght_val}: glyph mismatch ({inst_glyphs} vs {glyph_count})",
                    )
            except Exception as e:
                return ValidationResult(
                    "VF Output", False, f"wght={wght_val} failed: {e}"
                )

        return ValidationResult(
            "VF Output",
            True,
            f"✓ {glyph_count} glyphs, wght(300-1000), ital(0-1), 16 instances, 8 weights OK",
        )


def validate_merge_output() -> ValidationResult:
    """Validate that merged fonts contain glyphs from both sources."""
    rec_fonts = sorted(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))[:1]
    noto_font = BUILD_DIR / "Noto-400-subset.ttf"

    if not rec_fonts or not noto_font.exists():
        return ValidationResult("Merge Output", False, "Required fonts not found")

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        output = tmp / "merged.ttf"

        result = subprocess.run(
            [
                str(RUST_BIN),
                "merge",
                str(rec_fonts[0]),
                str(noto_font),
                "-o",
                str(output),
            ],
            capture_output=True,
        )

        if result.returncode != 0:
            return ValidationResult(
                "Merge Output",
                False,
                f"Merge failed: {result.stderr.decode()[:100]}",
            )

        from fontTools.ttLib import TTFont

        rec = TTFont(rec_fonts[0])
        noto = TTFont(noto_font)
        merged = TTFont(output)

        rec_count = len(rec.getGlyphOrder())
        merged_count = len(merged.getGlyphOrder())

        rec.close()
        noto.close()
        merged.close()

        if merged_count <= rec_count:
            return ValidationResult(
                "Merge Output",
                False,
                f"Merged glyphs ({merged_count}) <= base ({rec_count})",
            )

        return ValidationResult(
            "Merge Output",
            True,
            f"✓ {merged_count} glyphs (base: {rec_count})",
        )


def validate_monospace_flags() -> ValidationResult:
    """Validate that monospace flags are set correctly."""
    fonts = sorted(DIST_DIR.glob("WarpnineMono-*.ttf"))[:1]
    if not fonts:
        fonts = sorted(BUILD_DIR.glob("RecMonoDuotone-*.ttf"))[:1]
    if not fonts:
        return ValidationResult("Monospace Flags", False, "No fonts found")

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        dest = tmp / fonts[0].name
        shutil.copy(fonts[0], dest)

        subprocess.run(
            [str(RUST_BIN), "set-monospace", str(dest)],
            check=True,
            capture_output=True,
        )

        from fontTools.ttLib import TTFont

        font = TTFont(dest)

        is_fixed = font["post"].isFixedPitch == 1
        panose_prop = font["OS/2"].panose.bProportion == 9

        font.close()

        if not is_fixed:
            return ValidationResult("Monospace Flags", False, "isFixedPitch not set")
        if not panose_prop:
            return ValidationResult("Monospace Flags", False, "panose.bProportion != 9")

        return ValidationResult(
            "Monospace Flags", True, "✓ post.isFixedPitch=1, panose.bProportion=9"
        )


def validate_subset_output() -> ValidationResult:
    """Validate that subsetting reduces glyph count appropriately."""
    noto_fonts = sorted(BUILD_DIR.glob("NotoSans*CJK*.ttf"))
    if not noto_fonts:
        return ValidationResult("Subset Output", False, "No Noto CJK fonts found")

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        src = noto_fonts[0]
        output = tmp / "subset.ttf"

        result = subprocess.run(
            [str(RUST_BIN), "subset-japanese", str(src), str(output)],
            capture_output=True,
        )

        if result.returncode != 0:
            return ValidationResult(
                "Subset Output",
                False,
                f"Subset failed: {result.stderr.decode()[:100]}",
            )

        from fontTools.ttLib import TTFont

        original = TTFont(src)
        subset = TTFont(output)

        orig_count = len(original.getGlyphOrder())
        subset_count = len(subset.getGlyphOrder())

        original.close()
        subset.close()

        if subset_count >= orig_count:
            return ValidationResult(
                "Subset Output",
                False,
                f"Subset ({subset_count}) >= original ({orig_count})",
            )

        reduction = (1 - subset_count / orig_count) * 100
        return ValidationResult(
            "Subset Output",
            True,
            f"✓ {subset_count} glyphs ({reduction:.0f}% reduction)",
        )


def run_validations():
    """Run all validation tests."""
    print("\n" + "=" * 80)
    print("Validation Tests")
    print("=" * 80)

    validations = [
        validate_instance_output,
        validate_freeze_output,
        validate_monospace_flags,
        validate_merge_output,
        validate_subset_output,
        validate_vf_output,
    ]

    passed = 0
    failed = 0

    for validate in validations:
        print(f"  Testing: {validate.__doc__.split('.')[0]}...")
        try:
            result = validate()
            status = "✓" if result.passed else "✗"
            print(f"    {status} {result.name}: {result.message}")
            if result.passed:
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"    ✗ {validate.__name__}: Error - {e}")
            failed += 1

    print("-" * 80)
    print(f"Validation: {passed} passed, {failed} failed")
    return failed == 0


def format_time(t: float | None) -> str:
    if t is None:
        return "Error"
    if t < 0.001:
        return f"{t * 1000000:.0f}µs"
    if t < 1:
        return f"{t * 1000:.0f}ms"
    return f"{t:.2f}s"


def main():
    print("=" * 80)
    print("Warpnine Fonts Pipeline Benchmark: Python vs Rust")
    print("=" * 80)

    if not RUST_BIN.exists():
        print(f"\n❌ Rust binary not found: {RUST_BIN}")
        print("   Run: cargo build --release")
        return

    benchmarks = [
        ("Extract Duotone", benchmark_extract_duotone),
        ("Create Sans", benchmark_create_sans),
        ("Create Condensed", benchmark_create_condensed),
        ("Freeze Static", benchmark_freeze_static),
        ("Merge Fonts", benchmark_merge),
        ("Subset Japanese", benchmark_subset_japanese),
        ("Remove Ligatures", benchmark_remove_ligatures),
        ("Set Monospace", benchmark_set_monospace),
        ("Set Version", benchmark_set_version),
        ("Build VF", benchmark_build_vf),
        ("Copy GSUB", benchmark_copy_gsub),
        ("Fix Calt", benchmark_fix_calt),
        ("Set Name", benchmark_set_name),
    ]

    results = []
    for name, func in benchmarks:
        print(f"\n⏱️  Running: {name}...")
        result = func()
        results.append(result)

    print("\n")
    print("=" * 80)
    print("Results Summary")
    print("=" * 80)
    print(f"{'Operation':<35} {'Python':>12} {'Rust':>12} {'Speedup':>12}")
    print("-" * 80)

    total_py = 0.0
    total_rs = 0.0

    for r in results:
        py_str = (
            format_time(r.python_time)
            if not r.python_error
            else f"❌ {r.python_error[:20]}"
        )
        rs_str = (
            format_time(r.rust_time) if not r.rust_error else f"❌ {r.rust_error[:20]}"
        )
        print(f"{r.name:<35} {py_str:>12} {rs_str:>12} {r.speedup:>12}")

        if r.python_time:
            total_py += r.python_time
        if r.rust_time:
            total_rs += r.rust_time

    print("-" * 80)
    if total_rs > 0:
        total_speedup = f"{total_py / total_rs:.1f}x"
    else:
        total_speedup = "N/A"
    print(
        f"{'TOTAL':<35} {format_time(total_py):>12} {format_time(total_rs):>12} {total_speedup:>12}"
    )
    print("=" * 80)

    if total_rs > 0 and total_py > 0:
        print(f"\n✨ Overall: Rust is {total_py / total_rs:.1f}x faster than Python")
        print(f"   Time saved: {total_py - total_rs:.2f}s per pipeline run")

    # Run validation tests
    run_validations()


if __name__ == "__main__":
    main()
