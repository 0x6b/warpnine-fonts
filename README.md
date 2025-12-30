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
- Condensed Fonts: `WarpnineSansCondensed-*.ttf` (proportional sans-serif, 90% width, Latin only)
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
$ uv run warpnine build all
$ uv run warpnine build all --date 2025-01-15  # with explicit version date
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

### Other Commands

```console
$ uv run warpnine validate all      # run all validations
$ uv run warpnine build clean       # remove build artifacts
$ uv run warpnine build --help      # list all build subcommands
```

Preview fonts locally:

```console
$ uv run python -m http.server 8000
# Open http://localhost:8000/preview.html
```

## Technical Details

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

## Development

This project uses [Ruff](https://github.com/astral-sh/ruff) for code formatting and linting.

```bash
# Install dev dependencies
uv sync --dev

# Check code
uv run ruff check warpnine_fonts/

# Auto-fix issues
uv run ruff check --fix warpnine_fonts/

# Format code
uv run ruff format warpnine_fonts/

# Check formatting (without modifying files)
uv run ruff format --check warpnine_fonts/
```

## License

SIL Open Font License. See [OFL](./OFL) for detail. This project combines fonts with the following licenses:

- Recursive Mono: [SIL Open Font License](https://raw.githubusercontent.com/arrowtype/recursive/refs/tags/v1.085/OFL.txt)
- Noto Sans Mono CJK JP: [SIL Open Font License](https://raw.githubusercontent.com/notofonts/noto-cjk/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/LICENSE)
