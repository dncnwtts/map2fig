# map2fig Development Roadmap

**Goal:** Eliminate all use cases where map2png would be preferred over map2fig.

## Current Status vs map2png

✅ = Implemented  
🔄 = Possible/Partial  
⬜ = Not Yet Implemented

| Feature | Status | Priority | Est. Effort |
|---------|--------|----------|-------------|
| **Multi-panel output** | ⬜ | **CRITICAL** | Medium |
| **Quantile-based auto-scaling** | ⬜ | **HIGH** | Low |
| **Shorter command-line syntax** | 🔄 | HIGH | Low |
| **Multi-map HDF selection** | ⬜ | MEDIUM | Low |
| **Column name support** | ⬜ | MEDIUM | Medium |
| **Batch processing mode** | ⬜ | MEDIUM | Medium |
| **Performance parity (already better)** | ✅ | - | - |
| **Feature parity** | ✅ | - | - |
| **PDF output** | ✅ | - | - |
| **Coordinate systems** | ✅ | - | - |
| **Data masking** | ✅ | - | - |

---

## Priority 1: CRITICAL - Multi-Panel Output

**Issue:** map2png users heavily rely on `-ncol` to create comparison figures in one pass.

### Current Limitation
```bash
# map2png - single command produces 2-panel figure
map2png -ncol 2 -signal 1 -signal 2 input.fits output.png

# map2fig requires external composition
./map2fig -f input.fits -o panel1.pdf -i 0
./map2fig -f input.fits -o panel2.pdf -i 1
# Then ImageMagick/manual composition needed
```

### Proposed Implementation

#### Option A: Native Multi-Panel Flag
```bash
# Proposed syntax
./map2fig -f input.fits -o comparison.pdf \
  --columns 0,1 --layout 2x1 --spacing 10

# Or with multiple passes on same file
./map2fig -f input.fits -o multi.pdf \
  --column 0,1,2,3 --layout 2x2
```

**Design Decisions Needed:**
- Layout specification: `2x1`, `1x2`, `2x2`, etc.?
- Spacing/gaps between panels?
- Shared colorbar or independent?
- Same scale or independent scaling per column?

#### Option B: Simpler Multi-Column Interface
```bash
# Like map2png's -signal approach
./map2fig -f input.fits -o output.pdf \
  -i 0 -i 1 -i 2 \
  --layout 1x3 --shared-cbar
```

**Complexity:** Medium
- Requires layout engine (panel positioning, borders)
- Colorbar merging logic
- Independent/shared scaling option

**Timeline:** 1-2 weeks

**Blockers:** None - can be developed independently

---

## Priority 2: HIGH - Quantile-Based Auto-Scaling

**Issue:** Users with outliers/bad-pixel data want robust auto-scaling.

### Current Limitation
```bash
# map2png
map2png -auto 0.1 input.fits output.png  # Uses 10th-90th percentile

# map2fig
./map2fig -f input.fits -o output.pdf    # Always uses min-max
```

### Proposed Implementation
```bash
# New option
./map2fig -f input.fits -o output.pdf --auto-quantile 0.05
# Or shorter
./map2fig -f input.fits -o output.pdf --quantile 0.1
```

**Implementation Strategy:**
1. Add `--auto-quantile` / `--quantile` parameter to CLI (src/cli.rs)
2. Modify `setup::load_data()` to calculate percentiles instead of exact min/max
3. Update `AutoScale` logic in `scale.rs`

**Complexity:** Low
- Simple percentile calculation (use ndarray or itertools)
- Minimal code changes to scaling pipeline
- No UI/layout changes

**Timeline:** 2-3 days

**Blockers:** None

---

## Priority 3: HIGH - Shorter Command Syntax (Better Discovery)

**Issue:** map2png flags are shorter and more discoverable; map2fig uses verbose long options.

### Current Gap
```bash
# map2png
map2png -color plasma -logarithmic -minimum 1e-6 -maximum 1e-3 in.fits out.png

# map2fig
./map2fig -c plasma --log --min 1e-6 --max 1e-3 -f in.fits -o out.pdf
```

### Proposed Improvements

