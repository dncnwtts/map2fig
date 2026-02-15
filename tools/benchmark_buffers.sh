#!/bin/bash

echo "=== BUFFERED I/O OPTIMIZATION BENCHMARK ==="
echo "Testing BufReader with 256 KB buffer optimization"
echo

BINARY="./target/release/healpix_plotter"
DATA_DIR="./tests/data"

# Test files with different sizes
TEST_FILES=(
    "m_test.fits"
    "class_dr1_40GHz_skymap_n128.fits"
    "cosmoglobe_DIRBE_06_I_n00512_DR2.fits"
    "cosmoglobe_clipped.fits"
    "combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits"
)

echo "File,Size (MB),Time (ms)"

for file in "${TEST_FILES[@]}"; do
    filepath="$DATA_DIR/$file"
    if [ -f "$filepath" ]; then
        SIZE_MB=$(($(stat --format=%s "$filepath") / 1024 / 1024))
        echo -n "$file,$SIZE_MB,"
        
        # Run once and measure time with /usr/bin/time
        /usr/bin/time -f "%E" $BINARY -f "$filepath" -o /tmp/test.pdf > /dev/null 2>&1
    fi
done
