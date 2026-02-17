# Markdown Documentation Inventory & Organization Guide

**Repository**: healpix_plotter (map2fig)  
**Date**: February 17, 2026  
**Total Markdown Files**: ~170 (excluding build artifacts)  
**Total Lines**: 43,409  
**Status**: Highly fragmented - requires organization

---

## 📊 Documentation Overview

This repository contains extensive documentation accumulated across multiple development phases. The documentation is organized into loose categories but suffers from:

- **Redundancy**: Multiple files covering similar topics
- **Obsolescence**: Many files from earlier phases (2024-2025) no longer relevant
- **Fragmentation**: No clear navigation hierarchy
- **Duplication**: Key information repeated across multiple files

### Recommended Action: Archive Old Files

Many files from development phases (0.5-1.6.2) are now obsolete. A cleanup is recommended to improve maintainability.

---

## 📁 Current Directory Structure

```
healpix_plotter/
├── Root Level Documentation (10 files)
├── docs/
│   ├── analysis/             (23 files) - Benchmark & performance analysis
│   ├── architecture/         (9 files)  - Design & implementation details
│   ├── archived/             (14 files) - Obsolete documentation (archive candidate)
│   ├── comparison/           (3 files)  - Feature comparison with healpy
│   ├── development/          (4 files)  - Development guides
│   ├── history/              (43 files) - Historical documentation (ARCHIVE)
│   ├── optimization/         (49 files) - Optimization results (ARCHIVE MOSTLY)
│   ├── reports/              (9 files)  - Test & comparison reports
│   └── README.md             (1 file)   - Docs overview
├── examples/
│   └── README.md             (1 file)   - Examples documentation
├── tools/
│   ├── README.md             (1 file)   - Tools overview
│   └── PROFILING.md          (1 file)   - Profiling guide
├── .github/
│   └── copilot-instructions.md (1 file) - AI coding guidelines
└── target/                             - Build artifacts (IGNORE)
```

---

## 📚 Detailed File Inventory by Category

### 🏠 ROOT LEVEL DOCUMENTATION (10 files)

**Purpose**: High-level navigation and current status

| File | Size | Purpose | Status |
|------|------|---------|--------|
| `README.md` | 942 lines | Main project documentation | ✅ Current |
| `INDEX.md` | 463 lines | Project index & phase history | ✅ Current |
| `IMPROVEMENTS_SUMMARY.md` | 414 lines | **Feb 17, 2026 optimization summary** | ✅ **LATEST SESSION** |
| `OPTIMIZATION_FINAL_SUMMARY.md` | 108 lines | Performance validation report | ✅ Recent |
| `OPTIMIZATION_ASYMPTOTE_ANALYSIS.md` | 136 lines | Analysis of optimization limits | ✅ Recent |
| `PERFORMANCE_BASELINE.md` | ~100 lines | Initial performance metrics | ✅ Recent |
| `FINAL_BENCHMARK_SUMMARY.md` | Variable | Benchmark compilation | ⚠️ Dated |
| `BENCHMARKING_SETUP.md` | Variable | Hyperfine/Criterion setup | ✅ Recent |
| `RELEASE_NOTES.md` | Variable | Version history | ⚠️ Needs update |
| `.github/copilot-instructions.md` | Variable | AI assistant guidelines | ✅ Current |

**Recommendation**: Keep these 5 files at root:
- README.md (primary entry point)
- INDEX.md (navigation)
- IMPROVEMENTS_SUMMARY.md (latest work)
- RELEASE_NOTES.md (user-facing)
- .github/copilot-instructions.md (developer guidelines)
- Move others to `docs/current/`

---

### 📊 docs/analysis/ (23 files)

**Purpose**: Benchmark results, performance analysis, investigation reports

| File | Purpose | Status |
|------|---------|--------|
| `BENCHMARK_COMPARISON_PHASE5.md` | Phase 5 benchmark analysis | ⚠️ Archived phase |
| `PERFORMANCE_TRACKING.md` | Historical tracking | ⚠️ Incomplete |
| `IO_OPTIMIZATION_ANALYSIS.md` | I/O bottleneck investigation | ⚠️ Partial |
| `TIER*.md` (Multiple) | Tier-specific analysis | ⚠️ Archived |
| `SESSION_DOCUMENTATION_INDEX.md` | Navigation (old) | ⚠️ Outdated |

**Status**: Ready for archival (all from 2025 phases)

**Recommendation**: Create `docs/archived/analysis/` and move most files there

---

### 🏗️ docs/architecture/ (9 files)

**Purpose**: Design documents, implementation details

