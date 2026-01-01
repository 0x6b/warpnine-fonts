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
RUST_BIN = Path(__file__).parent / "rust/warpnine-fonts/target/release/warpnine-fonts"
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

        # Rust: use warpnine-fonts instance
        def run_rust():
            for style, axes in DUOTONE_INSTANCES:
                output = tmp / f"rs-{style}.ttf"
                axis_args = []
                for k, v in axes.items():
                    axis_args.extend(["-a", f"{k}={v}"])
                subprocess.run(
                    [str(RUST_BIN), "instance"]
                    + axis_args
                    + [str(RECURSIVE_VF), str(output)],
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
            for rec in rec_fonts:
                output = tmp / f"rs-{rec.stem}-merged.ttf"
                subprocess.run(
                    [
                        str(RUST_BIN),
                        "merge",
                        str(rec),
                        str(noto_font),
                        "-o",
                        str(output),
                    ],
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
        print("   Run: cd rust/warpnine-fonts && cargo build --release")
        return

    benchmarks = [
        ("Extract Duotone", benchmark_extract_duotone),
        ("Freeze Static", benchmark_freeze_static),
        ("Merge Fonts", benchmark_merge),
        ("Set Monospace", benchmark_set_monospace),
        ("Set Version", benchmark_set_version),
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


if __name__ == "__main__":
    main()
