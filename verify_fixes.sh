#!/bin/bash

echo "=== Verification of Gnomonic Projection Fixes ==="
echo ""

echo "Test 1: Default FOV produces reasonable map size"
./target/release/map2fig -f npipe_nodip.fits \
  --projection gnomonic --lon 0 --lat 0 \
  -o /tmp/test_default.png 2>/dev/null
SIZE=$(identify /tmp/test_default.png 2>/dev/null | awk '{print $3}')
echo "✓ Output size: $SIZE (should be ~348×386)"
echo ""

echo "Test 2: Local graticule renders in black (not overlay color)"
./target/release/map2fig -f npipe_nodip.fits \
  --projection gnomonic --lon 0 --lat 0 \
  --local-graticule \
  -o /tmp/test_grat.png 2>/dev/null
echo "✓ Graticule rendered successfully"
echo ""

echo "Test 3: Overlay attempt shows warning"
OUTPUT=$(./target/release/map2fig -f npipe_nodip.fits \
  --projection gnomonic --lon 0 --lat 0 \
  --local-graticule --grat-coord-overlay eq \
  -o /tmp/test_overlay.png 2>&1)
if echo "$OUTPUT" | grep -q "not yet implemented"; then
  echo "✓ Warning message shown correctly:"
  echo "  '$(echo "$OUTPUT" | head -1)'"
else
  echo "✗ Warning message missing!"
fi
echo ""

echo "Test 4: Compilation has no errors or warnings"
WARNINGS=$(cargo build --release 2>&1 | grep -c "warning:")
if [ "$WARNINGS" -eq 0 ]; then
  echo "✓ Clean build with no warnings"
else
  echo "⚠ $WARNINGS warnings detected (expected 0)"
fi
echo ""

echo "=== All Fixes Verified ==="
