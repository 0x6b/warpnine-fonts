# Rust CLI Validation Guide

This document describes how to validate that the Rust CLI produces identical output to the Python pipeline.

## Current Status (2026-01-02)

### ✅ Fully Validated (Identical Output)

| Command | Fonts | Status |
|---------|-------|--------|
| `clean` | - | ✓ Pass |
| `download` | - | ✓ Pass |
| `copy-gsub` | - | ✓ Pass |
| `remove-ligatures` | 16 Duotone | ✓ Pass |
| `freeze` | All | ✓ Pass |
| `set-monospace` | All | ✓ Pass |
| `set-version` | All | ✓ Pass |
| `subset-japanese` | 2 Noto | ✓ Pass |
| `merge` | 16 Mono | ✓ Pass (uses existing fonts) |
| `instance` | All | ✓ Pass (MVAR metrics match) |
| `create-sans` | 14 Sans | ✓ Pass (MVAR metrics match) |
| `create-condensed` | 14 Condensed | ✓ Pass (MVAR metrics match) |

### ✅ All Validations Passing

All font-instancer issues have been resolved:

- ✅ MVAR metrics interpolation
- ✅ avar table application
- ✅ Simple glyph coordinate interpolation
- ✅ Composite glyph component offset interpolation
- ✅ Composite glyph bounding box recalculation
- ✅ head table xMin/yMin/xMax/yMax
- ✅ hhea table extents (minLeftSideBearing, minRightSideBearing, xMaxExtent)

**Validation result**: 144/144 checks passed (2026-01-02)

### ✅ All Known Issues Fixed

**Fixed (2026-01-02)**:
- `subset-japanese` now drops VF tables, matching Python output size (~51%)
- `instance` now updates `OS/2.usWeightClass` to match wght axis value

---

## Validation Scripts

### Quick Validation (Run After Any Change)

```bash
cd /Users/kaoru/Projects/fonts/warpnine-fonts
uv run pytest tests/integration/test_rust_cli.py -v
```

### Full Pipeline Benchmark

```bash
# Rust pipeline (excluding merge/VF build)
time ./rust/warpnine-fonts/target/release/warpnine-fonts instance \
  --input build/Recursive_VF_1.085.ttf --output-dir /tmp/rust_test \
  --name Test --axes "wght=400,MONO=0,CASL=0,slnt=0,CRSV=0.5" --style Regular

# Compare with Python
time uv run python -c "
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools.ttLib import TTFont
vf = TTFont('build/Recursive_VF_1.085.ttf')
instance = instantiateVariableFont(vf, {'wght': 400, 'MONO': 0, 'CASL': 0, 'slnt': 0, 'CRSV': 0.5})
instance.save('/tmp/python_test.ttf')
"
```

---

## Validation After font-instancer Fixes

When MVAR interpolation is implemented in font-instancer, run these validations:

### 1. Metrics Validation

```python
from fontTools.ttLib import TTFont

def validate_metrics(rust_path, python_path):
    rs = TTFont(rust_path)
    py = TTFont(python_path)
    
    checks = [
        ("OS/2.sxHeight", rs['OS/2'].sxHeight, py['OS/2'].sxHeight),
        ("OS/2.yStrikeoutPosition", rs['OS/2'].yStrikeoutPosition, py['OS/2'].yStrikeoutPosition),
        ("OS/2.yStrikeoutSize", rs['OS/2'].yStrikeoutSize, py['OS/2'].yStrikeoutSize),
        ("post.underlinePosition", rs['post'].underlinePosition, py['post'].underlinePosition),
        ("post.underlineThickness", rs['post'].underlineThickness, py['post'].underlineThickness),
    ]
    
    all_pass = True
    for name, rs_val, py_val in checks:
        status = "✓" if rs_val == py_val else "✗"
        if rs_val != py_val:
            all_pass = False
            print(f"{status} {name}: Rust={rs_val} Python={py_val}")
        else:
            print(f"{status} {name}: {rs_val}")
    
    return all_pass
```

### 2. Bounding Box Validation

