# HEALPix Plotter - Codebase Structure Audit

**Date:** February 14, 2026  
**Status:** Structure Analysis & Reorganization Plan

---

## Current State: Top-Level Clutter Inventory

### File Counts by Type
| Type | Count | Status |
|------|-------|--------|
| Markdown Documentation | 62 | Needs consolidation |
| Python Scripts | 8 | Needs `tools/` subdirectory |
| Shell Scripts | 4 | Needs `tools/` subdirectory |
| Rust Source Files | 3 | Likely build artifacts or tests |
| FITS Data Files | 9 | Belongs in `data/` or `examples/data/` |
| PNG Image Files | 2 | Belongs in `examples/output/` or test outputs |
| Config/Build Files | 3 | `Cargo.toml`, `Cargo.lock`, `CITATION.cff` (OK) |
| Text Output Files | 1 | Temporary test output |

### Directory Sizes
- `target/`: 5.2GB (build artifacts - OK)
- `test_debug/`: 3.8MB (debug/test data - needs move)
- `test_edge/`: 3.8MB (debug/test data - needs move)

### Total Root Files: ~100+ items

---

## Detailed File Analysis

### MARKDOWN DOCUMENTATION (62 files)

#### Category 1: Architecture & Design (14 files)
Essential documentation that should be kept prominent:
- `README.md` - Main entry point
- `INDEX.md` - Navigation hub
- `QUICK_REFERENCE.md` - Quick lookup
- `IMPLEMENTATION_SUMMARY.md` - High-level overview
- `ARCHITECTURE.md` (or consolidated from multiple)
- Planning/Reference:
  - `FEATURE_COMPARISON.md`
  - `ISSUES_AND_FEATURES.md`
  - `GNOMONIC_GRATICULE_DESIGN.md`
  - `LATEX_RENDERING_PNG.md`
  - `LATEX_UNITS_GUIDE.md`
  - `LAYOUT_SCALING.md`
  - `OVERLAY_IMPLEMENTATION.md`
  - `TECTONIC_INSTALL.md`
  - `TEST_REQUIREMENTS.md`

**Status:** Could reduce to 5-6 core docs + move detailed specs to `docs/architecture/`

#### Category 2: Session/Work Records (14 files)
Development notes & progress tracking (often redundant):
- `SESSION_SUMMARY.md`
- `RECENT_CHANGES.md`
- `CHANGES.md`
- `REFACTORING_*.md` (multiple)
- `FIXES_*.md` (multiple)
- `GNOMONIC_*.md` variants
- etc.

**Status:** Archive to `docs/history/` - useful but clutter for daily development

#### Category 3: Test & Verification Reports (12 files)
Output from testing sessions:
- `INTEGRATION_TEST_RESULTS.md`
- `CRAB_NEBULA_TEST_RESULTS.md`
- `COMPARISON.md`
- `HEALPY_COMPARISON.md`
- `PDF_DETERMINISM_VERIFICATION.md`
- `PDF_VS_PNG_ANALYSIS.md`
- `RASTERIZATION_FIX.md`
- `SYMMETRY_FIX_COMPLETE.md`
- `TRIANGLE_RENDERING_FIX.md`
- `PARAMETER_BUNDLING_COMPLETE.md`
- `FUZZING_AND_PROPERTY_TESTING.md`
- `GRATICULE_*_REPORTS.md` (3 variants)

**Status:** Move to `docs/reports/` or `docs/testing/`

#### Category 4: Refactoring Documentation (12 files)
Multiple variations on refactoring information:
- `REFACTORING_*.md` (8 variants)
- `CODE_CLEANUP_SUMMARY.md`
- `MODULAR_REFACTORING_GUIDE.md`
- `BEFORE_AFTER_ARCHITECTURE.md`
- `PARAMETER_BUNDLING_COMPLETE.md`

**Status:** Consolidate into 1-2 files, move to `docs/development/`

#### Category 5: Completion/Summary Records (10 files)
Status markers from development phases:
- `*_COMPLETE.md` (5 files)
- `*_SUMMARY.md` (5+ files)
- `SOLUTION_COMPLETE.md`
- `CLEANUP_SUMMARY.md`

**Status:** Archive to `docs/history/` or delete if superseded

### PYTHON SCRIPTS (8 files)
**Status:** Move to `tools/`

| File | Purpose | Usage |
|------|---------|-------|
| `analyze_heights.py` | Triangle height analysis | Development/testing |
| `compare_outputs.py` | Test output comparison | CI/testing |
| `compare_pdf_png.py` | Format comparison | Benchmarking |
| `cosmoglobe_benchmark.py` | Performance testing | CI |
| `debug_coordinates.py` | Coordinate debugging | Development |
| `HEIGHT_ANALYSIS.py` | Geometry analysis | Development |
| `mollview_benchmark.py` | Benchmark comparison | CI |
| (colormap generation) | See `tools/generate_colormaps.py` | Build |

