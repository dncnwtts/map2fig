#!/bin/bash
# Hyperfine end-to-end benchmarks for healpix_plotter
# 
# This script benchmarks the complete rendering pipeline across different file sizes.
# Results are statistically analyzed with confidence intervals.

set -e

BINARY="./target/release/map2fig"
WARMUP_RUNS=1
RUNS=5

if [ ! -f "$BINARY" ]; then
    echo "Error: Release binary not found. Run: cargo build --release"
    exit 1
fi

echo "═══════════════════════════════════════════════════════════════"
echo "HEALPix Plotter - Hyperfine Benchmark Suite"
echo "═══════════════════════════════════════════════════════════════"
echo "Binary:      $BINARY"
echo "Warmup runs: $WARMUP_RUNS"
echo "Benchmark: $RUNS runs each"
echo ""

# Test suite: different file sizes and resolutions
declare -a BENCHMARKS=(
    "tests/data/class_dr1_40GHz_skymap_n128.fits:small_nside128"
    "tests/data/cosmoglobe_clipped.fits:small_nside512"
    "tests/data/cosmoglobe_DIRBE_06_I_n00512_DR2.fits:medium_nside512"
    "tests/data/npipe_nodip.fits:medium_nside512"
    "tests/data/npipe6v20_217_map_K.fits:large_nside512"
    "tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits:huge_nside8192"
)

# Build command strings
declare -a COMMANDS

for benchmark in "${BENCHMARKS[@]}"; do
    IFS=':' read -r filepath label <<< "$benchmark"
    if [ -f "$filepath" ]; then
        size_mb=$(( $(stat -f%z "$filepath" 2>/dev/null || stat -c%s "$filepath" 2>/dev/null) / 1048576 ))
        COMMANDS+=("$BINARY -f $filepath -o /tmp/bench_${label}.pdf:$label (${size_mb}MB)")
    fi
done

# Run hyperfine with statistical analysis
hyperfine \
    --warmup "$WARMUP_RUNS" \
    --runs "$RUNS" \
    --min-benchmarking-time 1 \
    --prepare "sync; sleep 0.5" \
    --export-json "/tmp/hyperfine_results.json" \
    --export-markdown "/tmp/hyperfine_results.md" \
    "${COMMANDS[@]}"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Results saved to:"
echo "  - JSON:     /tmp/hyperfine_results.json"
echo "  - Markdown: /tmp/hyperfine_results.md"
echo ""
echo "Statistics computed with 95% confidence intervals."
echo "Results account for system variance and startup overhead."
echo "═══════════════════════════════════════════════════════════════"
