# Refactoring Opportunities

## Overview
The codebase has several functions with many parameters (8-27 parameters) that would benefit from parameter bundling via structs. This document outlines the approach.

## Problem
Several key functions currently accept many individual parameters, making them difficult to maintain and extending:

- `plot_mollweide_auto()` - 25 parameters
- `plot_mollweide_pdf()` - 25 parameters  
- `plot_mollweide_png()` - 25 parameters
- `plot_gnomonic_auto()` - 27 parameters
- `plot_gnomonic_pdf()` - 27 parameters
- `plot_gnomonic_png()` - 27 parameters
- `render_mollweide_pixels()` - 13 parameters
- `render_projection_to_grid()` - 12 parameters
- Other visualization and rendering functions - 8-10 parameters

## Solution: Parameter Bundling with Structs

Group related parameters into logical structs. This reduces function signatures and improves maintainability.

### Proposed Struct Organization

#### 1. Core Plot Parameters
```rust
pub struct PlotData<'a> {
    pub map: &'a [f64],
    pub width: u32,
    pub filename: &'a str,
}
```

#### 2. Scale and Color Parameters
```rust
pub struct ScaleParams {
    pub minv: Option<f64>,
    pub maxv: Option<f64>,
    pub gamma: f64,
    pub scale: Scale,
    pub neg_mode: NegMode,
}

pub struct ColorParams {
    pub cmap: &'static Colormap,
    pub bad_color: Rgba<u8>,
    pub bg_color: Rgba<u8>,
}
```

#### 3. Display Parameters
```rust
pub struct DisplayParams {
    pub show_colorbar: bool,
    pub transparent: bool,
    pub draw_border: bool,
    pub latex_rendering: bool,
    pub units: Option<String>,
}
```

#### 4. Graticule Parameters
```rust
pub struct GraticuleParams {
    pub show_graticule: bool,
    pub grat_coord: Option<CoordSystem>,
    pub grat_overlay: Option<CoordSystem>,
    pub overlay_color: Rgba<u8>,
    pub show_labels: bool,
    pub dpar_deg: f64,
    pub dmer_deg: f64,
}
```

#### 5. Mollweide-Specific Parameters
```rust
pub struct MollweideParams<'a> {
    pub plot: PlotData<'a>,
    pub scale: ScaleParams,
    pub color: ColorParams,
    pub display: DisplayParams,
    pub graticule: GraticuleParams,
    pub meta: HealpixMeta,
    pub view: &'a ViewTransform,
}
```

#### 6. Gnomonic-Specific Parameters
```rust
pub struct GnomonicParams<'a> {
    pub plot: PlotData<'a>,
    pub scale: ScaleParams,
    pub color: ColorParams,
    pub display: DisplayParams,
    pub graticule: GraticuleParams,
    pub meta: HealpixMeta,
    pub view: &'a ViewTransform,
    pub lon_deg: f64,
    pub lat_deg: f64,
    pub fov_arcmin: f64,
    pub resolution_arcmin: f64,
    pub roll_deg: f64,
    pub grat_line_width: u32,
}
```

### Implementation Strategy

1. **Create the structs** in a new module `src/params.rs`
2. **Update main.rs** to construct these structs from CLI arguments
3. **Refactor plotting functions** to accept struct parameters instead of individual parameters
4. **Update all call sites** to use the bundled parameters
5. **Run tests** to ensure identical rendering

### Benefits

- **Reduced function signatures** from 25+ parameters to 1-2
- **Easier to extend** - add new parameters without changing function signatures
- **Better documentation** - struct names clarify parameter groupings
- **Type safety** - related parameters are grouped and validated together
- **Maintainability** - cleaner code with logical organization

### Impact Assessment

- **No change to rendering logic** - structs are just organizational
- **PNG outputs remain deterministic** - same computation
- **PDF outputs retain behavior** - Cairo rendering unchanged
- **Tests unaffected** - only parameter passing changes

### Estimated Effort

- **Phase 1**: Create parameter structs (~200 lines)
- **Phase 2**: Refactor main.rs to construct structs (~50 lines)
- **Phase 3**: Update plot_mollweide_* functions (~200 lines)
- **Phase 4**: Update plot_gnomonic_* functions (~200 lines)
- **Phase 5**: Update helper functions (~150 lines)
- **Phase 6**: Testing and verification (~50 lines)

**Total**: ~800 lines of changes, with significant improvement in code clarity.

### Risk Mitigation

- All changes are internal refactoring - no public API changes
- Comprehensive test suite validates identical output
- Staged approach allows incremental verification
- Can revert any phase if issues arise
