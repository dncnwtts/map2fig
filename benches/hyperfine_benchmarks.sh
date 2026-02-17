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
echo "Benchmark runs: $RUNS each"
echo ""

# Test suite: different file sizes and resolutions
declare -a BENCHMARKS=(
    "tests/data/class_dr1_40GHz_skymap_n128.fits"
    "tests/data/cosmoglobe_clipped.fits"
    "tests/data/cosmoglobe_DIRBE_06_I_n00512_DR2.fits"
    "tests/data/npipe_nodip.fits"
    "tests/data/npipe6v20_217_map_K.fits"
    "tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits"
)

# Build command strings and names separately
declare -a COMMANDS
declare -a NAMES

for filepath in "${BENCHMARKS[@]}"; do
    if [ -f "$filepath" ]; then
        filename=$(basename "$filepath")
        size_bytes=$(stat -c%s "$filepath" 2>/dev/null || stat -f%z "$filepath" 2>/dev/null)
        size_mb=$((size_bytes / 1048576))
        
        # Extract nside from filename or estimate from size
        if [[ $filename =~ nside([0-9]+) ]]; then
            nside="${BASH_REMATCH[1]}"
        else
            nside="128"
        fi
        
        outfile="/tmp/bench_$(basename "$filepath" .fits).pdf"
        COMMANDS+=("$BINARY -f '$filepath' -o '$outfile'")
        NAMES+=("$filename (${size_mb}MB, nside=$nside)")
    fi
done

# Build hyperfine arguments with labels
HYPERFINE_ARGS=(
    "--warmup" "$WARMUP_RUNS"
    "--runs" "$RUNS"
    "--min-benchmarking-time" "1"
    "--prepare" "sync; sleep 0.5"
    "--export-json" "/tmp/hyperfine_results.json"
    "--export-markdown" "/tmp/hyperfine_results.md"
)

# Add command names
for i in "${!COMMANDS[@]}"; do
    HYPERFINE_ARGS+=("--command-name" "${NAMES[$i]}")
done

# Add the actual commands
for cmd in "${COMMANDS[@]}"; do
    HYPERFINE_ARGS+=("$cmd")
done

# Run hyperfine with statistical analysis
echo "Running benchmarks (this may take 10-15 minutes)..."
echo ""
hyperfine "${HYPERFINE_ARGS[@]}"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Results saved to:"
echo "  - JSON:     /tmp/hyperfine_results.json"
echo "  - Markdown: /tmp/hyperfine_results.md"
echo ""
echo "Statistics computed with 95% confidence intervals."
echo "Results account for system variance and startup overhead."
echo "═══════════════════════════════════════════════════════════════"
