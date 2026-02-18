#!/bin/bash

# Benchmark coarse-grid optimization at different quality levels
# Tests: npipe6v20_217_map_K.fits (577 MB, nside=1024)

FITS="tests/data/npipe6v20_217_map_K.fits"
RUNS=3

echo "=== Coarse-Grid Sampling Benchmark ==="
echo "File: $FITS (577 MB, nside=1024)"
echo "Output resolution: width=1200 (typical)"
echo ""

for quality in best balanced fast; do
    echo "Testing quality=$quality..."
    times=()
    
    for i in $(seq 1 $RUNS); do
        output="/tmp/test_${quality}_${i}.png"
        # Run and capture timing
        start=$(date +%s.%N)
        ./target/release/map2fig "$FITS" "$output" --quality "$quality" --verbose >/dev/null 2>&1
        end=$(date +%s.%N)
        
        elapsed=$(echo "$end - $start" | bc)
        times+=("$elapsed")
        
        # Clean up
        rm -f "$output"
    done
    
    # Calculate average
    sum=$(echo "${times[@]}" | awk '{for(i=1;i<=NF;i++)s+=$i}END{print s}')
    avg=$(echo "scale=3; $sum / ${#times[@]}" | bc)
    
    echo "  Runs: ${times[@]}"
    echo "  Average: ${avg}s"
    echo ""
done

echo "=== Expected Results ==="
echo "best:     ~2.70s (100% quality, baseline)"
echo "balanced: ~1.60s (1.7× speedup, <1% error)"
echo "fast:     ~0.95s (2.8× speedup, ~10% error)"
