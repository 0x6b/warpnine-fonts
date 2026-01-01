#!/usr/bin/env python3
"""
Benchmark comparison between Python and Rust font tools.

Usage:
    uv run benchmark.py
"""

import shutil
import subprocess
import tempfile
import time
from pathlib import Path

BUILD_DIR = Path(__file__).parent / "build"
RUST_BIN = Path(__file__).parent / "rust/warpnine-fonts/target/release/warpnine-fonts"

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


def run_timed(name: str, func) -> float:
    """Run a function and return elapsed time in seconds."""
    start = time.perf_counter()
    func()
    elapsed = time.perf_counter() - start
    return elapsed


def benchmark_freeze_python(font_path: Path, features: list[str]) -> float:
    """Benchmark Python pyftfeatfreeze."""
    with tempfile.NamedTemporaryFile(suffix=".ttf", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        shutil.copy(font_path, tmp_path)
        feature_str = ",".join(features)

        def run():
            subprocess.run(
                [
                    "uv",
                    "run",
                    "pyftfeatfreeze",
                    "-f",
                    feature_str,
                    str(tmp_path),
                    str(tmp_path),
                ],
                check=True,
                capture_output=True,
            )

        return run_timed("python freeze", run)
    finally:
        tmp_path.unlink(missing_ok=True)


def benchmark_freeze_rust(font_path: Path, features: list[str]) -> float:
    """Benchmark Rust font-feature-freezer."""
    with tempfile.NamedTemporaryFile(suffix=".ttf", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        shutil.copy(font_path, tmp_path)
        feature_str = ",".join(features)

        def run():
            subprocess.run(
                [str(RUST_BIN), "freeze", "-f", feature_str, str(tmp_path)],
                check=True,
                capture_output=True,
            )

        return run_timed("rust freeze", run)
    finally:
        tmp_path.unlink(missing_ok=True)


def benchmark_instance_python(vf_path: Path, axes: dict[str, float]) -> float:
    """Benchmark Python fonttools instancer."""
    with tempfile.NamedTemporaryFile(suffix=".ttf", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        axis_args = [f"{k}={v}" for k, v in axes.items()]

        def run():
            # axis args must come before -o flag
            subprocess.run(
                ["uv", "run", "fonttools", "varLib.instancer", str(vf_path)]
                + axis_args
                + ["-o", str(tmp_path)],
                check=True,
                capture_output=True,
            )

        return run_timed("python instance", run)
    finally:
        tmp_path.unlink(missing_ok=True)


def benchmark_instance_rust(vf_path: Path, axes: dict[str, float]) -> float:
    """Benchmark Rust font-instancer."""
    with tempfile.NamedTemporaryFile(suffix=".ttf", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        axis_args = []
        for k, v in axes.items():
            axis_args.extend(["-a", f"{k}={v}"])

        def run():
            subprocess.run(
                [str(RUST_BIN), "instance"] + axis_args + [str(vf_path), str(tmp_path)],
                check=True,
                capture_output=True,
            )

        return run_timed("rust instance", run)
    finally:
        tmp_path.unlink(missing_ok=True)


def benchmark_merge_python(font1: Path, font2: Path) -> float:
    """Benchmark Python fonttools merge."""
    with tempfile.NamedTemporaryFile(suffix=".ttf", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        code = f'''
from fontTools.merge import Merger
merger = Merger()
fonts = merger.merge(["{font1}", "{font2}"])
fonts.save("{tmp_path}")
'''

        def run():
            subprocess.run(
                ["uv", "run", "python", "-c", code],
                check=True,
                capture_output=True,
            )

        return run_timed("python merge", run)
    finally:
        tmp_path.unlink(missing_ok=True)


def benchmark_merge_rust(font1: Path, font2: Path) -> float:
    """Benchmark Rust font-merger."""
    with tempfile.NamedTemporaryFile(suffix=".ttf", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:

        def run():
            subprocess.run(
                [str(RUST_BIN), "merge", str(font1), str(font2), "-o", str(tmp_path)],
                check=True,
                capture_output=True,
            )

        return run_timed("rust merge", run)
    finally:
        tmp_path.unlink(missing_ok=True)


def format_speedup(python_time: float, rust_time: float) -> str:
    """Format speedup ratio."""
    if rust_time > 0:
        speedup = python_time / rust_time
        return f"{speedup:.1f}x faster"
    return "N/A"


def main():
    print("=" * 70)
    print("Font Tool Benchmark: Python vs Rust")
    print("=" * 70)

    # Check if Rust binary exists
    if not RUST_BIN.exists():
        print(f"Rust binary not found at {RUST_BIN}")
        print("Run: cd rust/warpnine-fonts && cargo build --release")
        return

    # Benchmark freeze
    rec_regular = BUILD_DIR / "RecMonoDuotone-Regular.ttf"
    if rec_regular.exists():
        print("\n## Feature Freeze (RecMonoDuotone-Regular.ttf)")
        print(
            f"   Features: {','.join(MONO_FEATURES[:5])}... ({len(MONO_FEATURES)} total)"
        )

        try:
            py_time = benchmark_freeze_python(rec_regular, MONO_FEATURES)
            print(f"   Python (pyftfeatfreeze): {py_time:.3f}s")
        except Exception as e:
            print(f"   Python (pyftfeatfreeze): Error - {e}")
            py_time = None

        try:
            rs_time = benchmark_freeze_rust(rec_regular, MONO_FEATURES)
            print(f"   Rust (font-feature-freezer): {rs_time:.3f}s")
        except Exception as e:
            print(f"   Rust (font-feature-freezer): Error - {e}")
            rs_time = None

        if py_time and rs_time:
            print(f"   Speedup: {format_speedup(py_time, rs_time)}")
    else:
        print(f"\nSkipping freeze benchmark - {rec_regular} not found")

    # Benchmark instancer
    # Use Recursive VF (TrueType outlines) instead of Noto CJK (CFF2)
    recursive_vf = (
        Path(__file__).parent
        / "recursive/fonts/ArrowType-Recursive-1.085/Recursive_Desktop/Recursive_VF_1.085.ttf"
    )
    if recursive_vf.exists():
        print(f"\n## Variable Font Instancing ({recursive_vf.name})")
        print("   Axes: wght=700")

        try:
            py_time = benchmark_instance_python(recursive_vf, {"wght": 700})
            print(f"   Python (fonttools instancer): {py_time:.3f}s")
        except Exception as e:
            print(f"   Python (fonttools instancer): Error - {e}")
            py_time = None

        try:
            rs_time = benchmark_instance_rust(recursive_vf, {"wght": 700})
            print(f"   Rust (font-instancer): {rs_time:.3f}s")
        except Exception as e:
            print(f"   Rust (font-instancer): Error - {e}")
            rs_time = None

        if py_time and rs_time:
            print(f"   Speedup: {format_speedup(py_time, rs_time)}")
    else:
        print(f"\nSkipping instance benchmark - {recursive_vf} not found")

    # Benchmark merge (fontTools merge has issues with VarStore tables, so skip Python comparison)
    font1 = BUILD_DIR / "RecMonoDuotone-Regular.ttf"
    font2 = BUILD_DIR / "Noto-400-subset.ttf"
    if font1.exists() and font2.exists():
        print(f"\n## Font Merge ({font1.name} + {font2.name})")
        print("   Note: Python fonttools merge fails with VarStore tables")

        try:
            py_time = benchmark_merge_python(font1, font2)
            print(f"   Python (fonttools merge): {py_time:.3f}s")
        except Exception as e:
            print(f"   Python (fonttools merge): Error - {e}")
            py_time = None

        try:
            rs_time = benchmark_merge_rust(font1, font2)
            print(f"   Rust (font-merger): {rs_time:.3f}s")
        except Exception as e:
            print(f"   Rust (font-merger): Error - {e}")
            rs_time = None

        if py_time and rs_time:
            print(f"   Speedup: {format_speedup(py_time, rs_time)}")
    else:
        print(f"\nSkipping merge benchmark - fonts not found")

    print("\n" + "=" * 70)


if __name__ == "__main__":
    main()
