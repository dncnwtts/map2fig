# Parallelization Archive Summary

## What Happened

The `rayon-parallelization` branch attempted to speed up HEALPix rendering using Rayon for multi-threaded pixel rendering. While technically successful, **the performance improvement was insufficient to justify the added complexity**, leading to the decision to archive rather than merge.

## Decision

**Archived**: Not merged to main  
**Branch**: `rayon-parallelization`  
**Archive tag**: `archive/rayon-parallelization-attempt`  
**Documentation**: See [PARALLELIZATION_ARCHIVE.md](./PARALLELIZATION_ARCHIVE.md)

## What Was Kept

The `--no-downgrade` flag was too useful to discard:
- Allows disabling automatic NSIDE downgrading
- Enables accurate performance benchmarking at full resolution
- Added to main branch in commit `31f8d7c`

## Performance Summary

| Scenario | Speedup |
|----------|---------|
| cosmoglobe 6000px | 20.7% |
| cosmoglobe 4000px | 17.0% |
| combined_map (3.1GB) 6400px | 11% |
| npipe_nodip 2400px | 4.2% |
| I/O limited cases | <3% |

**Expected**: 30-40%  
**Delivered**: 10-20% typical, with diminishing returns

## Why Archived

1. **Complexity vs. Gain Mismatch**
   - Per-pixel work is too small to amortize thread overhead
   - Memory bandwidth already saturated
   - I/O dominates on large files

2. **Better Alternative Exists**
   - Automatic downgrade is **2.5x faster** than parallelized full-resolution
   - Works for all typical use cases (1200-2400px)
   - Zero complexity, zero user confusion

3. **Maintenance Cost**
   - Dual code paths to test
   - Binary bloat (Rayon dependency)
   - User documentation for `--parallel` flag

## If You Need This Later

The branch contains complete, tested code. To resurrect:

```bash
git checkout rayon-parallelization
cargo build --release --features parallel
./target/release/map2fig -f data.fits -o output.pdf --parallel
```

Consider first:
1. Is downgrade insufficient for your use case?
2. Are you rendering ultra-high-resolution posters (6000px+)?
3. Would you use `--parallel` regularly enough to justify maintenance?

## Lessons Learned

1. **Not all optimizations are equal** - 10-20% improvement doesn't justify code complexity
2. **Algorithmic beats parallelization** - Downgrading is worth more than any threading
3. **Measure on real data** - I/O constraints weren't obvious until testing 3.1GB files
4. **Know your bottlenecks** - Per-pixel computation was fundamentally incompatible with Rayon

## References

- Full analysis: [PARALLELIZATION_ARCHIVE.md](./PARALLELIZATION_ARCHIVE.md)
- Benchmark results: [RAYON_PARALLELIZATION_RESULTS.md](./docs/RAYON_PARALLELIZATION_RESULTS.md) (on branch)
- Architecture notes in git history: `rayon-parallelization` branch
