#!/bin/bash
# Run Rust tests and clean up generated files

set -e

cd "$(dirname "$0")/../.."

echo "Running cargo test..."
cargo test "$@"

echo ""
echo "Cleaning up test artifacts..."
# Remove PNG/PDF files from root and examples/output
rm -f *.png *.pdf examples/output/*.png examples/output/*.pdf examples/output/*.txt
# Also clean up /tmp test files
rm -f /tmp/colorbar*.png

echo "✓ Tests complete, artifacts cleaned"