#### Already Implemented Short Flags
✅ `-f` (--fits)  
✅ `-i` (--col)  
✅ `-c` (--cmap)  
✅ `-w` (--width)  
✅ `-o` (--out)  

#### Proposed Additional Short Flags
```bash
./map2fig -f in.fits -o out.pdf \
  -c plasma \           # colormap
  -C grid \             # -C reserved, could be --coord or similar
  -m 1e-6 \             # --min
  -M 1e-3 \             # --max
  -g 0.8 \              # --gamma
  -l                    # --log
  -h                    # --hist (note: conflicts with --help)
  -t "My Title"         # --title
  --no-cbar             # keep long for toggle
```

**Implementation:** Update clap definitions in `src/cli.rs` to add short aliases
```rust
#[arg(short = 'm', long)]
pub min: Option<f64>,

#[arg(short = 'M', long)]
pub max: Option<f64>,

#[arg(short = 'g', long)]
pub gamma: f64,
```

**Complexity:** Very Low
- Just adding `short = 'X'` to clap definitions
- No logic changes

**Timeline:** 1 day

**Blockers:** None - pure interface improvement

---

## Priority 4: MEDIUM - Multi-Map HDF Selection

**Issue:** Complex HDF files with multiple maps aren't easily selectable.

### Current Gap
```bash
# map2png
map2png -map 1 -map 2 input.hdf output.png  # Can specify multiple maps

# map2fig
./map2fig -f input.hdf -i 0 -o output.pdf   # Only one map dimension
```

### Proposed Implementation
```bash
# Stack multiple maps
./map2fig -f input.hdf -i 0 --map-index 0,1 -o stacked.pdf

# Or multi-panel with multiple maps
./map2fig -f input.hdf --map-index 0 --col 0,1 --layout 2x1 -o compare.pdf
```

**Implementation Strategy:**
1. Add `--map-index` parameter to CLI
2. Modify `load_data()` to load multiple HDF maps
3. Integrate with multi-panel system (depends on Priority 1)

**Complexity:** Low-Medium
- HDF reading logic already exists
- Need to handle multiple tables
- Depends on multi-panel implementation

**Timeline:** 3-5 days (after Priority 1)

**Blockers:** None

---

## Priority 5: MEDIUM - Column Name Support

**Issue:** Named FITS columns are more discoverable than numeric indices.

### Proposed Implementation
```bash
# By numeric index (current)
./map2fig -f input.fits -i 0 -o output.pdf

# By column name (new)
./map2fig -f input.fits --column-name "TEMPERATURE" -o output.pdf

# Multi-column by name
./map2fig -f input.fits --columns "TEMPERATURE","POLARIZATION" --layout 1x2 -o compare.pdf
```

**Implementation Strategy:**
1. Read FITS header to get column names
2. Add `--column-name` parameter
3. Map names to indices at load time
4. Fallback to numeric if name not found

**Complexity:** Medium
- Need to parse FITS header column names
- Name resolution / fuzzy matching?
- Error handling for missing columns

**Timeline:** 3-4 days

**Blockers:** None

---

## Priority 6: MEDIUM - Batch Processing Mode

**Issue:** Processing multiple files requires shell loops.

### Proposed Implementation
```bash
# Process file glob with common settings
./map2fig --batch "data/*.fits" \
  -o "output_{filename}.pdf" \
  --log --cmap plasma --graticule

# Or with pattern substitution
./map2fig --batch-dir data/ \
  --batch-pattern "*.fits" \
  --out-pattern "results/{basename}.pdf" \
  --settings common.toml
```

**Implementation Strategy:**
1. Add `--batch` mode to CLI
2. Pattern matching for input files
3. Template substitution for output names
4. Optional settings file (TOML) for common options

**Complexity:** Medium
- Glob pattern support
- Template engine for output names
- Settings file parsing

**Timeline:** 1 week

**Blockers:** Optional - nice-to-have feature

---

## Priority 7: NICE-TO-HAVE - Template/Preset System

**Issue:** Users repeat the same option combinations.

