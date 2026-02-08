# HEALPix Plotter: Issues & Features Tracker

**Last Updated:** 2026-02-07
**Status:** Active Development

---

## 🔴 CRITICAL ISSUES

### ✅ RESOLVED - Colorbar Extend Arrows Feature
- **Status:** COMPLETED (2026-02-08)
- **Feature:** Add arrows at colorbar ends indicating data extends beyond displayed range
- **Implementation Details:**
  - Added `--extend` CLI option with choices: `none` (default), `min`, `max`, `both`
  - Enum `Extend` in `src/cli.rs` with `FromStr` parsing
  - Arrow size scales proportionally with colorbar height: `(height * 0.4).max(4.0)`
  - Arrow colors taken from colormap endpoints (`sample(0.0)` and `sample(1.0)`)
  - Positioning offset: `arrow_size * 0.8` from colorbar edge
- **PNG Implementation:**
  - `draw_colorbar_extends()` function in `src/colorbar.rs` for PNG rendering
  - Three helper functions: `draw_triangle_left()`, `draw_triangle_right()`, `fill_triangle()`
  - Triangle fill using barycentric coordinate algorithm
  - Integrated into both Mollweide and Gnomonic PNG rendering paths
- **PDF Implementation:**
  - `draw_colorbar_pdf_extends()` function in `src/render/pdf.rs` for PDF rendering
  - Uses Cairo paths for triangle drawing and filling
  - Called after PDF colorbar labels are rendered
  - Works with both Mollweide and Gnomonic PDF output
- **Files Modified:**
  - `src/cli.rs`: Added Extend enum with FromStr implementation
  - `src/params.rs`: Added extend field to DisplayParams
  - `src/main.rs`: Parse extend argument and pass to both projection types (2 locations)
  - `src/colorbar.rs`: Added 4 new functions for PNG extend rendering
  - `src/plot.rs`: Integrated extend calls in mollweide PNG (line ~705) and gnomonic PNG; updated colorbar PDF calls
  - `src/render/pdf.rs`: Added PDF extend drawing function and integrated into draw_colorbar_pdf
- **Testing:**
  - Created test PNG files at widths 800, 1200, 1600px
  - Tested all extend options: none, min, max, both
  - Verified with multiple colormaps: default, plasma, viridis-r
  - Tested both PNG and PDF output formats
  - Verified arrow scaling works correctly across different widths
- **CLI Usage:**
  ```bash
  map2fig -f data.fits -w 800 --extend both -o map.png
  map2fig -f data.fits -w 800 --extend min -c plasma -o map.pdf
  ```

### ✅ RESOLVED - Text Scaling Not Working
- **Status:** FIXED (2026-02-07)
- **Issue:** Text and label sizes were not scaling with `--width` parameter
- **Root Cause:** Font size calculations in `compute_cbar_layout()` used colorbar width instead of image width
- **Fix Applied:** 
  - Fixed scale calculation to use `width / 1200.0` instead of `cbar_w / 1000.0`
  - Scaled units label Y-offset: `units_y = (cb_layout.tick_label_pad + 30.0 * scale)`
  - Adjusted font clamps from `.max(5.0)` to `.max(3.0)` for better scaling range
- **Verification:** 800px vs 1600px comparison shows 2.0x text scaling (height ratio exactly 2.0)
- **Files Modified:** src/layout.rs, src/plot.rs, src/render/pdf.rs

### ⚠️ PENDING - Tests Failing
- **Status:** NOT STARTED
- **Issue:** Unit tests fail due to `scale_value()` API changes
- **Details:** Missing `Option<&HistogramScale>` parameter in test calls
- **Files Affected:** src/scale.rs (likely)
- **Action Required:** Update all `scale_value()` calls in tests to include histogram scale parameter

---

## 🟡 HIGH PRIORITY ISSUES

