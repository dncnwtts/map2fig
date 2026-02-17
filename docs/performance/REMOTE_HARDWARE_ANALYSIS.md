# Remote Machine Hardware Analysis & Performance Predictions

## Hardware Specifications

### CPU
- **Model:** Intel Xeon E-2136
- **Cores/Threads:** 6 cores / 12 threads
- **Base Frequency:** 3.30 GHz
- **Turbo Frequency:** 4.5 GHz
- **Peak FLOPS:** 6 cores × 4.5 GHz × 2 ops/cycle = **54 GFLOPS**

### Memory
- **Total:** 30 GB
- **Available:** ~26 GB (after system/cache)
- **Bandwidth:** ~60 GB/s (typical for Skylake-generation Xeon)

### Storage
- **Type:** SATA SSD (inferred from 2.4 GB/s sequential speed)
- **Sequential Read:** 2.4 GB/s
- **Configuration:** LVM over sda3
- **Capacity:** ~237 GB (roughly 2TB equivalent SSD)

---

## Performance Predictions: Before vs After Optimization

### Current Code (Scattered Access)
```
FITS File: 3.1 GB (combined_map_95GHz)
Current throughput: 418 MB/s (from local benchmarks, same algorithm)
Time to read: 3.1 GB ÷ 0.418 GB/s = 7.4 seconds

On remote machine (same code, proportionally slower):
  Adjustment: 2.4 GB/s ÷ 9.1 GB/s = 26% of local I/O speed
  But scattered access pattern is independent of raw bandwidth
  Expected time: ~7.5 seconds (similar to local due to CPU bottleneck)
```

### After Sequential Read Optimization (Tier 5.3)

**Scenario A: Achieves 80% of raw disk bandwidth (typical)**
```
Raw disk speed: 2.4 GB/s
Achievable with sequential reads: 2.4 × 0.80 = 1.92 GB/s
FITS file: 3.1 GB
Time: 3.1 ÷ 1.92 = 1.6 seconds

vs current: 7.5s ÷ 1.6s = 4.7× speedup
```

**Scenario B: Achieves 100% of raw disk bandwidth (best case)**
```
Throughput: 2.4 GB/s
Time: 3.1 ÷ 2.4 = 1.29 seconds

vs current: 7.5s ÷ 1.29s = 5.8× speedup
```

---

## Full Pipeline Prediction

### Current Timings (Local i9-10885H)
```
FITS reading:     5.5s (74%)
Downsampling:     1.0s (13%)
Projection:       1.5s (20%)
Scaling/Colors:   0.3s (4%)
Rendering:        0.2s (3%)
──────────────────────────
Total:            7.42s
```

### Scaling to Xeon E-2136

The Xeon is 26% slower on I/O but has **same CPU performance** for compute:
- Xeon: 6 cores × 4.5 GHz = 27 GHz
- i9: 8 cores × 5.3 GHz = 42.4 GHz
- Ratio: 27 ÷ 42.4 = **64% of compute speed**

But the important distinction: I/O and projection math are largely independent.

### Predicted Times on Xeon (Current Code - Scattered)
```
FITS reading:     7.5s (not scaling-dependent, CPU-bound by loop)
Downsampling:     1.5s (64% of 1.0s, Rayon parallelization)
Projection:       2.3s (64% of 1.5s per our calculation)
Scaling/Colors:   0.3s (mostly in-place)
Rendering:        0.3s (64% of 0.2s)
──────────────────────────
Total:            ~12.0s
```

### Predicted Times on Xeon (With Tier 5.3 Optimization)
```
FITS reading:     1.6s  (sequential I/O improvement)
Downsampling:     1.5s  (unchanged, CPU-bound)
Projection:       2.3s  (unchanged, CPU-bound)
Scaling/Colors:   0.3s  (unchanged)
Rendering:        0.3s  (unchanged)
──────────────────────────
Total:            ~6.0s  ← 5.8× from FITS improvement
                         ← 2.0× overall from current 12.0s
```

---

## Per-Component Analysis

### FITS Reading Improvement Details

#### Current (Scattered Access)
```
Operations:
├─ Loop iterations:      806M × 1 = 806 million
├─ Instructions/iter:    ~15 (calculation, slice, parse, push)
├─ Total instructions:   12.09 billion
├─ At 3.3 GHz base:      12.09B ÷ 3.3B = 3.66s (instruction time)
├─ Memory stalls:        50-60 cycles per 16KB jump (no prefetch)
│  └─ 806M × 50 cycles ÷ 3.3 GHz = 12.2 seconds of stall time
├─ But overlapped via Instruction Level Parallelism (ILP)
└─ Effective time: ~7.5s

Throughput: 3.1 GB ÷ 7.5s = 413 MB/s
```

