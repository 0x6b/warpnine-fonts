# font-feature-freezer

Permanently apply OpenType GSUB features by remapping the cmap table.

A Rust port of [fonttools-opentype-feature-freezer](https://github.com/twardoch/fonttools-opentype-feature-freezer) (`pyftfeatfreeze`).

## Installation

```bash
cargo install font-feature-freezer
```

## Usage

### CLI

```bash
# Freeze features into a font (output to input.featfreeze.ttf)
font-feature-freezer -f ss01,ss02 input.ttf

# Specify output file
font-feature-freezer -f ss01,ss02 input.ttf output.ttf

# Add suffix to font family name
font-feature-freezer -f c2sc,smcp -S -U SC OpenSans.ttf OpenSansSC.ttf

# Replace strings in font name table
font-feature-freezer -R 'Lato/Otal' Lato-Regular.ttf Otal-Regular.ttf

# Filter by script and language
font-feature-freezer -f smcp -s latn -l DEU input.ttf output.ttf

# Report available features
font-feature-freezer --report input.ttf

# Verbose output with glyph names
font-feature-freezer -f onum -v -n input.ttf output.ttf
```

### CLI Options

If `OUTPUT` is omitted, the font is written to `<input>.featfreeze.<ext>`.

```
Options:
  -f, --features <FEATURES>      Comma-separated feature tags (e.g., 'smcp,c2sc,onum')
  -s, --script <SCRIPT>          OpenType script tag (e.g., 'cyrl')
  -l, --lang <LANG>              OpenType language tag (e.g., 'SRB ')
  -z, --zapnames                 Set post table to version 3 (remove glyph names)
  -S, --suffix                   Add suffix to font family name
  -U, --usesuffix <USESUFFIX>    Custom suffix (implies --suffix)
  -R, --replacenames <REPLACE>   Search/replace in name table: 'old/new,old2/new2'
  -i, --info                     Update font version string
  -r, --report                   Report scripts, languages, and features
  -n, --names                    Output remapped glyph names
  -v, --verbose                  Verbose output
  -q, --quiet                    Suppress output except errors
```

### Library

```rust
use font_feature_freezer::{freeze, FreezeOptions};

let font_data = std::fs::read("input.ttf")?;

// Simple usage
let options = FreezeOptions::new(["ss01", "ss02"]);
let result = freeze(&font_data, &options)?;
std::fs::write("output.ttf", result.data)?;

// With all options
let options = FreezeOptions::new(["ss01", "ss02"])
    .with_script("latn")
    .with_lang("DEU ")
    .with_usesuffix("SC")
    .with_info();
let result = freeze(&font_data, &options)?;

// Check warnings
for warning in &result.warnings {
    eprintln!("WARNING: {warning}");
}

// Access statistics
println!("Applied {} substitutions", result.stats.substitutions_applied);
```

Simple API (backwards compatible):

```rust
use font_feature_freezer::freeze_features;

let font_data = std::fs::read("input.ttf")?;
let frozen = freeze_features(&font_data, &["ss01", "ss02"])?;
std::fs::write("output.ttf", frozen)?;
```

## How It Works

1. Parses the GSUB table to find lookups for the requested features
2. Optionally filters by script and language
3. Builds a substitution map from Single and Alternate substitution lookups
4. Remaps the cmap table so Unicode codepoints point directly to substituted glyphs
5. Optionally modifies the name table (family name suffix, replacements)
6. Rebuilds the font with modified tables

## Supported Lookup Types

- **Single substitution** (format 1 and 2)
- **Alternate substitution** (uses the first alternate)

Unsupported lookup types (ligature, contextual, etc.) are silently skipped.

## Performance

~190x faster than the Python version (0.065s vs 12.37s for 16 fonts).

## Testing

```bash
cargo test
```

Tests are ported from [fonttools-opentype-feature-freezer](https://github.com/twardoch/fonttools-opentype-feature-freezer/tree/master/tests), using the same OpenSans-Bold subset font (Apache 2.0 licensed).

## License

MIT OR Apache-2.0
