# HEALPix Plotter Optimization Documentation Index

**Overview:** Complete guide to optimization work, results, and future direction.

---

## 📋 Executive Summary

**Current Performance:** 13.79 seconds (nside=8192, 3.1GB file)  
**Total Improvement:** 64.8% from baseline (39.2s → 13.79s)  
**Status:** Near optimization ceiling due to I/O bottleneck (81% of runtime)

---

## 📚 Documentation Map

### 🎯 START HERE - Quick Orientation

**First time exploring optimizations?**
1. Read `OPTIMIZATION_QUICK_REFERENCE.md` (5 min)
2. Skim `OPTIMIZATION_AUDIT_2026.md` sections 1-3 (15 min)
3. Return here for detailed deep dives

**Planning a new optimization?**
1. Read "Recommended Next Optimizations" in `OPTIMIZATION_QUICK_REFERENCE.md`
2. Read sections 4-5 in `OPTIMIZATION_AUDIT_2026.md`
3. Check `.github/copilot-instructions.md` for what NOT to do
4. Use checklist in `OPTIMIZATION_QUICK_REFERENCE.md` → "Before You Optimize Something"

**Curious about why something was chosen?**
1. Read `OPTIMIZATION_ARCHITECTURE_DECISIONS.md`
2. Find the decision number (1-10)
3. Read rationale and trade-offs

---

## 📖 Document Descriptions

### OPTIMIZATION_AUDIT_2026.md (COMPREHENSIVE REFERENCE)

**20,000+ words | 7 hours reading time | Most important document**

**Contents:**
- Executive summary
- Complete performance timeline
- 6 successful optimizations (Tiers 1, 1.1, 1.2, 4, 2a, 2b) with:
  - How it works
  - Why it works
  - Trade-offs
  - Lessons learned
- 3 failed optimizations with root cause analysis
- Bottleneck analysis (I/O dominates)
- Amdahl's Law limiting factor
- Feasible future optimizations (GPU, caching, async I/O)
- Recommendations by priority
- Performance ceiling analysis

**When to read:**
- ✅ Planning serious optimization work (10+ hours)
- ✅ Need complete context on past attempts
- ✅ Implementing new tier
- ❌ Just want quick summary (use QUICK_REFERENCE instead)

**Key Sections:**
- Part 1: Successful Optimizations
- Part 2: Failed Optimizations (with root causes)
- Part 3: Bottleneck Analysis
- Part 4: Feasible Future Work
- Part 5: Performance Ceiling Analysis

---

### OPTIMIZATION_QUICK_REFERENCE.md (PRACTICAL GUIDE)

**2,000 words | 10 minutes reading time | For contributors**

**Contents:**
- What's been optimized (table)
- What failed (with why)
- Current bottleneck (81% I/O)
- DO/DON'T checklist before optimizing
- Recommended next optimizations
- Key lessons learned
- Development workflow (commands)
- Testing & validation checklist

**When to read:**
- ✅ Planning to make optimization changes
- ✅ Need quick overview of current state
- ✅ Want DO/DON'T checklist
- ✅ Building/benchmarking locally

**Key Section:**
- "Before You Optimize Something" → DO/DON'T checklist

---

### OPTIMIZATION_ARCHITECTURE_DECISIONS.md (THE "WHY" DOCUMENT)

**4,500 words | 15 minutes reading time | Understanding design choices**

**Contents:**
10 major architectural decisions:
1. Memory-mapped I/O by default
2. Streaming percentile with sampling
3. Lazy Rayon parallelization
4. Stable Rust only (no nightly for SIMD)
5. Direct float32 binary reading
6. Deferred GPU acceleration
7. Explicit failed optimization documentation
8. Threshold-based algorithm selection
9. I/O prioritized over math optimization
10. Version lock for reproducibility

**When to read:**
- ✅ Understanding why current approach was chosen
- ✅ Considering design change
- ✅ Curious about trade-off analysis

---

### .github/copilot-instructions.md (PROJECT GUIDELINES)

**Includes "KNOWN FAILED OPTIMIZATIONS" section:**
- F32 precision reduction (failed)
- Tier 3 downgrade-during-parsing (failed)
- Summary of lessons learned

**When to read:**
- ✅ Before attempting any optimization
- ✅ Checking what NOT to try

---

## 🗺️ Problem → Solution Map

**"I want to speed up X"**

| Want to speed up | Read this first | Then read | Action |
|------------------|-----------------|-----------|--------|
| File loading | QUICK_REFERENCE | AUDIT Part 3 | Likely already optimized (Tier 1) |
| Large map memory | QUICK_REFERENCE | AUDIT Part 1.2 | Check streaming percentile |
| Many small files | AUDIT Part 4 | ARCH Decision 7 | Caching or async I/O |
| Multi-file batch | AUDIT Part 4 | ARCH Decision 9 | Async I/O or parallelization |
| Single map rendering | AUDIT Part 4 | ARCH Decision 6 | GPU acceleration only viable option |
| Projection math | QUICK_REFERENCE | AUDIT Part 2 | Already f64x2 SIMD; f64x8 failed |
| Interactive use | AUDIT Part 4 | ARCH Decision 8 | Header caching (5% gain) |

