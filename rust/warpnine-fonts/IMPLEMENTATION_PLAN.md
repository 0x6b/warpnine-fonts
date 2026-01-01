# Rust CLI Implementation Plan

## Overview

Missing commands to fully migrate the Python pipeline (excluding VF build).

---

## 1. `clean` - Remove build artifacts

**Complexity:** Trivial  
**Dependencies:** None (std only)

**Implementation:**
```rust
// src/clean.rs
pub fn clean(build_dir: &Path, dist_dir: &Path) -> Result<()>
```

**Logic:**
- Remove `build/` directory if exists
- Remove `dist/` directory if exists
- Print status for each

**CLI:**
```
warpnine-fonts clean
```

---

## 2. `download` - Download source fonts

**Complexity:** Easy  
**Dependencies:** `reqwest` (blocking), `zip`

**Implementation:**
```rust
// src/download.rs

const DOWNLOADS: &[DownloadItem] = &[
    // Noto Sans Mono CJK JP VF
    DownloadItem {
        url: "https://raw.githubusercontent.com/notofonts/noto-cjk/.../NotoSansMonoCJKjp-VF.ttf",
        output: "NotoSansMonoCJKjp-VF.ttf",
    },
    // Licenses...
];

const RECURSIVE_ZIP_URL: &str = "https://github.com/arrowtype/recursive/releases/download/v1.085/ArrowType-Recursive-1.085.zip";
const RECURSIVE_ZIP_PATH: &str = "ArrowType-Recursive-1.085/Recursive_Desktop/Recursive_VF_1.085.ttf";

pub fn download(build_dir: &Path) -> Result<()>
```

**Logic:**
1. Create `build/` directory
2. Download direct files (Noto VF, licenses)
3. Download Recursive zip, extract VF from it
4. Report success/failure count

**CLI:**
```
warpnine-fonts download [--build-dir <PATH>]
```

---

## 3. `remove-ligatures` - Remove triple-backtick ligature

**Complexity:** Medium  
**Dependencies:** `read-fonts`, `write-fonts`

**Implementation:**
```rust
// src/ligatures.rs
pub fn remove_grave_ligature(path: &Path) -> Result<bool>
```

**Logic:**
1. Parse GSUB table
2. Find Lookup Type 6 (chaining contextual) with:
   - InputCoverage[0] containing "grave"
   - LookAheadCoverage with 2 entries, both containing "grave"
   - Clear SubstLookupRecord for this subtable
3. Find Lookup Type 1 (single substitution) with:
   - mapping "grave" → "grave_grave_grave.code"
   - Remove this mapping
4. Rebuild and save font

**CLI:**
```
warpnine-fonts remove-ligatures <FILES>...
```

---

## 4. `copy-gsub` - Copy GSUB table between fonts

**Complexity:** Easy  
**Dependencies:** `read-fonts`, `write-fonts`

**Implementation:**
```rust
// src/copy_table.rs
pub fn copy_gsub(source: &Path, target: &Path) -> Result<()>
```

**Logic:**
1. Read source font, extract raw GSUB table bytes
2. Read target font
3. Replace GSUB table with source's GSUB
4. Save target font

**CLI:**
```
warpnine-fonts copy-gsub --from <SOURCE> --to <TARGET>
```

---

## 5. `create-condensed` - Create condensed fonts (90% width)

**Complexity:** Medium-High  
**Dependencies:** `read-fonts`, `write-fonts`, `kurbo`

**Implementation:**
```rust
// src/condense.rs
pub fn create_condensed(input_vf: &Path, output_dir: &Path, scale: f32) -> Result<()>
```

**Logic for each instance:**
1. Extract static instance from VF (use font-instancer)
2. Scale all glyph coordinates horizontally by 0.9:
   - For simple glyphs: scale all x coordinates
   - For composite glyphs: scale component x offsets
3. Scale metrics:
   - `hmtx`: advance widths and LSBs
   - `hhea`: advanceWidthMax, minLeftSideBearing, minRightSideBearing, xMaxExtent
   - `head`: xMin, xMax
   - `OS/2`: xAvgCharWidth, usWidthClass = 3 (Condensed)
4. Update name table with "Warpnine Sans Condensed"
5. Save font

**Instances (14 total):**
| Style | wght | MONO | CASL | slnt | CRSV |
|-------|------|------|------|------|------|
| Light | 300 | 0 | 0 | 0 | 0.5 |
| LightItalic | 300 | 0 | 0 | -15 | 1 |
| Regular | 400 | 0 | 0 | 0 | 0.5 |
| ... | ... | ... | ... | ... | ... |

**CLI:**
```
warpnine-fonts create-condensed --input <VF> --output-dir <DIR> [--scale 0.9]
```

---

## 6. `create-sans` - Create proportional sans fonts

**Complexity:** Medium  
**Dependencies:** `read-fonts`, `write-fonts`, font-instancer

**Implementation:**
```rust
// src/sans.rs
pub fn create_sans(input_vf: &Path, output_dir: &Path) -> Result<()>
```

**Logic for each instance:**
1. Extract static instance from VF with MONO=0 (proportional)
2. Update name table with "Warpnine Sans"
3. Save font

**Same 14 instances as condensed, but with MONO=0 instead of MONO=1**

**CLI:**
```
warpnine-fonts create-sans --input <VF> --output-dir <DIR>
```

---

## 7. File operations (backup/restore)

**Complexity:** Trivial  
**Can be shell commands or simple subcommands**

```
warpnine-fonts backup <PATTERN> --to <DIR>
warpnine-fonts restore <DIR> --to <DIR>
```

Or just use shell: `cp dist/WarpnineMono-*.ttf build/frozen/`

---

## Dependencies to Add

```toml
[dependencies]
# Existing
clap = { version = "4", features = ["derive"] }
read-fonts = "0.36"
write-fonts = "0.44"
anyhow = "1"
rayon = "1.10"

# New
reqwest = { version = "0.12", features = ["blocking"] }
zip = "2"
```

---

## File Structure

```
src/
├── main.rs          # CLI entry point
├── clean.rs         # NEW: clean command
├── download.rs      # NEW: download command
├── ligatures.rs     # NEW: remove-ligatures command
├── copy_table.rs    # NEW: copy-gsub command
├── condense.rs      # NEW: create-condensed command
├── sans.rs          # NEW: create-sans command
├── freeze.rs        # Existing
├── instance.rs      # Existing
├── merge.rs         # Existing
├── metadata.rs      # Existing (set-monospace, set-version)
└── subset.rs        # Existing
```

---

## Implementation Order

1. **Phase 1: Easy wins**
   - `clean` (10 min)
   - `download` (30 min)

2. **Phase 2: GSUB operations**
   - `copy-gsub` (20 min)
   - `remove-ligatures` (1 hour)

3. **Phase 3: Font creation**
   - `create-sans` (1 hour)
   - `create-condensed` (2 hours - glyph scaling is tricky)

---

## Testing Strategy

For each command:
1. Run Python version, capture output
2. Run Rust version on same input
3. Compare output fonts with `fontTools` or binary diff
4. Benchmark timing
