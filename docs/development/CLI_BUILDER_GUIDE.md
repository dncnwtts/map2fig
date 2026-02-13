# CLI Builder Module - Developer Guide

## Overview

The `cli_builder` module (`src/cli_builder.rs`) contains utilities for constructing projection parameters from command-line arguments. It bridges the gap between raw CLI input and the rich parameter types expected by the projection rendering functions.

## Module Structure

### Public Functions

All functions are documented with rustdoc. Access these from `map2fig::cli_builder::*` or `crate::cli_builder::*` depending on context.

```
cli_builder/
├── create_pixel_mask()           [30 lines] - Unified mask creation
├── resolve_overlay_color()       [8 lines]  - Overlay color resolution
├── resolve_graticule_coord()     [11 lines] - Coordinate system resolution
├── parse_overlay_coord()         [6 lines]  - String → CoordSystem parsing
├── build_mollweide_params()      [60 lines] - MollweideParams construction
├── build_gnomonic_params()       [60 lines] - GnomonicParams construction
└── build_hammer_params()         [60 lines] - HammerParams construction
```

## When to Use Each Function

### `create_pixel_mask()`

**Purpose:** Build a PixelMask from CLI arguments  
**When to use:**
- User specified `--mask-below`, `--mask-above`, or `--mask-file`
- Any time you need the mask before rendering

**Example:**
```rust
let mask = cli_builder::create_pixel_mask(&args, &data, verbose)?;
// Returns: Option<PixelMask>
```

**Error cases:**
- Invalid FITS file path
- Coordinate system parsing failure (handled gracefully)

### `resolve_overlay_color()`

**Purpose:** Determine overlay color for graticule lines  
**When to use:**
- Building graticule parameters with overlay
- Any time you need overlay color

**Example:**
```rust
let color = cli_builder::resolve_overlay_color(args)?;
```

**Returns:**
- If `--grat-coord-overlay` is set: parsed color with 200 alpha
- Otherwise: transparent yellow `Rgba([255, 255, 0, 0])`

### `resolve_graticule_coord()`

**Purpose:** Resolve graticule coordinate system  
**When to use:**
- Building full-sky projection parameters (Mollweide, Hammer)
- Determining what coordinate system to show graticule in

**Example:**
```rust
let grat_coord = cli_builder::resolve_graticule_coord(args, data.meta.coord);
// Returns: Option<CoordSystem>
```

**Logic:**
1. If `--graticule` is disabled → return `None`
2. If `--grat-coord` is explicitly provided → use it
3. Otherwise → use data's native coordinate system

### `parse_overlay_coord()`

**Purpose:** Parse overlay coordinate system string  
**When to use:**
- Extracting `--grat-coord-overlay` value
- Generally called from within `build_*_params()` functions

**Example:**
```rust
let overlay = args.grat_coord_overlay.as_ref()
    .map(|s| cli_builder::parse_overlay_coord(s));
```

**Panics:** If coordinate system is invalid (internal validation)

### `build_mollweide_params()`

**Purpose:** Construct complete MollweideParams from CLI args  
**When to use:**
- User selected `--projection mollweide`
- Building parameters for full-sky Mollweide rendering

**Example:**
```rust
let params = cli_builder::build_mollweide_params(
    &args,
    &data,
    &config,
    &view,
    mask
)?;
plot_mollweide_auto(params);
```

**Parameters:**
- `args: &Args` - Parsed command-line arguments
- `data: &ProcessedData` - Loaded map data with metadata
- `config: &PlotConfig` - Resolved color/scale/rendering config
- `view: &ViewTransform` - Rotation transform (if any)
- `mask: Option<PixelMask>` - Computed pixel mask (if any)

**Returns:** `Result<MollweideParams<'a>, String>`

### `build_gnomonic_params()`

**Purpose:** Construct complete GnomonicParams from CLI args  
**When to use:**
- User selected `--projection gnomonic`
- Building parameters for zoomed Gnomonic rendering

**Note:** Gnomonic settings (lon, lat, FOV, resolution) are extracted from args and applied here

