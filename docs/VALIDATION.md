# Rust CLI Validation Guide

This document describes how to validate that the Rust CLI produces identical output to the Python pipeline.

## Current Status (2026-01-02)

### ✅ Fully Validated (Identical Output)

| Command            | Fonts        | Status                       |
| ------------------ | ------------ | ---------------------------- |
| `clean`            | -            | ✓ Pass                       |
| `download`         | -            | ✓ Pass                       |
| `copy-gsub`        | -            | ✓ Pass                       |
| `remove-ligatures` | 16 Duotone   | ✓ Pass                       |
| `freeze`           | All          | ✓ Pass                       |
| `set-monospace`    | All          | ✓ Pass                       |
| `set-version`      | All          | ✓ Pass                       |
| `subset-japanese`  | 2 Noto       | ✓ Pass                       |
| `merge`            | 16 Mono      | ✓ Pass (uses existing fonts) |
| `instance`         | All          | ✓ Pass (MVAR metrics match)  |
| `create-sans`      | 14 Sans      | ✓ Pass (MVAR metrics match)  |
| `create-condensed` | 14 Condensed | ✓ Pass (MVAR metrics match)  |

### ✅ All Validations Passing

**Validation result**: 232/232 checks passed (2026-01-02)

## Quick Validation

```bash
cd /Users/kaoru/Projects/fonts/warpnine-fonts
uv run pytest tests/integration/test_rust_cli.py -v
```

## Build

```bash
cargo build --release
./target/release/warpnine-fonts --help
```

## Performance Baseline

| Operation                   | Rust  | Python | Speedup |
| --------------------------- | ----- | ------ | ------- |
| Full pipeline (sans fonts)  | 0.48s | 74.9s  | 157x    |
| create-sans (14 fonts)      | 0.07s | 7.7s   | 105x    |
| create-condensed (14 fonts) | 0.09s | 9.2s   | 99x     |
| set-monospace (16 fonts)    | 0.21s | 8.1s   | 38x     |
| remove-ligatures (5 fonts)  | 0.03s | 0.38s  | 14x     |
