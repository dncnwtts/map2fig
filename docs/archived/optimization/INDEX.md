# HEALPix Plotter Optimization Status & Index

**Last Updated:** February 15, 2026  
**Current Performance:** 10.14s (55.1% improvement from 22.58s baseline)

---

## Quick Navigation

### Active Optimization Efforts
- **Current Target:** Tier 3b (Cache-Aware Loop Reordering)
- **Previous Completed:** Tier 1, 1.5, 2b, 3a
- **Status Document:** See CURRENT_TIER_STATUS.md

### Documentation Index

**Session & Results Documents** (Most Recent):
1. [Session 2 Summary](OPTIMIZATION_SESSION_2_SUMMARY.md) - Overview of Tier 2b + 3a work
2. [Tier 3a Results](TIER3A_RESULTS.md) - Lazy initialization (3.6% gain)
3. [Tier 2b Results](TIER2B_RESULTS.md) - Metadata mmap (4% gain)

**Failed/Blocked Attempts** (Don't Retry):
1. [F32 Precision Reduction](../../F32_OPTIMIZATION_RESULTS.md) - **FAILED: -2-3.7% (slower)**
   - Math is only 11.8% CPU, already LLVM-optimized
   - F64→F32 conversion overhead not worth it
   
2. SIMD Math Optimization - **BLOCKED by F32 failure**
   - Same issue: math only 11.8% of CPU
   - Bottleneck is Mollweide algorithm (77.5%), not math operations

**Detailed Analysis** (Background Context):
1. [Current Bottleneck Analysis](../../CURRENT_BOTTLENECK_ANALYSIS.md) - Post-Tier2b analysis
2. [Performance Optimization Results](../../PERFORMANCE_OPTIMIZATION_RESULTS.md) - Tier 1/1.5 details
3. [Healpix Memory Analysis](../../HEALPIX_MEMORY_ANALYSIS.md) - Deep memory profiling

**Legacy Optimization Docs** (Archived):
- docs/optimization/OPTIMIZATION_JOURNEY.md
- docs/optimization/OPTIMIZATION_ROADMAP.md
- docs/optimization/SIMD_INVESTIGATION_RESULTS.md
- docs/optimization/TRUE_SIMD_ANALYSIS.md

---

## Performance Progression

```
Session Start (Baseline):
  22.58s ← All previous optimizations (Tier 1, 1.5)

Session 2, Tier 2b (metadata mmap):
  10.51s ← -4% (434ms saved)

Session 2, Tier 3a (lazy buffer init):
  10.14s ← -3.6% (371ms saved)
  
CURRENT:
  10.14s ← Total 55.1% improvement
```

---

## Current Bottleneck (Post-Tier 3a)

### CPU Time Breakdown
| Component | % CPU | Status |
|-----------|-------|--------|
| Mollweide algorithm | 77.5% | **TARGET for Tier 3b** |
| Cairo PDF rendering | ~10% | Hard to optimize |
| Math operations | 11.8% | Already LLVM-optimized |
| Page fault handling | ~1-2% | Not worth pursuing |

### Memory Metrics
| Metric | Value | Status |
|--------|-------|--------|
| Cache misses | 31.85% | **TOO HIGH** - limiting factor |
| Instructions/Cycle | 2.05 | Moderate (goal >2.2) |
| Page faults | 1.58M | Unchanged by Tier 3a (not bottleneck) |

---

## Next Priority: Tier 3b - Cache-Aware Loop Reordering

**Rationale:**
- Cache miss rate (31.85%) is limiting factor
- Mollweide algorithm (77.5% CPU) can be reordered for better locality
- Potential gain: 5-8%

**Approach:**
1. Profile with `perf c2c` to find cache contention
2. Identify innermost loops in Mollweide projection
3. Reorder loop nest to improve spatial/temporal locality
4. Benchmark and validate

**Risk:** Medium (algorithmic changes, must preserve correctness)

---

## Tier Reference

### Completed ✅

**Tier 1: Optimized Data Loading**
- Eliminated Vec<DataValue> intermediate
- Gain: 30-35%
- Status: DONE

**Tier 1.5: MmapFitsReader for Column Data**
- Memory-mapped I/O for column reads
- Gain: 20-21%
- Status: DONE

**Tier 2b: Metadata Mmap I/O**
- BufReader → Mmap for FITS metadata
- Gain: 4%
- Status: DONE (Session 2)

**Tier 3a: Lazy Buffer Initialization**
- Skip kernel zero-init via unsafe Vec sizing
- Gain: 3.6% (via improved cache locality, not page faults)
- Status: DONE (Session 2)

### Blocked ❌

**Tier 3: SIMD Math** - **DO NOT PURSUE**
- F32 precision attempted, was SLOWER by 2-3.7%
- Math only 11.8% of CPU, already well-optimized
- See: F32_OPTIMIZATION_RESULTS.md

**Tier 3 (Original):** Vectorize scaling loop
- Same issue as SIMD math
- Math is not the bottleneck

### In Progress 🔄

**Tier 3b: Cache-Aware Loop Reordering**
- Target: Mollweide algorithm (77.5% CPU)
- Estimated: 5-8% gain
- Effort: Medium
- Risk: Medium

### Not Yet Attempted ⏳

**Tier 4: Parallel Block-Wise Loading**
- Load FITS data in parallel chunks
- Estimated: 6-10% gain
- Effort: High (threading complexity)
- Risk: High (concurrency bugs)

---

## Key Insights from Sessions

### Session 1 & 2 Discoveries
1. **Math is not the bottleneck** - Only 11.8% CPU, already LLVM-optimized
2. **Page faults are not the bottleneck** - Lazy init didn't reduce them but still improved perf
3. **Cache misses ARE the limiting factor** - 31.85% miss rate, improving here has best ROI
4. **Mollweide algorithm dominates** - 77.5% of CPU time goes to projection math loops
5. **Hammer projection is free** - Already reuses mollweide infrastructure

### What NOT to Repeat
- ❌ Don't try F32 precision reduction (confirmed -2-3.7% slower)
- ❌ Don't try SIMD math (same issue as F32)
- ❌ Don't focus on page fault reduction (not the bottleneck)
- ❌ Don't focus on math optimization (already optimized)

---

## Documentation Organization

### Location Structure
```
healpix_plotter/
├── Root level (high-level session results)
│   ├── TIER2B_RESULTS.md
│   ├── TIER3A_RESULTS.md
│   ├── OPTIMIZATION_SESSION_2_SUMMARY.md
│   ├── F32_OPTIMIZATION_RESULTS.md
│   ├── HEALPIX_MEMORY_ANALYSIS.md
│   └── PERFORMANCE_OPTIMIZATION_RESULTS.md
│
├── docs/optimization/
│   ├── INDEX.md (this file)
│   ├── CURRENT_TIER_STATUS.md
│   └── ... (legacy detailed analyses)

└── .github/
    └── copilot-instructions.md (KEY REFERENCE - includes failed tiers)
```

### How to Find Things
1. **"What's the current status?"** → Read this file (INDEX.md) top section
2. **"What have we tried?"** → Read "Tier Reference" below
3. **"Why didn't SIMD work?"** → See F32_OPTIMIZATION_RESULTS.md (root) + copilot-instructions.md
4. **"What's next?"** → See "Next Priority" section above
5. **"How did Tier X work?"** → See TIERX_RESULTS.md in root or detailed analyses in docs/optimization/

---

## Checklist for Next Optimization Attempt

- [ ] Read this INDEX.md
- [ ] Read the TIER REFERENCE section to understand what's been done
- [ ] Check F32_OPTIMIZATION_RESULTS.md to avoid repeating failed attempts
- [ ] Check .github/copilot-instructions.md "KNOWN FAILED OPTIMIZATIONS" section
- [ ] Read current bottleneck analysis (CURRENT_BOTTLENECK_ANALYSIS.md)
- [ ] Review previous successful tier (TIER3A_RESULTS.md)
- [ ] Document new work in docs/optimization/TIER3B_RESULTS.md

---

## Performance Targets

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Execution time | 10.14s | <8s | Need 20% more |
| Cache misses | 31.85% | <25% | Need 21% improvement |
| IPC | 2.05 | >2.2 | Need modest gain |
| Overall improvement | 55.1% | 70%+ | Need 27% more |

---

## Session History

| Session | Date | Work | Result |
|---------|------|------|--------|
| Session 1 | Earlier | Tier 1, 1.5 | 51.5% improvement |
| Session 2 | Feb 15, 2026 | Tier 2b, 3a | +7.3% improvement |
| Session 3 | TBD | Tier 3b | Target: +5-8% |

---

**Note:** This is the master index. Refer here to understand the full optimization history and avoid duplicating past work.
