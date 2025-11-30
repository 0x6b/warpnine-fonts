# Warpnine Fonts

Custom monospace font combining:

- [Recursive Mono Duotone](https://github.com/arrowtype/recursive) 1.085 (Linear for Light/Regular, Casual for Medium+)
- [Noto Sans Mono CJK JP](https://github.com/notofonts/noto-cjk) commit f8d15753 for Japanese characters

And proportional sans-serif variants based on Recursive Sans Linear.

## Download

Pre-built fonts are available from [GitHub Releases](https://github.com/0x6b/warpnine-fonts/releases). Each release includes:

- Variable Font: `WarpnineMono-VF.ttf` (recommended)
- Static Fonts: Light through ExtraBlack, upright and italic
- Sans Fonts: `WarpnineSans-*.ttf` (proportional sans-serif, Latin only)
- Condensed Fonts: `WarpnineSansCondensed-*.ttf` (proportional sans-serif, 85% width, Latin only)
- License file: `OFL`

Releases are automatically built and published via GitHub Actions when a tag matching the pattern `v[YYYY-MM-DD]` or `[YYYY-MM-DD]` is pushed.

## Features

- Variable Font with 2 axes:
  - `wght`: 300 (Light) to 1000 (ExtraBlack)
  - `ital`: 0 (Upright) to 1 (Italic)
- Programming Ligatures: `->`, `=>`, `>=`, `!=`, `===`, `&&`, `||`, and more (frozen via `dlig` and `liga`)
- Always-Active OpenType Features (frozen at build time):
  - `dlig`, `liga`, `pnum`
  - `ss01` through `ss08`, `ss10`, `ss11`, `ss12`
  - See [OpenType Features](#opentype-features) for details
- CJK Support: Full Japanese character coverage (99% Kanji, 98% Hiragana/Katakana)
- Static Fonts: Light through ExtraBlack, both upright and italic
- Mixed CASL Style: Light/Regular use Linear (traditional), Medium+ use Casual (rounder)

## Requirements

- Python 3.10+
- [uv](https://docs.astral.sh/uv/)
- [FontForge](https://fontforge.org/)

## Setup

```console
$ brew install fontforge
$ curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Build

```console
$ uv run all
$ uv run all --date 2025-01-15  # with explicit version date
```

That will generate the following fonts:

- Variable font: `dist/WarpnineMono-VF.ttf` (74 MB)
- Static fonts: `dist/WarpnineMono-*.ttf` (18 MB each)
  - Light, Regular, Medium, SemiBold, Bold, ExtraBold, Black, ExtraBlack
  - Each with upright and italic variants
- Sans fonts: `dist/WarpnineSans-*.ttf`
  - Light, Regular, Medium, SemiBold, Bold, ExtraBold, Black
  - Each with upright and italic variants
- Condensed fonts: `dist/WarpnineSansCondensed-*.ttf`
  - Light, Regular, Medium, SemiBold, Bold, ExtraBold, Black
  - Each with upright and italic variants

You can run each step in the build pipeline manually.

````bash
# Download Noto Sans Mono CJK JP and Recursive VF.
uv run download

# Extract 16 Duotone instances from Recursive VF (Light through ExtraBlack).
# Light/Regular use Linear (CASL=0), Medium+ use Casual (CASL=1).
uv run extract-duotone

# Remove the three-backtick (```) ligature from extracted Recursive fonts.
# Must run before merge to ensure ligature is removed from final fonts.
uv run remove-ligatures

# Extract weights (400 and 700) from Noto Sans Mono CJK JP Variable Font as build/Noto-*.ttf.
uv run extract-weights

# Extract only Japanese characters from Noto fonts as build/Noto-*-subset.ttf.
# - Basic ranges:
#   - 3000-303F: CJK Symbols and Punctuation
#   - 3041-3096, 3099-309F: Hiragana
#   - 30A0-30FF: Katakana
#   - 4E00-9FFF: CJK Unified Ideographs (Kanji)
#   - FF00-FFEF: Fullwidth ASCII variants
# - Extended ranges:
#   - Kana Extended-A, B, Supplement, Small Kana Extension
#   - CJK Unified Ideographs Extension A-I
#   - CJK Compatibility Ideographs
uv run subset

# Merge Recursive Duotone (English) and Noto CJK subset (Japanese) to dist/WarpnineMono-*.ttf.
uv run merge

# Freeze OpenType features in static mono fonts (while they still have GSUB).
# Features: dlig, ss01, ss02, ss03, ss04, ss05, ss06, ss07, ss08, ss10, ss11, ss12, pnum, liga
# Static fonts are backed up before VF build and restored after.
uv run freeze-features

# Create a single Variable Font from the static fonts as dist/WarpnineMono-VF.ttf.
# GSUB tables are removed before building to avoid varLib incompatibility.
uv run build

# Copy GSUB table (with programming ligatures and FeatureVariations) from Recursive VF.
# This adds back all the OpenType features and ligatures to the variable font.
uv run copy-gsub

# Ensure every output font advertises itself as monospaced:
# This updates post.isFixedPitch, OS/2.panose.bProportion, and OS/2.xAvgCharWidth for all fonts in dist/.
uv run set-monospace

# Create condensed variant of Recursive Sans Linear (proportional, 85% width, Latin only).
# Extracts static instances and applies horizontal scaling.
uv run create-condensed

# Create non-condensed variant of Recursive Sans Linear (proportional, Latin only).
uv run create-sans

# Freeze OpenType features into VF and Sans fonts.
# WarpnineMono-VF: dlig, ss01, ss02, ss03, ss04, ss05, ss06, ss07, ss08, ss10, ss11, ss12, pnum, liga
# WarpnineSans/WarpnineSansCondensed: ss01, ss02, ss03, ss04, ss05, ss06, ss07, ss08, ss10, ss12, case, pnum, liga
uv run freeze-features

# Embed a version string (`Version yyyy-mm-dd`) into each font.
# The script synchronises the `name` and `head` tables so downstream apps see a consistent version number.
uv run set-version  # use today's date
uv run set-version --date 2025-11-01  # explicit date

# Validate quickly the variable font.
uv run test

# Start HTTP server from project root to see the preview at http://localhost:8000/preview.html
uv run python -m http.server 8000

# Clean up
uv run clean
````

## Technical Details

### Build Process

The build pipeline creates fonts with frozen OpenType features:

1. Extract instances from Recursive VF:
   - Light/Regular (300-400): Linear (CASL=0) for traditional appearance
   - Medium and heavier (500-1000): Casual (CASL=1) for better readability
2. Remove three-backtick ligature from extracted fonts
3. Merge with Noto CJK using fontforge to add Japanese character support
4. Freeze OpenType features in static mono fonts (while GSUB is intact)
5. Backup frozen static fonts
6. Build variable font with fontTools varLib (GSUB removed before building)
7. Copy GSUB table from original Recursive VF to the VF
8. Restore frozen static fonts
9. Create WarpnineSansCondensed from Recursive Sans Linear (85% width)
10. Create WarpnineSans from Recursive Sans Linear (100% width)
11. Freeze OpenType features in VF and Sans fonts

### Font Axes

- wght (Weight): 300-1000
  - 300: Light, 400: Regular, 500: Medium, 600: SemiBold
  - 700: Bold, 800: ExtraBold, 900: Black, 1000: ExtraBlack
- ital (Italic): 0-1
  - 0: Upright, 1: Italic

### OpenType Features

See [arrowtype/recursive-code-config](https://github.com/arrowtype/recursive-code-config) for detail of the each feature.

#### WarpnineMono (Frozen at Build)

- `dlig`: Discretionary ligatures (programming ligatures: `->`, `=>`, `>=`, `!=`, `===`, etc.)
- `liga`: Standard ligatures
- `pnum`: Proportional figures
- `ss01`: Single-story a
- `ss02`: Single-story g
- `ss03`: Simplified f
- `ss04`: Simplified i
- `ss05`: Simplified l
- `ss06`: Simplified r
- `ss07`: Serifless I
- `ss08`: Serifless L and Z
- `ss10`: Dotted 0
- `ss11`: Simplified 1
- `ss12`: Simplified @

#### WarpnineSans and WarpnineSansCondensed (Frozen at Build)

- `case`: Case-sensitive forms
- `liga`: Standard ligatures
- `pnum`: Proportional figures
- `ss01`: Single-story a
- `ss02`: Single-story g
- `ss03`: Simplified f
- `ss04`: Simplified i
- `ss05`: Simplified l
- `ss06`: Simplified r
- `ss07`: Serifless I
- `ss08`: Serifless L and Z
- `ss10`: Dotted 0
- `ss12`: Simplified @

#### Additional Features (Variable Font)

The variable font retains additional OpenType features from Recursive:

- Stylistic Sets: `ss09`, `ss20`
- Other: `zero`, `frac`, `locl`, `calt`, and more

## License

SIL Open Font License. See [OFL](./OFL) for detail. This project combines fonts with the following licenses:

- Recursive Mono: [SIL Open Font License](https://raw.githubusercontent.com/arrowtype/recursive/refs/tags/v1.085/OFL.txt)
- Noto Sans Mono CJK JP: [SIL Open Font License](https://raw.githubusercontent.com/notofonts/noto-cjk/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/LICENSE)

## Development

### Code Formatting and Linting

This project uses [Ruff](https://github.com/astral-sh/ruff) for code formatting and linting.

```bash
# Install dev dependencies
uv sync --dev

# Check code
uv run ruff check src/

# Auto-fix issues
uv run ruff check --fix src/

# Format code
uv run ruff format src/

# Check formatting (without modifying files)
uv run ruff format --check src/
```