### Proposed Implementation
```bash
# Create preset
./map2fig --save-preset planck-log \
  --log --cmap "planck_r" --grat-coord gal --graticule --extend both

# Use preset
./map2fig -f input.fits -o output.pdf --preset planck-log

# Stored in ~/.map2fig/presets/ or ./map2fig.presets.toml
```

**Timeline:** 2 weeks

**Blockers:** Optional

---

## Priority 8: NICE-TO-HAVE - Interactive Mode

**Issue:** Tuning parameters requires many iterations.

### Proposed Implementation
```bash
./map2fig -f input.fits --interactive

# Opens TUI (terminal UI) with:
# - Real-time parameter adjustment
# - Live preview (if terminal supports)
# - Save/load presets
```

**Complexity:** High
- Requires TUI crate (ratatui, cursive)
- Real-time rendering
- Complex state management

**Timeline:** 3-4 weeks

**Blockers:** Lower priority - stretch goal

---

## Implementation Sequence

### Phase 1: Eliminate Critical Gap (Weeks 1-3)
1. **Multi-panel output** (Priority 1) - **Critical blocker**
   - Establish `--layout` syntax
   - Implement panel grid layout
   - Test with 2x1, 1x2, 2x2 configurations

### Phase 2: Quick Wins (Weeks 4-5)
2. **Quantile auto-scaling** (Priority 2)
3. **Short command flags** (Priority 3)

### Phase 3: Enhanced Workflow (Weeks 6-8)
4. **Multi-map HDF selection** (Priority 4)
5. **Batch processing** (Priority 6)

### Phase 4: Quality of Life (Weeks 9-10)
6. **Column name support** (Priority 5)
7. **Template/preset system** (Priority 7)

### Phase 5: Advanced (Future)
8. **Interactive mode** (Priority 8)

---

## Success Criteria

### By End of Phase 1
- ✅ Multi-panel output working for basic layouts
- ✅ map2png `-ncol` use case fully covered
- ✅ Documentation updated with examples

### By End of Phase 2
- ✅ Quantile scaling for robust auto-range
- ✅ Short flags for all common options
- ✅ `./map2fig` as discoverable as `map2png`

### By End of Phase 3
- ✅ All data input scenarios covered (HDF, FITS)
- ✅ Batch processing for high-volume work
- ✅ Performance > map2png on all datasets

### Final Status
- ✅ **map2png becomes obsolete**
- ✅ **Zero legitimate use cases remaining**
- ✅ **Complete superset of functionality**

---

## Testing Strategy

### Unit Tests
- Quantile calculation accuracy
- Layout grid generation
- HDF multi-map loading
- Preset TOML parsing

### Integration Tests
- Multi-panel rendering (compare image output)
- Batch processing file discovery
- Column name resolution
- Settings file application

### Benchmarks
- Multi-panel rendering time vs single panel × N
- File I/O for batch mode
- Memory usage for multi-map loading

### User Testing
- Survey map2png users on remaining pain points
- Beta test each feature before release
- Gather feedback on CLI discoverability

---

## Documentation Updates

### To be updated as features ship:
1. **README.md** - Add multi-panel examples
2. **tools/README.md** - Update comparison
3. **FEATURE_COMPARISON.md** - Mark completed gaps
4. **IMPLEMENTATION_GAPS.md** - Track progress
5. **docs/advanced.md** - Template/batch mode guide

---

## Breaking Changes to Avoid

Current stable interface should remain:
- ✅ `-f` / `--fits` for input
- ✅ `-o` / `--out` for output
- ✅ `-i` / `--col` for column selection
- ✅ All existing scaling options
- ✅ All existing projection options

New features should use new flags, never change existing behavior.

---

## References

- **FEATURE_COMPARISON.md** - Detailed feature matrix
- **IMPLEMENTATION_GAPS.md** - Original gap analysis
- **Benchmark Results** - benchmark_results/map2png_vs_map2fig/timings.csv

---

## Tracking

This roadmap should be updated as:
1. Features are started → mark 🔄
2. Features are completed → mark ✅
3. New gaps are discovered → add to list
4. Priorities change → update table

Revisit quarterly or when map2png user feedback arrives.
