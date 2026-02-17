# Archived Documentation (2024-2025)

This directory contains historical documentation from earlier development phases (2024-2025). Files here are preserved for reference and understanding project evolution but are not actively maintained.

## Directory Structure

### `history/`
**43 files** documenting phase history and development sessions

Contains:
- Session summaries and work logs (Feb 2024 - Feb 2025)
- Refactoring documentation and completion reports
- Feature implementation records
- Archive decision summaries

**Notable Files**:
- `REFACTORING_EXECUTIVE_SUMMARY.md` - Major 2024 refactoring overview
- `CODE_CLEANUP_SUMMARY.md` - Cleanup and technical debt work
- Phase-specific documentation from phases 0.5-1.6.2

### `optimization/`
**50 files** documenting tier-based optimization attempts

Contains:
- Tier 1: Direct float32 FITS reading (✅ successful, 3.4× speedup)
- Tier 2: Vectorization attempts (⚠️ complex, secondary priority)
- Tier 3: SIMD scaling (⛔ failed, lessons learned archived)
- Tier 4: Parallelization analysis
- Tier 5: GPU rendering explorations

**Learning Value**:
- `TIER1_OPTIMIZATION_SUCCESS.md` - What worked
- `TIER3_OPTIMIZATION_FAILURE_ANALYSIS.md` - What didn't (Amdahl's Law lesson)
- `F32_OPTIMIZATION_RESULTS.md` - Failed narrowing approach

### `analysis/`
**20 files** of historical performance analysis

Contains:
- Benchmark analysis from 2024-2025 phases
- Performance profiling reports
- Optimization roadmaps and planning docs
- Session documentation from earlier development

**Content**: Mostly superseded by [../current/](../current/) analysis, kept for reference.

### `phases/`
Early phase documentation (phases 0.5-1.4)

Contains:
- Initial project scope documentation
- Early architecture decisions
- Feature selection and planning

## Why These Files Are Archived

1. **Historical Value**: Document project decisions and learnings
2. **Reference**: Understanding "why" things were done certain ways
3. **Lessons Learned**: Failed attempts documented for future avoidance
4. **Context**: How the project evolved to current state
5. **Traceability**: Git history maintained via `git log` and blame

## Using Archived Files

For understanding:
- **"Why was X designed this way?"** → Check `history/` for old decisions
- **"What optimizations were tried?"** → Check `optimization/` for attempts
- **"Why doesn't Tier 3 exist?"** → See `TIER3_OPTIMIZATION_FAILURE_ANALYSIS.md`
- **"How did we get here?"** → Browse `phases/` for initial decisions

**Important**: Do NOT modify files here. Create new files in [../current/](../current/) for new work.

## Cleanup Plan

These files are candidates for permanent archival (moving to git history only):
- `history/` - Most files pre-Aug 2024
- `optimization/` - Completed/failed tiers
- `analysis/` - Analysis from phases 0.5-1.5

**Recommendation**: Archive to git archives or external storage in Q2 2026.

## Statistics

| Category | Files | Lines | Status |
|----------|-------|-------|--------|
| history | 43 | ~15,000 | Historical reference |
| optimization | 50 | ~18,000 | Lessons learned |
| analysis | 20 | ~8,000 | Superseded |
| phases | 8 | ~4,000 | Initial context |
| **Total** | **121** | **~45,000** | Reference archive |

## Access Pattern

If you need archived information:

```
1. Check current docs first [../current/] and [../]
2. If looking for historical context → search archived/history/
3. If investigating failed optimization → archived/optimization/
4. If understanding project origin → archived/phases/
5. For specific analysis → archived/analysis/
```

## See Also

- [../README.md](../README.md) - Main documentation hub
- [../MARKDOWN_ORGANIZATION_GUIDE.md](../MARKDOWN_ORGANIZATION_GUIDE.md) - Why docs are organized this way
- Current work: [../current/](../current/)

---

**Archive Date**: February 17, 2026  
**Last Updated**: Consolidated from multiple doc phases  
**Purpose**: Historical reference and lessons learned  
**Status**: ✅ Read-only archive, well-organized
