# Warpnine Mono

Custom monospace font combining:

- [Recursive Mono Duotone](https://github.com/arrowtype/recursive) 1.085
- [Noto Sans Mono CJK JP](https://github.com/notofonts/noto-cjk) commit f8d15753 for Japanese characters

## Requirements

- Python 3.10+
- [uv](https://docs.astral.sh/uv/)
- [FontForge](https://fontforge.org/)

Scripts in this repo could be cross-platform, but have not been tested on an OS other than macOS Tahoe.

## Setup

```console
$ brew install fontforge
$ curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Build

```console
$ uv run all
```

That will generate the following fonts:

- Static fonts: `dist/WarpnineMono-*.ttf`
- Variable Font: `dist/WarpnineMono-VF.ttf`

You can run each step in the build pipeline manually.

```bash
# Download Noto Sans Mono CJK JP and Recursive Mono Duotone font files with license files to build/ directory.
uv run download

# Extract weights (400 and 700) from Noto Sans Mono CJK JP Variable Font as build/Noto-*.ttf.
# Noto CJK Variable Font contains 400 (Regular), 500 (Medium), and 700 (Bold), but we only extract 400 and 700 to match Recursive Duotone's available weights.
uv run extract

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

# Create a single Variable Font from the 4 static fonts as dist/WarpnineMono-VF.ttf.
uv run build

# Ensure every output font advertises itself as monospaced:
# This updates post.isFixedPitch, OS/2.panose.bProportion, and OS/2.xAvgCharWidth for all fonts in dist/.
uv run set-monospace

# Embed a version string (`Version yyyy-mm-dd`) into each font.
# The script synchronises the `name` and `head` tables so downstream apps see a consistent version number.
uv run set-version            # use today's date
uv run set-version --date 2025-11-01  # explicit date

# Validate quickly the variable font.
uv run test

# Start HTTP server from project root to see the preview at http://localhost:8000/preview.html
uv run python -m http.server 8000 &&

# Clean up
uv run clean
```

## License

SIL Open Font License. See [OFL](./OFL) for detail. This project combines fonts with the following licenses:

- Recursive Mono Duotone: [SIL Open Font License](https://raw.githubusercontent.com/arrowtype/recursive/refs/tags/v1.085/OFL.txt)
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