---

## 🔍 Detailed Content Finder

### By Tier/Optimization

**Tier 1 - Fast FITS I/O**
- AUDIT Section: Part 1, subsection 1
- Quick ref: Table "What's Been Optimized"
- See also: `src/fits.rs` lines 60-200

**Tier 1.1 - Memory Mapping**
- AUDIT Section: Part 1, subsection 2
- See also: `src/fits.rs` line 223

**Tier 1.2 - Streaming Percentile**
- AUDIT Section: Part 1, subsection 3
- ARCH Decision: #2
- See also: `src/plot/mollweide.rs` new func

**Tier 4 - Rayon Parallelization**
- AUDIT Section: Part 1, subsection 4
- ARCH Decision: #3
- See also: `src/healpix.rs` parallel func

**Tier 2a/2b - SIMD**
- AUDIT Section: Part 1, subsections 5-6
- ARCH Decision: #4
- See also: `src/simd_wide.rs`

**Failed: F32 Precision**
- AUDIT Section: Part 2, subsection 1
- QUICK_REF: "What Failed" table
- Root cause: Conversion overhead

**Failed: Pre-allocation**
- AUDIT Section: Part 2, subsection 2
- File: `TIER3B_OPTIMIZATION_FAILURE_ANALYSIS.md`
- Root cause: Compiler optimization interference

**Failed: std::portable_simd**
- AUDIT Section: Part 2, subsection 3
- Full analysis: `docs/NIGHTLY_PORTABLE_SIMD_INVESTIGATION.md`
- Reason: Expected 2-3% gain, nightly dependency not worth it

---

### By Topic

**I/O Optimization**
- AUDIT Part 3: Bottleneck Analysis
- AUDIT Part 4: Future Optimizations → "Async I/O"
- ARCH Decisions: #1, #9

**Memory Optimization**
- AUDIT Part 1, subsection 3: Streaming Percentile
- ARCH Decision: #2

**Parallel Optimization**
- AUDIT Part 1, subsection 4: Rayon
- AUDIT Part 4: "GPU Acceleration"
- ARCH Decision: #3

**Math Optimization**
- AUDIT Part 1, subsections 5-6: SIMD
- AUDIT Part 2, subsection 1: Why F32 failed
- AUDIT Part 2, subsection 3: Why std::portable_simd failed
- ARCH Decisions: #4, #9

**Algorithm Design**
- ARCH Decision: #8
- AUDIT Part 4: Feasible future work

**Hardware Limits**
- AUDIT Part 5: Performance Ceiling
- AUDIT Part 3: Memory Bandwidth Analysis

---

### By Failure