### ⚠️ Unused Code & Imports
- **Status:** NOT STARTED
- **Issues:**
  - Unused imports throughout codebase (run `cargo fix` to identify)
  - Dead code in `colorbar.rs`: `apply_lightness()`, `sample_distortion()`
  - Unused variables in various modules
- **Action Required:** Clean up unused code; decide whether dead functions should be deleted
- **Benefit:** Reduce build warnings, improve code clarity

### ⚠️ Code Quality
- **Status:** NOT STARTED
- **Issues:**
  - Potential edge cases in edge coordinate handling
  - Performance optimizations possible in rendering pipeline
  - Error messages could be more user-friendly

---

## 🟢 FEATURES TO ADD

### High Priority - Compatibility & Integration

#### 1. Cosmoglobe.plot Integration
- **Status:** NOT STARTED
- **Description:** Full feature parity with Cosmoglobe's plotting module
- **Scope:**
  - [ ] Support for Cosmoglobe-specific data formats
  - [ ] Cosmoglobe color scheme library integration
  - [ ] Unit handling compatible with Cosmoglobe conventions
  - [ ] Output format options (FITS metadata preservation)
- **Files Involved:** src/plot.rs, src/colormap.rs
- **Dependencies:** May need Cosmoglobe API documentation

#### 2. Healpy.mollview Compatibility
- **Status:** NOT STARTED
- **Description:** Match key features of healpy's mollview function
- **Current Features Supported:**
  - [x] Basic Mollweide projection
  - [x] Colorbar with customizable scaling
  - [x] LaTeX unit labels
  - [x] Width/resolution control
- **Missing Features:**
  - [ ] Graticule lines (lat/lon grid)
  - [ ] Custom coordinate frames (Galactic, J2000, etc.)
  - [ ] Overlay support (points, vectors, contours)
  - [ ] Title/annotation support
  - [ ] Custom projection options
  - [ ] Rotation capability
  - [ ] Multiple sub-plots
- **Reference:** healpy/mollview.py
- **Priority:** Medium-High

#### 3. Map2PNG (map2fig) Enhancements
- **Status:** PARTIALLY IMPLEMENTED
- **Completed:**
  - [x] Basic PNG output
  - [x] LaTeX rendering for units
  - [x] Width-based scaling
  - [x] Colormap support (80+ maps)
  - [x] Data scaling options (linear, log, symlog, asinh, histogram)
  - [x] Colorbar extend arrows (--extend none/min/max/both) - ADDED 2026-02-08
- **Pending Features:**
  - [ ] Gnomonic projection improvements
  - [ ] Hammer projection (stub exists in src/hammer.rs)
  - [ ] Orthographic projection
  - [ ] Custom padding/margin control
  - [ ] Transparent background option
  - [ ] Batch processing mode
  - [ ] Progress indicators for large files
  - [ ] Memory efficiency improvements for huge maps (nside > 8192)

### Medium Priority - User Experience

#### 4. CLI Improvements
- **Status:** NOT STARTED
- **Features Needed:**
  - [ ] Better error messages with suggestions
  - [ ] Validation of input file existence
  - [ ] Warnings for unusual parameter combinations
  - [ ] Config file support (.map2figrc)
  - [ ] Batch processing from file list
  - [ ] Interactive CLI mode with preview
  - [ ] Help text with examples

#### 5. Documentation & Examples
- **Status:** PARTIALLY COMPLETE
- **Completed:**
  - [x] Basic README with build instructions
  - [x] Feature comparison document
  - [x] Integration test results
- **Pending:**
  - [ ] API documentation (Rustdoc)
  - [ ] Tutorial for common use cases
  - [ ] Cosmoglobe integration guide
  - [ ] Comparison with healpy/Cosmoglobe
  - [ ] Performance tuning guide

#### 6. Output Format Options
- **Status:** PARTIALLY COMPLETE
- **Completed:**
  - [x] PDF output (Cairo-based)
  - [x] PNG output (image crate)
  - [x] SVG intermediate (for LaTeX)
