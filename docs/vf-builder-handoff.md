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
| Rust (with IUP) | 127 MB | Basic IUP only |
| Rust (no IUP) | 130 MB | All deltas required |

**Gap: ~42% larger than fontTools**

### Validation Results

- ✅ VF has correct fvar axes (wght 300-1000, ital 0-1)
- ✅ VF has 16 named instances
- ✅ VF has gvar with 51,103 glyph variations
- ✅ Instancing at default location matches original master exactly
- ✅ Instancing at other locations produces correct interpolation
- ⚠️ fontTools cannot parse the gvar table (compatibility issue)
- ✅ read-fonts/skrifa can parse and use the VF correctly

## Usage

```bash
# Build WarpnineMono variable font
warpnine-fonts build-vf --dist-dir dist --output dist/WarpnineMono-VF.ttf

# The static masters must exist in dist/:
# - WarpnineMono-{Light,Regular,Medium,SemiBold,Bold,ExtraBold,Black,ExtraBlack}.ttf
# - WarpnineMono-{Light,Regular,Medium,SemiBold,Bold,ExtraBold,Black,ExtraBlack}Italic.ttf
```

## Missing Optimizations

Based on analysis of fontTools source code, these optimizations are missing:

### 1. Advanced IUP Optimization (Highest Impact)

**fontTools location**: `Lib/fontTools/varLib/iup.py`

fontTools uses a sophisticated Dynamic Programming algorithm:
- `_iup_contour_optimize_dp()` finds the truly minimal set of required points
- `_iup_contour_bound_forced_set()` pre-computes points that cannot be interpolated
- Achieves 30-60% reduction in explicit delta points

**Current Rust implementation**: Uses `write_fonts::tables::gvar::iup::iup_delta_optimize` which may be less aggressive.

**Path to improvement**:
- Study write-fonts IUP implementation vs fontTools
- Ensure tolerance parameter (0.5) matches fontTools default
- Verify phantom point handling is correct

### 2. Shared Point Numbers (High Impact)

**fontTools location**: `Lib/fontTools/ttLib/tables/TupleVariation.py` lines 709-781

When multiple tuple variations affect the same set of points, fontTools:
- Finds the most common point set
- Encodes it once in a shared location
- Sets `TUPLES_SHARE_POINT_NUMBERS` flag (0x8000)
- Other variations reference the shared set

**Current Rust implementation**: Each variation encodes its own point numbers.

**Path to improvement**:
```rust
// In GlyphDeltas or GlyphVariations, analyze point sets across all variations
// for a glyph and identify the most commonly used set
fn compute_shared_points(variations: &[GlyphDeltas]) -> Option<PackedPointNumbers> {
    // Count frequency of each point set
    // Return the most common one if it appears more than once
}
```

### 3. Shared Tuple Coordinates (Medium Impact)

**fontTools location**: `Lib/fontTools/ttLib/tables/TupleVariation.py` lines 693-706

Peak tuple coordinates that appear in multiple variations are stored once in a shared table.

**Current Rust implementation**: `write_fonts::tables::gvar::Gvar::new()` already computes shared tuples via `compute_shared_peak_tuples()`. This should be working.

**Verification needed**: Confirm shared tuples are being used effectively.

### 4. Delta Run Encoding (Low-Medium Impact)

**fontTools location**: `Lib/fontTools/ttLib/tables/TupleVariation.py` lines 327-526

Intelligently chooses encoding format:
- Zero runs: Single byte header, no data
- Byte runs: Values -128 to 127 in 1 byte each
- Word runs: Values -32768 to 32767 in 2 bytes each

**Current Rust implementation**: `write_fonts` handles this internally. Should be comparable.

### 5. Point Number Delta Encoding (Low Impact)

Point indices are delta-encoded (store differences, not absolute values).

**Current Rust implementation**: `write_fonts` handles this via `PackedPointNumbers`.

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

## Next Steps (Priority Order)

1. **Investigate IUP effectiveness**
   - Compare number of required vs optional deltas
   - Check if tolerance parameter needs adjustment
   - Verify phantom point handling

2. **Implement shared point numbers**
   - Analyze point sets across variations
   - Find most common set per glyph
   - Encode once, reference by flag

3. **Verify shared tuples**
   - Confirm write-fonts is computing shared tuples
   - Check if our tuple count matches fontTools

4. **Fix fontTools compatibility**
   - Investigate why fontTools can't parse our gvar
   - May be a write-fonts encoding quirk

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