**Proposed Location:** `tools/python/` or `tools/analysis/`

### SHELL SCRIPTS (4 files)
**Status:** Move to `tools/`

| File | Purpose |
|------|---------|
| `install.sh` | Installation helper |
| `run_benchmarks.sh` | Benchmark runner |
| `plot_rotation.sh` | Plotting utility |
| `verify_fixes.sh` | Test verification |

**Proposed Location:** `tools/scripts/` or just `tools/`

### RUST FILES AT ROOT (3 files)
**Status:** Investigate & remove

| File | Issue |
|------|-------|
| `test_debug.rs` | Should be in `tests/` |
| `test_edge.rs` | Should be in `tests/` |
| `build.rs` | Build script - OK at root |

**Action:** Move to proper locations, verify they're not duplicates

### FITS DATA FILES (9 files)
**Status:** Move to `data/examples/` or `examples/data/`

- `class_dr1_40GHz_skymap_n128.fits`
- `combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits`
- `cosmoglobe_clipped.fits`
- `cosmoglobe_DIRBE_06_I_n00512_DR2.fits`
- `cosmoglobe_DIRBE_10_I_n00512_DR2_v3.1.fits`
- `m_test.fits`
- `mhat_0_00_n00512_2025W17_4B.fits`
- `npipe6v20_217_map_K.fits`
- `npipe_nodip.fits`

**Proposed Location:** `examples/data/` or `data/examples/`

### TEST & DEBUG OUTPUT (misc files)
**Status:** Clean up

- `test_debug/` (3.8MB) - Move to `.gitignore`, use for local testing only
- `test_edge/` (3.8MB) - Move to `.gitignore`, use for local testing only
- `test_debug.rs` / `test_edge.rs` - Verify if needed, move or delete
- `test_output.txt` - Temporary output, should not be in repo
- `test.png` / `test_extreme.png` - Test outputs, move to `target/` or `.gitignore`
- `out.txt`, `out2.txt`, `out3.txt`, `out4.txt` - Temporary, delete
- `perf_report.txt` - Performance data, move to `reports/` if important
- `perf.data.old` - Old data, delete

---

## Proposed New Structure

```
map2fig/
├── README.md                          # Main entry (keep at root)
├── Cargo.toml                         # Build config (keep)
├── Cargo.lock                         # Lock file (keep)
├── build.rs                           # Build script (keep)
├── CITATION.cff                       # Citation info (keep)
├── docs/                              # **NEW: Consolidate all documentation**
│   ├── README.md                      # Docs navigation
│   ├── QUICK_REFERENCE.md             # Keep prominence
│   ├── architecture/
│   │   ├── DESIGN.md                  # Core architecture
│   │   ├── PROJECTIONS.md             # Projection info
│   │   ├── GRATICULE.md               # Graticule design
│   │   └── ... (consolidated from GNOMONIC_*, LATEX_*, etc.)
│   ├── development/
│   │   ├── CONTRIBUTING.md
│   │   ├── REFACTORING.md             # Consolidated refactoring guide
│   │   ├── CODE_CLEANUP.md
│   │   └── TESTING.md
│   ├── reports/
│   │   ├── TEST_RESULTS.md
│   │   ├── COMPARISONS.md
│   │   ├── BENCHMARKS.md
│   │   └── VERIFICATIONS.md
│   └── history/                       # Archive old session notes
│       └── 2025_sessions/             # Dated session logs
├── examples/                          # Examples & data (expand)
│   ├── README.md
│   ├── data/                          # **NEW: All FITS test files**
│   │   ├── cosmoglobe_*.fits
│   │   ├── npipe_*.fits
│   │   ├── class_*.fits
│   │   └── ...
│   └── output/                        # **NEW: Example output files**
│       ├── README.md
│       └── *.png (test outputs)
├── tools/                             # **CONSOLIDATE: All utility scripts**
│   ├── README.md
│   ├── scripts/
│   │   ├── install.sh
│   │   ├── run_benchmarks.sh
│   │   ├── verify_fixes.sh
│   │   └── plot_rotation.sh
│   ├── python/
│   │   ├── analyze_heights.py
│   │   ├── compare_outputs.py
│   │   ├── compare_pdf_png.py
│   │   ├── debug_coordinates.py
│   │   ├── HEIGHT_ANALYSIS.py
│   │   └── benchmarks/
│   │       ├── cosmoglobe_benchmark.py
│   │       └── mollview_benchmark.py
│   ├── colormap/
│   │   └── generate_colormaps.py      # Move from root if exists
│   └── fuzz/ (already exists)
├── src/                               # Source code (already good)
├── tests/                             # Tests (already good, add test_*.rs if needed)
├── fuzz/                              # Fuzzing targets (already exists)
├── assets/                            # Static assets (keep)
├── colormap/                          # Generated colormaps (keep)
├── target/                            # Build output (keep)
└── .gitignore                         # **UPDATE: Add test_debug, test_edge, *.png, *.txt outputs**
```