| File | Purpose | Current Use |
|------|---------|-------------|
| `GNOMONIC_GRATICULE_DESIGN.md` | Graticule design spec | ✅ Reference |
| `OVERLAY_IMPLEMENTATION.md` | Overlay feature design | ✅ Reference |
| `SVG_IMPLEMENTATION.md` | SVG support design | ✅ Reference |
| `LATEX_RENDERING_PNG.md` | LaTeX rendering approach | ✅ Reference |
| `TILE_PARALLELIZATION_DESIGN.md` | Parallelization design | ✅ Reference |
| `LATEX_UNITS_GUIDE.md` | Units documentation | ✅ Reference |
| `LAYOUT_SCALING.md` | Layout calculation docs | ✅ Reference |
| `GNOMONIC_TEXT_LABELS.md` | Text label positioning | ✅ Reference |

**Status**: All current, well-maintained

**Recommendation**: Keep in place, create `docs/architecture/README.md` index

---

### 📦 docs/archived/ (14 files)

**Purpose**: Explicitly archived documentation from earlier phases

| Pattern | Files | Recommendation |
|---------|-------|-----------------|
| `CAIRO_OPTIMIZATION_*.md` | 2 files | Move to `docs/archived/optimization/` |
| `PHASE*.md` | Multiple | Move to `docs/archived/phases/` |
| `PDF_*.md` | 3 files | Move to `docs/archived/rendering/` |

**Status**: Already marked as archived, needs reorganization

---

### 📋 docs/comparison/ (3 files)

**Purpose**: Comparison with healpy library

| File | Purpose | Status |
|------|---------|--------|
| `FEATURE_COMPARISON.md` | Feature parity | ✅ Current |
| `IMPLEMENTATION_GAPS.md` | Missing features | ✅ Current |
| `ROADMAP.md` | Feature roadmap | ✅ Current |

**Status**: Well-maintained

**Recommendation**: Keep in place, create index

---

### 🛠️ docs/development/ (4 files)

**Purpose**: Developer guides and setup instructions

| File | Purpose | Status |
|------|---------|--------|
| `BENCHMARK_GUIDE.md` | How to run benchmarks | ✅ Current |
| `CLI_BUILDER_GUIDE.md` | CLI development | ✅ Current |
| `TEST_REQUIREMENTS.md` | Testing setup | ✅ Current |
| `TECTONIC_INSTALL.md` | LaTeX setup | ✅ Current |

**Status**: All relevant

**Recommendation**: Keep, create index with all guides grouped

---

### 📚 docs/history/ (43 files) **⚠️ ARCHIVE CANDIDATE**

**Purpose**: Historical documentation from development phases

**Breakdown**:
- Session summaries: 8 files
- Refactoring documentation: 15 files
- Phase-specific documentation: 12 files
- Miscellaneous historical: 8 files

**All from 2024-2025**, cover phases 0.5-1.6.2

**Recommendation**: Move all to `docs/archived/history/` or delete

---

### 🚀 docs/optimization/ (49 files) **⚠️ MASSIVE ARCHIVE CANDIDATE**

**Purpose**: Optimization attempt documentation, mostly obsolete

**Breakdown**:
- Tier-specific reports: 25 files (Tier 1-5 from 2024-2025)
- Session summaries: 8 files
- Analysis documents: 16 files

**Sample files**:
- `TIER3_OPTIMIZATION_FAILURE_ANALYSIS.md` - Why downgrade optimization failed
- `F32_OPTIMIZATION_RESULTS.md` - Why f32 reduction didn't help
- `MMAP_OPTIMIZATION_RESULTS.md` - Why mmap gave 20% improvement
- Multiple `SESSION_*.md` files - Historical session notes

**Status**: All from 2024-2025 optimization phases

**Recommendation**: 
- Keep: `OPTIMIZATION_JOURNEY.md`, `OPTIMIZATION_MASTER_INDEX.md`, `OPTIMIZATION_ROADMAP.md`
- Archive rest: `docs/archived/optimization/`
- Consider extracting "lessons learned" into a single document

---

### 📋 docs/reports/ (9 files)

**Purpose**: Test results, comparison reports

| File | Purpose | Current Use |
|------|---------|-------------|
| `GRATICULE_DIAGNOSTIC_REPORT.md` | Graticule testing | ✅ Reference |
| `CRAB_NEBULA_TEST_RESULTS.md` | Validation test | ✅ Reference |
| `HEALPY_COMPARISON.md` | healpy vs map2fig | ✅ Reference |
| `PDF_DETERMINISM_VERIFICATION.md` | PDF reproducibility | ✅ Reference |
| `INTEGRATION_TEST_RESULTS.md` | Integration testing | ✅ Reference |
| `COMPARISON.md` | Feature comparison | ✅ Reference |

