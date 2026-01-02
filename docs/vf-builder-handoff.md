# Variable Font Builder - Implementation Handoff

## Summary

We implemented a pure Rust variable font builder (`warpnine-font-vf-builder`) to replace Python's `fontTools.varLib`. The builder works correctly but produces larger files than fontTools.

## Current State

### What's Implemented

| Component | Status | Location |
|-----------|--------|----------|
| Designspace model | ✅ Complete | `crates/vf-builder/src/designspace.rs` |
| Variation model | ✅ Complete | `crates/vf-builder/src/variation_model.rs` |
| fvar table builder | ✅ Complete | `crates/vf-builder/src/vf_builder.rs` |
| gvar table builder | ✅ Complete | `crates/vf-builder/src/vf_builder.rs` |
| IUP optimization | ⚠️ Basic | Uses `write_fonts::tables::gvar::iup::iup_delta_optimize` |
| CLI command | ✅ Complete | `crates/cli/src/build_vf.rs` |

### File Size Comparison

| Builder | Output Size | Notes |
|---------|-------------|-------|
| **Python (fontTools)** | **74 MB** | Full optimizations |
| Rust (current) | **83 MB** | With tent fix, IUP forced required |

**Gap: ~12% larger than fontTools** (down from 42%)

### Validation Results

- ✅ VF has correct fvar axes (wght 300-1000, ital 0-1)
- ✅ VF has 16 named instances
- ✅ VF has gvar with 51,103 glyph variations
- ✅ Instancing at default location matches original master exactly
- ✅ Instancing at other locations produces correct interpolation (±2 units)
- ✅ fontTools can parse the gvar table
- ✅ read-fonts/skrifa can parse and use the VF correctly

## Usage

```bash
# Build WarpnineMono variable font
warpnine-fonts build-vf --dist-dir dist --output dist/WarpnineMono-VF.ttf

# The static masters must exist in dist/:
# - WarpnineMono-{Light,Regular,Medium,SemiBold,Bold,ExtraBold,Black,ExtraBlack}.ttf
# - WarpnineMono-{Light,Regular,Medium,SemiBold,Bold,ExtraBold,Black,ExtraBlack}Italic.ttf
```

## Current Issues

### 1. IUP Sparse Optimization Disabled (write-fonts bug)

**Issue**: write-fonts has a bug where shared point numbers with sparse deltas causes
a mismatch between point count and delta count in the encoded gvar data. fontTools
cannot parse such files.

**Current workaround**: All deltas are forced to `required=true` even if IUP
optimization identifies them as interpolatable. This wastes space but ensures
fontTools compatibility.

**Impact**: ~9 MB larger gvar (83 MB vs 74 MB for fontTools)

**Path to fix**: Report bug to googlefonts/fontations with reproduction case, then
remove the workaround once fixed.

### 2. Shared Point Numbers (Blocked by #1)

write-fonts supports shared point numbers via `compute_shared_points()`, but this
cannot be used effectively until the sparse delta bug is fixed.

### 3. Shared Tuple Coordinates (Working)

`write_fonts::tables::gvar::Gvar::new()` computes shared tuples via
`compute_shared_peak_tuples()`. This is working correctly (15 shared tuples for
our 16-master VF).

## Architecture

```
crates/vf-builder/
├── Cargo.toml
└── src/
    ├── lib.rs              # Public API exports
    ├── designspace.rs      # Axis, Source, Instance, DesignSpace structs
    ├── variation_model.rs  # Region, VariationModel for delta computation
    ├── vf_builder.rs       # Main build_variable_font() function
    └── error.rs            # Error types

crates/cli/src/
└── build_vf.rs            # CLI command with WarpnineMono-specific config
```

## Key Functions

### `build_variable_font(designspace: &DesignSpace) -> Result<Vec<u8>>`

Main entry point. Steps:
1. Load all master fonts
2. Verify glyph compatibility (same glyph count)
3. Build VariationModel from designspace
4. Build gvar table with deltas for each glyph
5. Build fvar table with axes and instances
6. Copy glyf/loca from default master
7. Copy other tables (cmap, name, OS/2, etc.)

### `build_simple_glyph_variations(...)`

For simple (contour-based) glyphs:
1. Collect points from all masters
2. For each region (non-default master):
   - Compute raw deltas using variation model
   - Add phantom points (4 extra)
   - Apply IUP optimization
   - Create GlyphDeltas with Tent coordinates

### `build_composite_glyph_variations(...)`

For composite glyphs:
1. Collect component offsets from all masters
2. Compute deltas for each component offset
3. No IUP optimization (composite glyphs don't have contours)

## Completed Fixes

1. **Phantom point handling** - Fixed: phantom points are now included in gvar deltas
2. **Tent computation** - Fixed: intermediate masters now have proper (min, peak, max) regions
3. **fontTools compatibility** - Fixed: all glyphs parse correctly
4. **IUP optimization** - Working (75.8% optional) but disabled for compatibility

## Next Steps (Priority Order)

1. **File write-fonts bug report**
   - Reproduce the shared point numbers + sparse deltas issue
   - Submit to googlefonts/fontations

2. **Remove IUP workaround** (after write-fonts fix)
   - Remove `GlyphDelta::required()` forcing
   - Should reduce gvar by ~10 MB

3. **Investigate remaining size gap**
   - Current: 83 MB vs fontTools 74 MB
   - After IUP fix: should be closer to fontTools

## Testing

```bash
# Run unit tests
cargo test --package warpnine-font-vf-builder

# Build VF and test instancing
cargo run --release -p warpnine-fonts-cli -- build-vf --output dist/WarpnineMono-VF.ttf
cargo run --release -p warpnine-fonts-cli -- instance --axis wght=700 dist/WarpnineMono-VF.ttf /tmp/Bold.ttf

# Compare with original master
python3 -c "
from fontTools.ttLib import TTFont
inst = TTFont('/tmp/Bold.ttf')
orig = TTFont('dist/WarpnineMono-Bold.ttf')
# Compare glyph coordinates...
"
```

## References

- [OpenType gvar spec](https://learn.microsoft.com/en-us/typography/opentype/spec/gvar)
- [fontTools varLib source](https://github.com/fonttools/fonttools/tree/main/Lib/fontTools/varLib)
- [fontTools IUP optimization](https://github.com/fonttools/fonttools/blob/main/Lib/fontTools/varLib/iup.py)
- [write-fonts gvar module](https://docs.rs/write-fonts/latest/write_fonts/tables/gvar/)

## Commits

1. `feat(vf-builder): add pure Rust variable font builder crate` - Initial implementation
2. `feat(cli): add build-vf command for building variable fonts` - CLI integration
3. `feat(vf-builder): add IUP optimization for gvar deltas` - Basic IUP support