**What failed and why (DON'T RETRY):**

| Failure | Location | Root Cause |
|---------|----------|-----------|
| F32 precision | AUDIT Part 2.1 | Conversion overhead > math speedup |
| Pre-allocation | AUDIT Part 2.2 | Fought compiler optimizations (71% slower!) |
| Downgrade-fusion | AUDIT Part 2.3 | Coordinate conversion overhead (25% slower) |
| std::portable_simd | docs/NIGHTLY_PORTABLE_SIMD_INVESTIGATION.md | SLEEF incompatible, 2-3% gain not worth nightly |

---

## 🎯 Optimization Goals & Status

### Achieved ✅

| Goal | Target | Status | Document |
|------|--------|--------|----------|
| Fast FITS loading | <11.2s | ✅ 1.9s | AUDIT Part 1 |
| Memory scaling | 2-3× file size | ✅ 9.4GB for 3.1GB | AUDIT 1.2 |
| Vector SIMD | f64x2 working | ✅ Done | AUDIT Part 1 |
| Parallelization | Auto-scaling | ✅ Rayon works | AUDIT 1.4 |
| Overall speedup | 60%+ | ✅ 64.8% | AUDIT Summary |

### Not Pursued ❌

| Goal | Expected | Reason | Document |
|------|----------|--------|----------|
| f64x8 SIMD | +2-3% | Nightly only, SLEEF broken | NIGHTLY_PORTABLE_SIMD... |
| F32 precision | +2-3% | Conversion overhead > speedup | AUDIT Part 2.1 |
| GPU acceleration | +3-15× | High effort, deferred | ARCH Decision 6 |
| Cache reordering | +5-8% | Not done yet | AUDIT Part 4 |

---

## 📊 Key Numbers At A Glance

```
Performance:
  Baseline:        39.2 seconds
  Current:         13.79 seconds
  Improvement:     64.8% faster

Memory (nside=8192):
  Before Tier 1.2: 45 GB peak
  After Tier 1.2:  9.4 GB peak
  Reduction:       79%

Bottleneck:
  I/O:             81% of runtime
  Projection:      14% of runtime
  Rendering:       5% of runtime

Tier Speedups:
  Tier 1:          3.4×
  Tier 1.2:        1.5×
  Tier 4:          1.36×
  Tier 2:          1.04×
  Combined:        5.8×
```

---

## 🛠️ Quick Commands Reference

```bash
# Build normally
cargo build --release

# Build with all optimizations (LTO adds 2-3%)
cargo build --release -C target-cpu=native -C lto=fat

# Benchmark
time cargo run --release -- -f combined_map_*.fits -o /tmp/out.pdf

# Profile (flamegraph)
cargo flamegraph --release -- -f file.fits -o /tmp/out.pdf

# Detailed perf metrics
perf stat -r 5 cargo run --release -- -f file.fits -o /tmp/out.pdf

# Cache analysis
perf record -e LLC-load-misses cargo run --release -- -f file.fits -o /tmp/out.pdf
```

---

## 🚦 Decision Tree: Should I Optimize X?

```
Is X slow? (Use flamegraph)
  ├─ No → STOP, not a bottleneck
  └─ Yes → Is it I/O or rendering?
     ├─ Yes (I/O) → Read AUDIT Part 4: "Async I/O" or "Cache"
     ├─ Yes (Rendering) → Only GPU helps, read ARCH Decision 6
     └─ No (CPU math) → Likely already optimized; check Tier 2 in AUDIT
```

---

## 📞 Getting Help

**"What optimization should I work on?"**
→ Read QUICK_REF: "Recommended Next Optimizations"

**"Why did optimization Y fail?"**
→ Search AUDIT "Failed" sections or check .github/copilot-instructions.md

**"How do I measure improvement?"**
→ Read QUICK_REF: "Before You Optimize Something" checklist

**"What's the hardware limit?"**
→ Read AUDIT Part 5: "Performance Ceiling Analysis"

**"Why was decision Z made?"**
→ Find Decision #Z in OPTIMIZATION_ARCHITECTURE_DECISIONS.md

**"I found a different bottleneck!"**
→ 1. Verify with flamegraph (3+ runs)
   2. Check it's not already in AUDIT Part 3
   3. Document finding
   4. Update this index

---

## 📝 File Inventory

### Primary Documents (Read These)
- `OPTIMIZATION_AUDIT_2026.md` - Comprehensive reference
- `OPTIMIZATION_QUICK_REFERENCE.md` - Practical guide
- `OPTIMIZATION_ARCHITECTURE_DECISIONS.md` - Why document
- `OPTIMIZATION_AUDIT_2026.md` - You are here
- `.github/copilot-instructions.md` - Project guidelines

### Supporting Documents
- `docs/NIGHTLY_PORTABLE_SIMD_INVESTIGATION.md` - SIMD analysis
- `docs/TIER3B_OPTIMIZATION_FAILURE_ANALYSIS.md` - Pre-allocation failure
- `docs/MASTER_OPTIMIZATION_STATUS.md` - Detailed tier breakdown
- Various `TIER*_RESULTS.md` files - Individual tier results

### Code References
- `src/fits.rs` - Tier 1 implementations
- `src/healpix.rs` - Tier 4 parallelization
- `src/simd_wide.rs` - Tier 2 SIMD
- `src/plot/mollweide.rs` - Tier 1.2 percentile
- `src/simd.rs` - SIMD interface layer

---

## ✅ Validation Checklist

When reading this documentation:
- [ ] Understand why 64.8% improvement is good (Amdahl's Law ceiling)
- [ ] Know the primary bottleneck (I/O = 81%)
- [ ] Can explain why 3 major optimizations before failed
- [ ] Know the break-even point for parallelization (50K pixels)
- [ ] Understand memory scaling achieved (79% reduction)
- [ ] Can list 3 viable future optimizations

---

## 🔄 How to Update This Index

When new optimization work is done:
1. Create tier document: `docs/optimization/TIERX_RESULTS.md`
2. Update `OPTIMIZATION_AUDIT_2026.md` "Optimization Timeline" table
3. Add summary to `OPTIMIZATION_QUICK_REFERENCE.md` "What's Been Optimized"
4. If architectural decision was made, add to `OPTIMIZATION_ARCHITECTURE_DECISIONS.md`
5. Update this index with new findings
6. Update `.github/copilot-instructions.md` with results

---

**Last Updated:** February 2026  
**Total Optimization Work:** ~80-100 hours across 19+ strategies  
**ROI:** 64.8% improvement, stable codebase, clear path forward  
**Status:** Near ceiling, next 2-3× requires algorithmic redesign (GPU)
