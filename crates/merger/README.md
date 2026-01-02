# font-merger

A Rust port of [fontTools merge.Merger](https://github.com/fonttools/fonttools/tree/main/Lib/fontTools/merge) for combining multiple fonts into one unified font file.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Merge two fonts
font-merger font1.ttf font2.ttf -o merged.ttf

# Merge multiple fonts
font-merger latin.ttf cjk.ttf arabic.ttf -o combined.ttf

# Drop specific tables
font-merger font1.ttf font2.ttf --drop-tables DSIG,GDEF -o merged.ttf

# Verbose output
font-merger font1.ttf font2.ttf -o merged.ttf -v
```

## Library Usage

```rust
use font_merger::{Merger, Options};

let font1 = std::fs::read("font1.ttf")?;
let font2 = std::fs::read("font2.ttf")?;

let merger = Merger::new(Options::default());
let merged = merger.merge(&[&font1, &font2])?;

std::fs::write("merged.ttf", merged)?;
```

## Implementation Status

This is a Rust port of the fontTools `merge.Merger` module. The following tables are fully supported:

### Fully Implemented

| Table  | Description                                                   |
| ------ | ------------------------------------------------------------- |
| `head` | Font header with proper flag merging (bitwise OR/AND per bit) |
| `hhea` | Horizontal header (max ascender, min descender, max line gap) |
| `vhea` | Vertical header (same strategy as hhea)                       |
| `maxp` | Maximum profile (sum glyph counts, max other values)          |
| `hmtx` | Horizontal metrics (per-glyph metrics preserved)              |
| `vmtx` | Vertical metrics (per-glyph metrics preserved)                |
| `OS/2` | OS/2 table with proper version handling and flag merging      |
| `post` | PostScript names                                              |
| `name` | Naming table (copied from first font)                         |
| `cmap` | Character map with duplicate glyph detection                  |
| `glyf` | TrueType outlines with composite glyph component remapping    |
| `loca` | Index to location (generated from glyf)                       |
| `GSUB` | Glyph substitution with full lookup remapping                 |
| `GPOS` | Glyph positioning with full lookup remapping                  |

### Partial Implementation

| Table               | Status                                        |
| ------------------- | --------------------------------------------- |
| `CFF`               | Copies from first font only (see Limitations) |
| `CFF2`              | Copies from first font only                   |
| `fpgm`/`prep`/`cvt` | Hinting copied from first font only           |

### Merge Strategies

The merger applies table-specific strategies matching fontTools behavior:

- head flags: Bitwise merge (OR for most bits, AND for style bits)
- head macStyle: Bitwise merge (AND for bold/italic, OR for decorative)
- head bounds: min for x_min/y_min, max for x_max/y_max
- hhea/vhea: max for ascender/line_gap, min for descender
- OS/2 version: Uses maximum version from all fonts
- OS/2 Unicode ranges: Bitwise OR (combined coverage)
- OS/2 code page ranges: Bitwise OR
- OS/2 fsSelection: AND for style flags, OR for decorative flags
- OS/2 char ranges: min for first, max for last

### Layout Table Support

GSUB lookups remapped:

- Type 1: Single substitution (Format 1 & 2)
- Type 2: Multiple substitution
- Type 3: Alternate substitution
- Type 4: Ligature substitution
- Type 5: Contextual substitution (all formats)
- Type 6: Chained contextual substitution (all formats)
- Type 8: Reverse chain single substitution

GPOS lookups remapped:

- Type 1: Single adjustment
- Type 2: Pair adjustment (Format 1 & 2)
- Type 3: Cursive attachment
- Type 4: Mark-to-base attachment
- Type 5: Mark-to-ligature attachment
- Type 6: Mark-to-mark attachment
- Type 7: Contextual positioning (all formats)
- Type 8: Chained contextual positioning (all formats)

Extension lookups (Type 7 for GSUB, Type 9 for GPOS) are not yet remapped.

## Limitations

### CFF Outlines

CFF (Compact Font Format) merging is not yet implemented. When merging CFF fonts:

- Only the first font's CFF table is included
- Glyphs from subsequent fonts are not merged
- A warning is logged when this occurs

A proper implementation would require:

1. Parsing CFF charstrings from all fonts
2. Desubroutinizing (inlining all subroutine calls)
3. Reordering charstrings to match the merged glyph order
4. Rebuilding the CFF table with combined data

The `write-fonts` crate does not yet support CFF table construction.

Workaround: Convert CFF fonts to TrueType outlines before merging.

### TrueType Hinting

When merging fonts with TrueType hints:

- Global hint programs (`fpgm`, `prep`, `cvt`) are copied from the first font only
- Per-glyph instructions in `glyf` are preserved from each font
- Glyphs from fonts 2+ may not hint correctly if they rely on different `fpgm` functions or `cvt` values

A warning is logged if fonts have different hinting programs.

### Other Limitations

- All fonts must have the same `unitsPerEm`
- CID-keyed CFF fonts are not supported
- Extension lookups (GSUB type 7, GPOS type 9) are not remapped
- BASE table merging is incomplete (same as fontTools)
- FeatureParams nameIDs are not remapped in layout tables

## How It Works

1. Glyph Order Consolidation: Creates a unified glyph order from all input fonts, disambiguating duplicate names (e.g., `A`, `A.1`, `A.2`)
2. cmap Merging: Combines Unicode-to-glyph mappings, detecting when the same codepoint maps to different glyphs across fonts
3. Table Merging: Applies table-specific strategies for each table type
4. Outline Merging: For TrueType fonts, copies glyph outlines and remaps component references in composite glyphs
5. Layout Merging: Combines GSUB/GPOS tables, remapping glyph IDs in coverage tables, class definitions, and lookup subtables. Adds synthetic `locl` lookups for duplicate glyphs.
6. Other Tables: Copies remaining tables from the first font

## Testing

```bash
cargo test
```

Tests are based on patterns from the fontTools test suite.

## Differences from fontTools

This Rust port aims to match fontTools `merge.Merger` behavior but has some differences.

### Fontations Crate Limitations

These limitations are due to missing functionality in the `write-fonts` crate:

| Feature                | Status        | Notes                                                              |
| ---------------------- | ------------- | ------------------------------------------------------------------ |
| CFF table construction | Not supported | `write-fonts` has no CFF module; we copy raw bytes from first font |
| cmap format-14 (UVS)   | Not supported | `write-fonts` only supports format-4 and format-12                 |

### Implementation Differences

These could be implemented but are not yet:

| Aspect                      | fontTools                                                 | Rust Port                      |
| --------------------------- | --------------------------------------------------------- | ------------------------------ |
| OS/2.fsType                 | Complex "least restrictive" merge with `mergeOs2FsType()` | Takes from first font only     |
| Extension lookups           | Delegates to wrapped subtable                             | Not remapped                   |
| Default Ignorable filtering | Skips U+00AD, U+25CC etc. for duplicate detection         | Not implemented                |
| Lookup/Feature pruning      | Post-merge removes unused features/lookups                | All features/lookups retained  |
| GDEF MarkFilteringSet       | Remapped during layout merge                              | Not handled                    |
| Feature merging             | Same tag features combined, lookups deduplicated          | Features concatenated per-font |

### API Differences

| Aspect         | fontTools                          | Rust Port                     |
| -------------- | ---------------------------------- | ----------------------------- |
| Font loading   | TTFont objects with lazy loading   | Raw bytes parsed into FontRef |
| Glyph order    | Modified in-place, fonts reloaded  | Computed upfront, immutable   |
| Options        | `Options` class with many settings | Minimal `Options` struct      |
| Logging        | Python logging module              | `log` crate                   |
| Error handling | Exceptions                         | `Result<T, MergeError>`       |

### Merge Strategy Differences

| Table Field                | fontTools                    | Rust Port                   |
| -------------------------- | ---------------------------- | --------------------------- |
| head.flags bit 2           | Taken from first             | Taken from first ✓          |
| head.flags bits 5-10       | Reserved, taken from first   | Taken from first ✓          |
| maxp.maxStorage            | `first`                      | `first` ✓                   |
| maxp.maxFunctionDefs       | `first` (TODO: should merge) | `first` ✓                   |
| maxp.maxSizeOfInstructions | `first` (TODO: should merge) | `first` ✓                   |
| post.formatType            | `max`                        | `first` (differs)           |
| post.mapping               | `sumDicts`                   | Not preserved (version 3.0) |

### Not Implemented

- `--drop-ot-layout` option (drop all GSUB/GPOS)
- `--enable-bitmap-merging` for EBDT/EBLC/EBSC tables
- BASE table merging (partial in fontTools too)
- STAT table merging
- COLR/CPAL color font merging
- Variable font (fvar, gvar, HVAR, etc.) merging
- CID-keyed CFF fonts
- Font collections (TTC)

## License

MIT OR Apache-2.0