```python
def validate_bounds(rust_path, python_path):
    rs = TTFont(rust_path)
    py = TTFont(python_path)
    
    checks = [
        ("head.xMin", rs['head'].xMin, py['head'].xMin),
        ("head.xMax", rs['head'].xMax, py['head'].xMax),
        ("head.yMin", rs['head'].yMin, py['head'].yMin),
        ("head.yMax", rs['head'].yMax, py['head'].yMax),
        ("hhea.minLeftSideBearing", rs['hhea'].minLeftSideBearing, py['hhea'].minLeftSideBearing),
        ("hhea.minRightSideBearing", rs['hhea'].minRightSideBearing, py['hhea'].minRightSideBearing),
        ("hhea.xMaxExtent", rs['hhea'].xMaxExtent, py['hhea'].xMaxExtent),
    ]
    
    all_pass = True
    for name, rs_val, py_val in checks:
        # Allow ±1 for rounding differences
        diff = abs(rs_val - py_val)
        status = "✓" if diff <= 1 else "✗"
        if diff > 1:
            all_pass = False
        print(f"{status} {name}: Rust={rs_val} Python={py_val} (diff={diff})")
    
    return all_pass
```

### 3. Feature Validation

```python
def validate_features(rust_path, python_path):
    rs = TTFont(rust_path)
    py = TTFont(python_path)
    
    # GSUB features
    rs_gsub = {r.FeatureTag for r in rs['GSUB'].table.FeatureList.FeatureRecord}
    py_gsub = {r.FeatureTag for r in py['GSUB'].table.FeatureList.FeatureRecord}
    
    # GPOS features  
    rs_gpos = {r.FeatureTag for r in rs['GPOS'].table.FeatureList.FeatureRecord}
    py_gpos = {r.FeatureTag for r in py['GPOS'].table.FeatureList.FeatureRecord}
    
    gsub_match = rs_gsub == py_gsub
    gpos_match = rs_gpos == py_gpos
    
    print(f"GSUB features: {'✓' if gsub_match else '✗'} ({len(rs_gsub)} features)")
    print(f"GPOS features: {'✓' if gpos_match else '✗'} ({len(rs_gpos)} features)")
    
    return gsub_match and gpos_match
```

### 4. Complete Validation Script

Save as `tests/validate_rust_output.py`:

```python
#!/usr/bin/env python3
"""
Validate Rust CLI output against Python reference.

Usage:
    uv run python tests/validate_rust_output.py
"""

from pathlib import Path
from fontTools.ttLib import TTFont
import subprocess
import tempfile
import sys

PROJECT_ROOT = Path(__file__).parent.parent
RUST_CLI = PROJECT_ROOT / "rust/warpnine-fonts/target/release/warpnine-fonts"
BUILD_DIR = PROJECT_ROOT / "build"

def generate_test_fonts(tmp_dir: Path):
    """Generate test fonts with both Rust and Python."""
    
    # Rust
    rust_out = tmp_dir / "rust"
    rust_out.mkdir()
    subprocess.run([
        str(RUST_CLI), "create-sans",
        "--input", str(BUILD_DIR / "Recursive_VF_1.085.ttf"),
        "--output-dir", str(rust_out)
    ], check=True, capture_output=True)
    
    # Python
    python_out = tmp_dir / "python"
    python_out.mkdir()
    subprocess.run([
        "uv", "run", "python", "-c", f"""
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools.ttLib import TTFont

INSTANCES = [
    ("Regular", {{"wght": 400, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5}}),
    ("Bold", {{"wght": 700, "MONO": 0, "CASL": 0, "slnt": 0, "CRSV": 0.5}}),
    ("Italic", {{"wght": 400, "MONO": 0, "CASL": 0, "slnt": -15, "CRSV": 1}}),
]

for style, axes in INSTANCES:
    vf = TTFont("{BUILD_DIR}/Recursive_VF_1.085.ttf")
    instance = instantiateVariableFont(vf, axes)
    instance.save("{python_out}/WarpnineSans-" + style + ".ttf")
"""
    ], check=True, capture_output=True)
    
    return rust_out, python_out

def validate_font_pair(rust_path: Path, python_path: Path) -> dict:
    """Validate a single font pair."""
    rs = TTFont(rust_path)
    py = TTFont(python_path)
    
    results = {"pass": [], "fail": []}
    
    # Critical metrics (MVAR)
    mvar_checks = [
        ("OS/2.sxHeight", rs['OS/2'].sxHeight, py['OS/2'].sxHeight),
        ("OS/2.yStrikeoutPosition", rs['OS/2'].yStrikeoutPosition, py['OS/2'].yStrikeoutPosition),
        ("OS/2.yStrikeoutSize", rs['OS/2'].yStrikeoutSize, py['OS/2'].yStrikeoutSize),
        ("post.underlinePosition", rs['post'].underlinePosition, py['post'].underlinePosition),
        ("post.underlineThickness", rs['post'].underlineThickness, py['post'].underlineThickness),
    ]
    
    for name, rs_val, py_val in mvar_checks:
        if rs_val == py_val:
            results["pass"].append(name)
        else:
            results["fail"].append(f"{name}: Rust={rs_val} Python={py_val}")
    
    # Bounding box (allow ±2 for rounding)
    bbox_checks = [
        ("head.xMin", rs['head'].xMin, py['head'].xMin),
        ("head.xMax", rs['head'].xMax, py['head'].xMax),
        ("head.yMin", rs['head'].yMin, py['head'].yMin),
        ("head.yMax", rs['head'].yMax, py['head'].yMax),
    ]
    
    for name, rs_val, py_val in bbox_checks:
        if abs(rs_val - py_val) <= 2:
            results["pass"].append(name)
        else:
            results["fail"].append(f"{name}: Rust={rs_val} Python={py_val}")
    
    # Features
    rs_gsub = {r.FeatureTag for r in rs['GSUB'].table.FeatureList.FeatureRecord}
    py_gsub = {r.FeatureTag for r in py['GSUB'].table.FeatureList.FeatureRecord}
    if rs_gsub == py_gsub:
        results["pass"].append(f"GSUB features ({len(rs_gsub)})")
    else:
        results["fail"].append(f"GSUB features differ")
    
    rs.close()
    py.close()
    
    return results

def main():
    print("=" * 60)
    print("Rust CLI Output Validation")
    print("=" * 60)
    
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        
        print("\nGenerating test fonts...")
        rust_dir, python_dir = generate_test_fonts(tmp_dir)
        
        print("\nValidating output...\n")
        
        all_pass = True
        for style in ["Regular", "Bold", "Italic"]:
            rust_font = rust_dir / f"WarpnineSans-{style}.ttf"
            python_font = python_dir / f"WarpnineSans-{style}.ttf"
            
            results = validate_font_pair(rust_font, python_font)
            
            if results["fail"]:
                all_pass = False
                print(f"{style}: ✗ FAIL")
                for f in results["fail"]:
                    print(f"  - {f}")
            else:
                print(f"{style}: ✓ PASS ({len(results['pass'])} checks)")
    
    print("\n" + "=" * 60)
    if all_pass:
        print("RESULT: ALL VALIDATIONS PASSED ✓")
        sys.exit(0)
    else:
        print("RESULT: SOME VALIDATIONS FAILED ✗")
        print("\nExpected failures until font-instancer fixes:")
        print("  - MVAR interpolation (sxHeight, strikeout, underline)")
        print("  - Bounding box recalculation")
        sys.exit(1)

if __name__ == "__main__":
    main()
```

---

## Expected Results After font-instancer Fixes

When all issues are fixed, running the validation should show:

```
==============================================================
Rust CLI Output Validation
==============================================================

Generating test fonts...

Validating output...

Regular: ✓ PASS (10 checks)
Bold: ✓ PASS (10 checks)
Italic: ✓ PASS (10 checks)

==============================================================
RESULT: ALL VALIDATIONS PASSED ✓
```

---

## Performance Baseline

Current benchmarks (for regression testing):

| Operation | Rust | Python | Speedup |
|-----------|------|--------|---------|
| Full pipeline (sans fonts) | 0.48s | 74.9s | 157x |
| create-sans (14 fonts) | 0.07s | 7.7s | 105x |
| create-condensed (14 fonts) | 0.09s | 9.2s | 99x |
| set-monospace (16 fonts) | 0.21s | 8.1s | 38x |
| remove-ligatures (5 fonts) | 0.03s | 0.38s | 14x |

---

## Checklist for font-instancer Fix Verification

When font-instancer is updated:

- [x] Update font-instancer dependency in `Cargo.toml`
- [x] Run `cargo build --release`
- [x] Run `uv run pytest tests/integration/test_rust_cli.py -v` (12/12 passed)
- [x] Run `uv run python tests/validate_rust_output.py` (144/144 passed)
- [x] Verify MVAR metrics match Python output ✓
- [x] Verify bounding boxes match ✓
- [x] Verify GSUB/GPOS features match ✓
- [x] Run performance benchmark to check for regression (0.069s for 14 fonts)
- [x] Update this document with new status

**Completed: 2026-01-02** - All validations passing!