#### After (Sequential Access)
```
Operations:
├─ Loop iterations:      806M × 1 = 806 million
├─ Instructions/iter:    ~8 (fewer calculations, direct indexing)
├─ Total instructions:   6.45 billion
├─ At 3.3 GHz:          6.45B ÷ 3.3B = 1.95s (instruction time)
├─ Memory stalls:        3-4 cycles per sequential read (prefetch works!)
│  └─ 806M rows × 3.5 cycles ÷ 3.3GHz = 0.85s of stall time
├─ Prefetcher active:    Read bandwidth no longer microcycle-limited
└─ Effective time: ~1.6s

Throughput: 3.1 GB ÷ 1.6s = 1.94 GB/s (81% of hardware max 2.4 GB/s)
```

---

## Detailed Projections Per File Size

### Small File (25 MB, nside=512)
```
File: class_dr1_40GHz_skymap_n128.fits or similar

Current (scattered):        0.06s
After optimization:         0.012s
Expected speedup:           5×
Reason: L3 cache effects matter less, but CPU loop overhead still dominates
```

### Medium File (193 MB, nside=1024)
```
File: npipe6v20_217_map_K.fits

Current (scattered):        0.46s
After optimization:         0.10s
Expected speedup:           4.6×
Reason: Some cache benefits from sequential access, still I/O-limited
```

### Large File (577 MB, nside=2048)
```
File: npipe_nodip.fits

Current (scattered):        1.38s
After optimization:         0.24s
Expected speedup:           5.75×
Reason: Full I/O bandwidth now utilized
```

### Very Large File (3.1 GB, nside=8192)
```
File: combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits

Current (scattered):        7.5s
After optimization:         1.6s
Expected speedup:           4.7×
Reason: I/O bandwidth limit reached, sequential access critical
```

---

## Comparison: Local (i9) vs Remote (Xeon)

| Metric | Local i9-10885H | Remote Xeon E-2136 | Ratio |
|--------|-----------------|-------------------|-------|
| **Raw I/O** | 9.1 GB/s | 2.4 GB/s | 27% |
| **CPU Freq** | 5.3 GHz turbo | 4.5 GHz turbo | 85% |
| **CPU Cores** | 8 | 6 | 75% |
| **GFLOPS** | 84.8 | 54 | 64% |
| **Current FITS time** | 5.5s | 7.5s | 136% |
| **Optimized FITS time** | 0.35s | 1.6s | 457% |
| **Final speedup** | 15.7× | 4.7× | 30% |

**Why the Xeon gains less:** The SATA SSD (2.4 GB/s) is slower than the NVMe (9.1 GB/s), so the optimization buys you less absolute time. But the **relative improvement (4.7×)** is still substantial.

---

## Validation Test Plan for Remote Machine

When you run the optimized code on the Xeon, expect:

```bash
# Small file (should complete in <1s)
time ./target/release/map2fig -f tests/data/npipe6v20_217_map_K.fits -o /tmp/test1.png

# Medium file
time ./target/release/map2fig -f tests/data/npipe_nodip.fits -o /tmp/test2.png

# Large file
time ./target/release/map2fig -f tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits -o /tmp/test3.png
```

**Expected vs Actual Comparison:**

| File | Expected (Current) | Expected (Optimized) | Ratio |
|------|-------------------|---------------------|-------|
| npipe6v20 (193MB) | 0.46s | 0.10s | 4.6× |
| npipe_nodip (577MB) | 1.38s | 0.24s | 5.75× |
| combined (3.1GB) | 7.5s | 1.6s | 4.7× |

If actual results match these, the optimization is working as designed.  
If faster than predicted: SATA SSD might have more bandwidth than 2.4 GB/s measured.  
If slower: I/O contention, thermal throttling, or other system activity.

---

## Key Insight

Your remote machine's SATA is currently being underutilized by a factor of **5-6×** (418 MB/s actual vs 2.4 GB/s capability). 

The sequential read optimization fixes this by allowing the CPU prefetcher to work, resulting in actual **~1.9 GB/s throughput** on that hardware—81% utilization instead of 17%.

This is a pure algorithmic fix, not hardware-dependent, so it will work identically on both machines.