**Status**: All current and relevant

**Recommendation**: Keep in place, create index

---

### 📖 examples/README.md & tools/README.md

**Purpose**: Example usage and development tools documentation

**Status**: Relevant and maintained

**Recommendation**: Keep in place

---

## 📊 Statistical Summary

```
Category                      Files    Lines    Status
────────────────────────────────────────────────────────
Root Level                     10      3,500    ✅ Good
docs/architecture/              9      2,500    ✅ Good
docs/analysis/                 23      6,000    ⚠️  Archive
docs/history/                  43     10,000    ⚠️  Archive
docs/optimization/             49     12,000    ⚠️  Archive
docs/archived/                 14      3,000    ⚠️  Move
docs/comparison/                3      1,000    ✅ Good
docs/development/               4      1,500    ✅ Good
docs/reports/                   9      2,500    ✅ Good
examples/                       1        200    ✅ Good
tools/                          2        500    ✅ Good
────────────────────────────────────────────────────────
Total (excl. build)           167     42,700   Mixed
```

---

## 🎯 Recommended Organization

### New Structure

```
healpix_plotter/
├── README.md                          ← Main entry point
├── IMPROVEMENTS_SUMMARY.md            ← Latest session
├── INDEX.md                           ← Navigation
├── RELEASE_NOTES.md                   ← Version history
├── .github/copilot-instructions.md    ← Dev guidelines
│
├── docs/
│   ├── README.md                      ← Docs overview
│   │
│   ├── current/                       ← Latest/relevant docs
│   │   ├── OPTIMIZATION_FINAL_SUMMARY.md
│   │   ├── OPTIMIZATION_ASYMPTOTE_ANALYSIS.md
│   │   ├── PERFORMANCE_BASELINE.md
│   │   ├── BENCHMARKING_SETUP.md
│   │   ├── FINAL_BENCHMARK_SUMMARY.md
│   │   └── README.md (index of current docs)
│   │
│   ├── architecture/                  ← Design & implementation
│   │   ├── README.md (index)
│   │   ├── GNOMONIC_GRATICULE_DESIGN.md
│   │   ├── OVERLAY_IMPLEMENTATION.md
│   │   ├── SVG_IMPLEMENTATION.md
│   │   ├── LATEX_RENDERING_PNG.md
│   │   ├── TILE_PARALLELIZATION_DESIGN.md
│   │   ├── LATEX_UNITS_GUIDE.md
│   │   ├── LAYOUT_SCALING.md
│   │   └── GNOMONIC_TEXT_LABELS.md
│   │
│   ├── comparison/                    ← Feature analysis
│   │   ├── README.md (index)
│   │   ├── FEATURE_COMPARISON.md
│   │   ├── IMPLEMENTATION_GAPS.md
│   │   └── ROADMAP.md
│   │
│   ├── reports/                       ← Test & validation results
│   │   ├── README.md (index)
│   │   ├── GRATICULE_DIAGNOSTIC_REPORT.md
│   │   ├── CRAB_NEBULA_TEST_RESULTS.md
│   │   ├── HEALPY_COMPARISON.md
│   │   ├── PDF_DETERMINISM_VERIFICATION.md
│   │   ├── INTEGRATION_TEST_RESULTS.md
│   │   └── COMPARISON.md
│   │
│   ├── development/                   ← Developer guides
│   │   ├── README.md (index)
│   │   ├── BENCHMARK_GUIDE.md
│   │   ├── CLI_BUILDER_GUIDE.md
│   │   ├── TEST_REQUIREMENTS.md
│   │   └── TECTONIC_INSTALL.md
│   │
│   └── archived/                      ← Historical documentation
│       ├── README.md (archival note)
│       ├── history/                   ← Phase documentation (2024-2025)
│       ├── optimization/              ← Tier-based optimization (2024-2025)
│       ├── phases/                    ← Early phase docs
│       └── analysis/                  ← Historical analysis
│
├── examples/
│   └── README.md
│
└── tools/
    ├── README.md
    ├── PROFILING.md
    └── [scripts]
```

### Rationale

