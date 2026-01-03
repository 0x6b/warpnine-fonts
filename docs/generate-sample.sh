#!/bin/bash
# Generate Warpnine Fonts sample PDF
# Requires: typst (https://typst.app)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

typst compile --ignore-system-fonts --font-path ../dist/ sample.typ sample.pdf

echo "Generated: $SCRIPT_DIR/sample.pdf"