---

## Migration Plan

### Phase 1: Documentation Consolidation (Priority: HIGH)
1. **Keep at root level:** `README.md` only
2. **Create `docs/` directory structure** with:
   - `docs/QUICK_REFERENCE.md` 
   - `docs/architecture/*` (4-5 strategic files)
   - `docs/development/*` (2-3 files)
   - `docs/reports/*` (consolidated verification/test records)
   - `docs/history/*` (archive old session notes)
3. **Actions for 62 files:**
   - Consolidate refactoring docs → 1-2 files
   - Consolidate test reports → 1-2 files
   - Archive session notes → dated directory
   - Delete redundant/superseded files
   - **Result: 10-15 docs at root or in organized subdirs**

### Phase 2: Data & Examples Organization (Priority: HIGH)
1. **Create `examples/data/`**
   - Move all `*.fits` files (9 files)
2. **Create `examples/output/`**
   - Move `test*.png`, `test_extreme.png`
3. **Clean up test outputs**
   - Delete `out*.txt`, `test_output.txt`, stray `*.txt` files
4. **Update `.gitignore`**
   - Add `test_debug/`, `test_edge/`, example outputs

### Phase 3: Tools Organization (Priority: MEDIUM)
1. **Consolidate to `tools/` directory**
   - `tools/scripts/` ← 4 `.sh` files
   - `tools/python/` ← 8 `.py` files
   - `tools/python/benchmarks/` ← benchmark scripts
2. **Verify build scripts**
   - Ensure `build.rs` isn't duplicated
   - Check if `colormap/generate_colormaps.py` should move
3. **Update CI/docs** with new paths

### Phase 4: Code Cleanup (Priority: MEDIUM)
1. **Investigate root `.rs` files**
   - `test_debug.rs`, `test_edge.rs` - move or delete
   - Verify not duplicates of tests in `tests/`
2. **Clean test directories**
   - Remove `test_debug/`, `test_edge/` from repo (add to `.gitignore`)
   - Keep build artifacts in `target/`

### Phase 5: Repository Updates (Priority: MEDIUM)
1. **Update `.gitignore`** with:
   ```
   test_debug/
   test_edge/
   examples/output/*.png
   examples/output/*.txt
   *.pdf (if not tracked)
   perf.data*
   ```
2. **Update top-level `README.md`**
   - Add link to `docs/` directory structure
   - Update contributing/development instructions
3. **GitHub Pages** (if used)
   - Update doc generation to use new `docs/` structure

---

## Estimated Impact

### Before
- **Root files:** ~100+ items
- **Navigation:** Difficult, unclear organization
- **Clutter:** High (docs, data, scripts all mixed)

### After
- **Root files:** ~10 items (README, config, build script, examples, src, tests, fuzz, assets, colormap, tools, docs)
- **Navigation:** Clear hierarchy
- **Clutter:** 90% reduction
- **Maintainability:** Significantly improved

### Time Estimate
- **Phase 1 (Docs):** 2-3 hours (audit, consolidate, move)
- **Phase 2 (Data):** 30 min (move files, update paths)
- **Phase 3 (Tools):** 1 hour (organize, update CI references)
- **Phase 4 (Cleanup):** 30 min (verify, remove)
- **Phase 5 (Updates):** 1 hour (update docs/gitignore/CI)
- **Total:** ~5-6 hours for complete reorganization

---

## Recommendations

### Immediate Actions (Quick Wins)
1. ✅ Move all FITS files to `examples/data/` (5 min)
2. ✅ Move all scripts to `tools/` (5 min)
3. ✅ Add test_debug, test_edge to `.gitignore` (2 min)
4. ✅ Delete temporary output files (2 min)
5. ✅ Create `docs/` directory structure (5 min)

### High-Value Actions
1. 📋 Consolidate markdown documentation (archive 30+ files)
2. 📚 Create `docs/QUICK_REFERENCE.md` for fast navigation
3. 🔧 Move Python/Shell tools to organized `tools/` subdirs

### Optional (Lower Priority)
1. 📁 Create GitHub Pages docs site (if not already done)
2. 🗂️ Categorize examples by use case
3. 📊 Add per-directory README files for context

---

## Summary

**Current state:** Cluttered root with 62 markdown files, 8 Python scripts, 4 shell scripts, and 9 data files scattered throughout.

**Proposed solution:** Organize into clear structure with:
- **Minimal root** (config + primary entry points)
- **docs/** (all documentation organized by purpose)
- **examples/data/** (all test/example data)
- **tools/** (scripts and utilities)
- **Updated .gitignore** (temporary outputs)

**Expected result:** Professional, maintainable structure with 90% less clutter.

