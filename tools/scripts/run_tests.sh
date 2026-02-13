#!/bin/bash
# Run Rust tests and clean up generated files

set -e

cd "$(dirname "$0")/../.."

echo "Running cargo test..."
cargo test "$@"

echo ""
echo "Cleaning up test artifacts..."
rm -f examples/output/*.png examples/output/*.pdf examples/output/*.txt

echo "✓ Tests complete, artifacts cleaned"