- **Pending:**
  - [ ] FITS output (preserve metadata)
  - [ ] High-res TIFF
  - [ ] WebP format
  - [ ] Output format auto-detection
  - [ ] Batch format conversion

### Low Priority - Advanced Features

#### 7. Advanced Projections
- **Status:** NOT STARTED
- **Projections to Implement:**
  - [ ] Hammer-Aitoff (stub exists)
  - [ ] Orthographic
  - [ ] Lambert Conformal
  - [ ] Stereographic
  - [ ] Azimuthal Equidistant
- **Files:** src/projection.rs, src/mollweide.rs

#### 8. Interactive Features
- **Status:** NOT STARTED
- **Features:**
  - [ ] Web-based viewer
  - [ ] Zoom/pan capability
  - [ ] Value inspection tool
  - [ ] Real-time colormap preview
  - [ ] Statistics display

#### 9. Performance Optimization
- **Status:** NOT STARTED
- **Opportunities:**
  - [ ] Parallel pixel rendering
  - [ ] GPU acceleration investigation
  - [ ] Memory pooling for large maps
  - [ ] Caching of frequently used colormaps
  - [ ] Streaming processing for very large FITS files

---

## 📋 COMPARISON MATRIX

### vs. healpy.mollview

| Feature | map2fig | healpy | Priority |
|---------|---------|--------|----------|
| Mollweide projection | ✅ | ✅ | Core |
| Colorbar | ✅ | ✅ | Core |
| Log scaling | ✅ | ✅ | Core |
| LaTeX units | ✅ | ✅ | High |
| Custom colormaps | ✅ | ✅ | High |
| Graticule | ❌ | ✅ | High |
| Overlays (points/vectors) | ❌ | ✅ | Medium |
| Coordinate frames | ❌ | ✅ | Medium |
| Multiple subplots | ❌ | ✅ | Low |
| PDF output | ✅ | ❌ | High |
| Resolution scaling | ✅ | ❌ | High |

### vs. Cosmoglobe.plot

| Feature | map2fig | Cosmoglobe | Priority |
|---------|---------|-----------|----------|
| Sky map rendering | ✅ | ✅ | Core |
| FITS I/O | ✅ | ✅ | Core |
| Colormaps | ✅ | ✅ | High |
| Units handling | ⚠️ | ✅ | High |
| Output formats | ✅ | ✅ | High |
| Batch processing | ❌ | ✅ | Medium |
| Web integration | ❌ | ✅ | Low |

---

## 🔧 Technical Debt

- [ ] Refactor coordinate system handling
- [ ] Consolidate font size calculations (currently in layout.rs, plot.rs, pdf.rs)
- [ ] Improve error handling and validation
- [ ] Add compile-time checks for color space conversions
- [ ] Document internal architecture
- [ ] Create integration test suite

---

## 📝 NOTES FOR NEW CHAT

If this chat session ends, use this document to continue:

1. **Current State:** See commit history and latest build status
2. **Active Issues:** Review the "CRITICAL ISSUES" section above
3. **Next Steps:** Typically work on HIGH PRIORITY issues before MEDIUM/LOW
4. **Context:** This is a Rust CLI for HEALPix sky map visualization
5. **Key Files:**
   - Core: `src/plot.rs`, `src/layout.rs`, `src/render/`
   - Data: `src/healpix.rs`, `src/fits.rs`
   - Styling: `src/colormap.rs`, `src/colorbar.rs`
6. **Testing:** Run `cargo test` and `cargo build --release` frequently
7. **Verification:** Use test images in `/tmp/` for visual comparison

---

## 🚀 Quick Status Summary

- **Overall Status:** ✅ Functional with most core features working
- **Latest Work:** Fixed text scaling (width-based font sizing)
- **Build Status:** ✅ Compiling successfully
- **Test Status:** ⚠️ Some unit tests failing (scale_value API)
- **Next Focus:** Either fix tests or implement Cosmoglobe integration