**Example:**
```rust
let params = cli_builder::build_gnomonic_params(
    &args,
    &data,
    &config,
    &view,
    mask
)?;
plot_gnomonic_auto(params);
```

### `build_hammer_params()`

**Purpose:** Construct complete HammerParams from CLI args  
**When to use:**
- User selected `--projection hammer`
- Building parameters for Hammer-Aitoff projection rendering

**Example:**
```rust
let params = cli_builder::build_hammer_params(
    &args,
    &data,
    &config,
    &view,
    mask
)?;
plot_hammer_auto(params);
```

## Common Patterns

### Pattern 1: Using in main.rs

```rust
use map2fig::cli_builder;

fn run() -> Result<(), String> {
    let args = Args::parse();
    // ... load data ...
    
    // Create mask once
    let mask = cli_builder::create_pixel_mask(&args, &data, args.verbose)?;
    
    // Route to appropriate builder
    match args.projection.to_lowercase().as_str() {
        "mollweide" => {
            let params = cli_builder::build_mollweide_params(&args, &data, &config, &view, mask)?;
            plot_mollweide_auto(params);
        }
        // ... other projections ...
    }
    
    Ok(())
}
```

### Pattern 2: Adding a new projection

If you're adding a new projection (e.g., "stereographic"):

1. **Create projection module:** `src/stereographic.rs`
2. **Add to lib.rs:** `pub mod stereographic;`
3. **Add builder function to cli_builder.rs:**

```rust
pub fn build_stereographic_params<'a>(
    args: &'a Args,
    data: &'a ProcessedData,
    config: &'a crate::cli::PlotConfig,
    view: &'a ViewTransform,
    mask: Option<PixelMask>,
) -> Result<StereographicParams<'a>, String> {
    // Similar structure to build_gnomonic_params
    let grat_overlay = args.grat_coord_overlay.as_ref().map(|s| parse_overlay_coord(s));
    let overlay_color = resolve_overlay_color(args)?;
    
    Ok(StereographicParams {
        plot: PlotData { ... },
        scale: ScaleParams { ... },
        // ... other fields ...
    })
}
```

4. **Update main.rs match arm:**

```rust
"stereographic" => {
    let params = cli_builder::build_stereographic_params(&args, &data, &config, &view, mask)?;
    plot_stereographic_auto(params);
}
```

### Pattern 3: Adding a new CLI argument

If adding a new mask type (e.g., `--mask-polygon`):

1. **Add field to Args:** `src/cli.rs`
   ```rust
   #[arg(long)]
   pub mask_polygon: Option<String>,
   ```

2. **Update `create_pixel_mask()`:** Add new conditional branch
   ```rust
   if let Some(ref polygon_file) = args.mask_polygon {
       // Load polygon mask ...
       return Ok(Some(mask));
   }
   ```

3. **Update documentation:** Add examples to rustdoc

### Pattern 4: Sharing logic between builders

If multiple builders need the same logic, extract to a helper:

```rust
fn build_display_params(args: &Args, config: &PlotConfig, mask: Option<PixelMask>) -> DisplayParams {
    DisplayParams {
        show_colorbar: !args.no_cbar,
        transparent: args.transparent,
        // ... common fields ...
    }
}

// Then in each builder:
pub fn build_mollweide_params<'a>(
    args: &'a Args,
    // ...
) -> Result<MollweideParams<'a>, String> {
    let display = build_display_params(args, config, mask);
    Ok(MollweideParams {
        display,
        // ... projection-specific fields ...
    })
}
```

## Error Handling Convention

All public functions use `Result` types for proper error propagation:

```rust
// Good: Propagates error
pub fn create_pixel_mask(...) -> Result<Option<PixelMask>, String> { ... }

// Usage:
let mask = cli_builder::create_pixel_mask(&args, &data, verbose)?;
```

**Not:**
```rust
// Avoid using .expect() - let caller decide how to handle
.expect("Invalid color") // ❌ Panics, not recovered gracefully
```

## Type Lifetime Annotations

Builders use lifetime `'a` to tie parameters to the input argument references:

```rust
pub fn build_mollweide_params<'a>(
    args: &'a Args,
    data: &'a ProcessedData,
    config: &'a PlotConfig,
    view: &'a ViewTransform,
    // ...
) -> Result<MollweideParams<'a>, String>
```

This ensures parameters don't outlive the arguments they reference.

## Testing Guidelines

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_overlay_color_with_overlay() {
        let mut args = Args { /* ... */ };
        args.grat_coord_overlay = Some("galactic".to_string());
        
        let color = resolve_overlay_color(&args).unwrap();
        assert!(color.0[3] > 0); // Should have alpha
    }

    #[test]
    fn test_resolve_overlay_color_without_overlay() {
        let args = Args { /* default */ };
        assert!(args.grat_coord_overlay.is_none());
        
        let color = resolve_overlay_color(&args).unwrap();
        assert_eq!(color.0[3], 0); // Transparent
    }
}
```

### Integration Testing

```rust
#[test]
fn test_build_mollweide_params_full_flow() {
    let args = Args { /* populated with test values */ };
    let data = load_test_fits_data();
    let config = PlotConfig::default();
    let view = ViewTransform::identity();
    
    let params = build_mollweide_params(&args, &data, &config, &view, None).unwrap();
    
    assert_eq!(params.plot.width, args.width);
    assert_eq!(params.meta.nside, data.meta.nside);
}
```

## Documentation Standards

All public functions should have rustdoc with:

```rust
/// Brief one-liner description.
///
/// Longer description explaining:
/// - What the function does
/// - When to use it
/// - Major side effects (if any)
///
/// # Arguments
///
/// * `arg1` - Description of arg1
/// * `arg2` - Description of arg2
///
/// # Returns
///
/// Description of return type
///
/// # Errors
///
/// Describes error conditions
///
/// # Panics
///
/// Describes when/why it panics (if applicable)
///
/// # Example
///
/// ```ignore
/// let mask = create_pixel_mask(&args, &data, false)?;
/// ```
pub fn function_name(...) -> Result<T, String> { ... }
```

## Performance Considerations

- All functions are zero-cost abstractions (no allocations beyond what's necessary)
- Parameter building is O(1) - just struct initialization
- Mask creation is O(n_pixels) as data must be read from FITS
- No unnecessary clones (use lifetimes where appropriate)

## Compatibility Notes

- **Maintain backward compatibility**: Don't change function signatures
- **Add new functions** rather than modify existing ones
- **Deprecation path**: If a function must change, add new variants alongside old ones
- **Update documentation** when behavior changes

## Common Issues & Solutions

### Issue: "Explicit lifetime required"

**Symptom:** Compiler error about missing `'a` lifetime

**Solution:** Ensure all reference parameters that return data are annotated with `'a`:
```rust
// Wrong:
pub fn build_params(&Args, &ProcessedData) -> MollweideParams<'a>

// Right:
pub fn build_params<'a>(&'a Args, &'a ProcessedData) -> MollweideParams<'a>
```

### Issue: "Cannot move out of borrowed reference"

**Symptom:** Compiler error about `.clone()` being missing

**Solution:** Clone owned types when extracting from borrowed container:
```rust
// Wrong:
units: config.units,  // Can't move out of &config

// Right:
units: config.units.clone(),
```

### Issue: Parameter type mismatch

**Symptom:** Function expects different parameter type than builder provides

**Solution:** Check the params struct definition and ensure all fields are initialized correctly. Use IDE type hints and compiler messages.

## Version History

- **v1.0** (Initial): Three builders (Mollweide, Gnomonic, Hammer)
- Future: Additional builders for new projections

---

## Related Documentation

- [REFACTORING_SUMMARY.md](REFACTORING_SUMMARY.md) - High-level overview of changes
- [REFACTORING_CODE_COMPARISON.md](REFACTORING_CODE_COMPARISON.md) - Before/after code examples
- [src/params.rs](src/params.rs) - Parameter struct definitions
- [src/cli.rs](src/cli.rs) - CLI argument definitions