1. **Root Level**: Only most critical navigation files (5-6)
2. **docs/current/**: Latest session work and current status
3. **docs/architecture/**: Living design documentation
4. **docs/comparison/**: Feature tracking for future development
5. **docs/reports/**: Test results and validation
6. **docs/development/**: Developer onboarding materials
7. **docs/archived/**: Historical documentation (2024-2025)

---

## 🔄 Migration Plan

### Phase 1: Create New Structure (5 minutes)
```bash
mkdir -p docs/current
mkdir -p docs/archived/{history,optimization,phases,analysis}
```

### Phase 2: Move Files (10 minutes)
```bash
# Move latest session docs
mv OPTIMIZATION_FINAL_SUMMARY.md docs/current/
mv OPTIMIZATION_ASYMPTOTE_ANALYSIS.md docs/current/
mv PERFORMANCE_BASELINE.md docs/current/
mv BENCHMARKING_SETUP.md docs/current/
mv FINAL_BENCHMARK_SUMMARY.md docs/current/

# Move historical docs
mv docs/history/* docs/archived/history/
mv docs/optimization/* docs/archived/optimization/
mv docs/archived/*.md docs/archived/phases/ (selective)
mv docs/analysis/* docs/archived/analysis/
```

### Phase 3: Create Index Files (10 minutes)
- `docs/README.md` - main docs overview
- `docs/current/README.md` - latest work
- `docs/architecture/README.md` - design docs index
- `docs/reports/README.md` - test results index
- `docs/development/README.md` - developer guides
- `docs/archived/README.md` - archival explanation
- `docs/comparison/README.md` - feature tracking

### Phase 4: Update Main Index (5 minutes)
- Update `INDEX.md` with new structure
- Update top-level navigation in `README.md`

---

## 📖 Documentation Access Patterns

### For Users
```
README.md → examples/README.md → docs/reports/COMPARISON.md
```

### For Developers
```
README.md → docs/development/README.md → [specific guide]
```

### For Architecture Review
```
INDEX.md → docs/architecture/README.md → [specific design]
```

### For Optimization Study
```
docs/current/IMPROVEMENTS_SUMMARY.md → docs/archived/optimization/
```

### For Historical Context
```
docs/archived/README.md → [specific phase/component]
```

---

## 🎯 Key Files to Prioritize

### ✅ Essential (Keep at Root or docs/current/)
1. **IMPROVEMENTS_SUMMARY.md** (Feb 17, 2026) - Latest session work
2. **OPTIMIZATION_FINAL_SUMMARY.md** - Performance validation
3. **OPTIMIZATION_ASYMPTOTE_ANALYSIS.md** - Optimization limits
4. **PERFORMANCE_BASELINE.md** - Performance measurements
5. **BENCHMARKING_SETUP.md** - How to benchmark

### ⚠️ Important Context (Keep in docs/archive/)
1. **docs/history/INDEX.md** - Phase history overview
2. **docs/optimization/OPTIMIZATION_JOURNEY.md** - Optimization narrative
3. **docs/optimization/TIER1_OPTIMIZATION_SUCCESS.md** - Successful tier 1
4. **docs/optimization/TIER3_OPTIMIZATION_FAILURE_ANALYSIS.md** - Why tier 3 failed

### 🗂️ Reference (Keep at docs/architecture/)
1. All architecture files - design documentation
2. All reports files - validation results
3. All development files - developer guides

---

## 📋 Implementation Checklist

- [ ] Review this inventory document
- [ ] Create new directory structure
- [ ] Move files to appropriate locations
- [ ] Create index files for each major section
- [ ] Update root README.md with new structure
- [ ] Update INDEX.md with new navigation
- [ ] Add README.md files to key directories
- [ ] Test navigation from main README
- [ ] Remove old .md files from root (keep only 5-6)
- [ ] Verify git status (confirm moves weren't deletions)
- [ ] Commit with message: "Refactor: Organize 170 markdown files into coherent structure"

---

## 📊 File Count Summary by Status

| Status | Count | Action |
|--------|-------|--------|
| ✅ Current/Active | 35 | Keep in place |
| ⚠️ Relevant but dated | 40 | Move to docs/current/ or archive |
| ❌ Obsolete (pre-2026) | 92 | Move to docs/archived/ |
| **Total** | **167** | **Organize** |

---

## 💡 Post-Organization Recommendations

1. **Document Maintenance Policy**: Establish guidelines for documentation updates
   - Mark files with date of last review
   - Archive files older than 12 months automatically
   - Link to latest version in active docs

2. **Navigation Hub**: Create `docs/README.md` as single source of truth for documentation

3. **Search Improvement**: Consider adding front matter to markdown files with keywords

4. **Automated Index**: Generate table of contents from directory structure

5. **Deprecation Warnings**: Add header to archived files explaining their status

---

**Recommendation**: Implement the suggested reorganization to improve documentation maintainability and user experience. The current structure with 170 files and 43,000 lines is difficult to navigate and maintain.

**Estimated Time to Implement**: 30-45 minutes  
**Breaking Changes**: None (purely organizational)  
**Risk Level**: Very Low (can be reverted with git)
