# Fuzzing & Property-Based Testing Guide

This guide covers advanced testing techniques for map2fig: **fuzzing** and **property-based testing**. These catch edge cases that traditional unit tests miss.

## Overview

### Property-Based Testing (proptest)
Tests verify mathematical properties that should ALWAYS hold, across thousands of randomly-generated inputs:

```
Property: "If value < min, scaled result should be ≤ 0.0"
proptest generates 256 different (value, min, max) combinations
All 256 pass → Property verified for this range
```

**Advantages**:
- Automatically finds edge cases humans miss
- Catches off-by-one errors, boundary conditions
- Documents implicit contracts (scaling always outputs [0, 1])
- Fast feedback loop (256+ test cases per property)

### Fuzzing (cargo-fuzz)
Feeds arbitrary binary data to functions to find crashes:

```
Input: [random bytes]
↓ Fuzzer extracts f64 values and calls scale_value()
↓ If it crashes → bug found! Fuzzer saves that input
↓ If OK → generate new random input
↓ Repeat 10,000+ times
```

**Advantages**:
- Finds panics, memory issues, DoS vectors
- Effective at discovering invalid state
- Continuous CI/CD integration
- Finds bugs no one would think of

---

## 1. Property-Based Testing with proptest

### Running Property Tests

```bash
# Run all property tests
cargo test --test property_tests --

# Run a specific property test (verbose output)
cargo test --test property_tests prop_linear_scaling_bounded -- --nocapture

# Run multiple times (good for flaky tests)
for i in {1..100}; do cargo test --test property_tests --; done
```

### Test Categories

#### Scaling Properties (23 tests)
- **Bounds**: All scaling modes produce [0.0, 1.0]
- **Endpoints**: min → 0.0, max → 1.0
- **Monotonicity**: If v1 < v2 then scaled(v1) ≤ scaled(v2)
- **Clamping**: Values outside [min, max] clamp correctly
- **Extremes**: NaN, Infinity always → Bad
- **Edge cases**: min == max, negative ranges, tiny ranges

**Running bounds tests only**:
```bash
cargo test --test property_tests prop_scaling
```

#### Colormap Properties (2 tests)
- **Sampling**: All colormaps work without panic across [0, 1]
- **Availability**: All 80+ colormaps are valid

```bash
cargo test --test property_tests prop_colormap
```

### Example: Understanding a Property Test

```rust
/// Property: Linear scaling always produces normalized output in [0.0, 1.0]
#[test]
fn prop_linear_scaling_bounded() {
    proptest!(|(
        value in -1e6f64..1e6f64,      // Generate random values
        min in -1e6f64..0f64,          // Generate random min
        max in 0f64..1e6f64,           // Generate random max
    )| {
        let result = scale_value(value, min, max, Scale::Linear, NegMode::Zero, None);

        if let PixelValue::Color(t) = result {
            // Property: output must be in [0, 1]
            prop_assert!(
                t >= 0.0 && t <= 1.0,
                "Linear scale produced out-of-bounds value: {}",
                t
            );
        }
    });
}
```

**What happens**:
1. proptest generates 256 random combinations of (value, min, max)
2. Each combination is tested
3. If any fails the assertion, test fails with that exact case
4. If all 256 pass, property is verified ✅

### Adding New Properties

Example: Test that log scaling preserves value ordering:

```rust
/// Property: Log scaling maintains value ordering
#[test]
fn prop_log_monotonic() {
    proptest!(|(
        v1 in 1e-6f64..1e6f64,
        v2 in 1e-6f64..1e6f64,
        min in 1e-6f64..1e3f64,
        max in 1e3f64..1e6f64,
    )| {
        let (v1, v2) = if v1 <= v2 { (v1, v2) } else { (v2, v1) };

        let r1 = scale_value(v1, min, max, Scale::Log, NegMode::Zero, None);
        let r2 = scale_value(v2, min, max, Scale::Log, NegMode::Zero, None);

        if let (PixelValue::Color(t1), PixelValue::Color(t2)) = (r1, r2) {
            prop_assert!(
                t1 <= t2 + 1e-10,  // Allow small floating-point error
                "Log scale violated monotonicity"
            );
        }
    });
}
```

Add to `tests/property_tests.rs` and run:
```bash
cargo test --test property_tests prop_log_monotonic
```

---

## 2. Fuzzing with cargo-fuzz

Fuzzing requires Rust nightly. Install:

```bash
# Install nightly (one-time)
rustup toolchain install nightly
rustup component add rustfmt --toolchain nightly

# Install cargo-fuzz 
cargo install cargo-fuzz
```

### Running Fuzzing Targets

```bash
# Fuzz scale_value (finds if it ever panics)
cd fuzz && cargo +nightly fuzz run fuzz_scale_value

# Fuzz HEALPix projections
cd fuzz && cargo +nightly fuzz run fuzz_projection

# Fuzz FITS file parsing
cd fuzz && cargo +nightly fuzz run fuzz_fits_parsing
```

**Control execution**:
```bash
# Run for 10 seconds
timeout 10 cargo +nightly fuzz run fuzz_scale_value

# Run with 4 workers (parallel)
cargo +nightly fuzz run -j 4 fuzz_scale_value

# Run with limited memory (prevent DoS)
cargo +nightly fuzz run -m 512 fuzz_scale_value
```

### Fuzzing Targets

#### `fuzz_scale_value`
- **Input**: 25+ bytes (three f64 values + mode selector)
- **Tests**: Does scale_value ever panic with arbitrary (value, min, max, scale, mode)?
- **Catches**: Panics, divisions by zero, stack overflows

```bash
# Run continuously until crash or Ctrl+C
cd fuzz && cargo +nightly fuzz run fuzz_scale_value
```

#### `fuzz_projection`
- **Input**: 24 bytes (x, y, z vector coordinates)
- **Tests**: Do projection functions handle any vector?
- **Catches**: Panics on NaN, Inf, zero vectors

#### `fuzz_fits_parsing`
- **Input**: Arbitrary binary data
- **Tests**: Never panic on malformed FITS
- **Catches**: Memory issues, panics on corrupt files

### Understanding Fuzzing Output

```
$ cd fuzz && cargo +nightly fuzz run fuzz_scale_value
INFO: Seed: ...
INFO: Start fuzzing
... iteration 1000
... iteration 5000  (grows exponentially after finding new paths)
... iteration 50000
^C
⏱  20s, ~50K testsINFO: exiting: part of fuzz target
```

**Good**: Runs for a long time without crashing
**Bad**: Crashes with a saved `artifacts/fuzz_scale_value/crash-*` file

### If Fuzzing Finds a Crash

```bash
# Reproduces the crash every time
cargo +nightly fuzz run fuzz_scale_value artifacts/fuzz_scale_value/crash-abc123def

# Fix the bug, then verify it's fixed
# (fuzzer automatically tests the crash input again)
```

### Writing New Fuzz Targets

Example: Fuzz colormap sampling

```rust
// fuzz/fuzz_targets/fuzz_colormap.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use map2fig::get_colormap;

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    // Extract a float from bytes
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[0..8]);
    let t = f64::from_le_bytes(buf);

    // Pick a colormap
    let colormap_idx = data.get(8).unwrap_or(&0) as usize % 3;
    let cmap_names = ["viridis", "plasma", "inferno"];
    let cmap = get_colormap(cmap_names[colormap_idx]);

    // Should never panic
    let _ = cmap.sample(t);
});
```

Add to `fuzz/Cargo.toml`:
```toml
[[bin]]
name = "fuzz_colormap"
path = "fuzz_targets/fuzz_colormap.rs"
test = false
doc = false
```

Run:
```bash
cd fuzz && cargo +nightly fuzz run fuzz_colormap
```

---

## 3. CI/CD Integration

### GitHub Actions Workflow

Add to `.github/workflows/fuzzing.yml`:

```yaml
name: Fuzzing

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    # Run nightly fuzzing
    - cron: "0 2 * * *"

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - uses: dtolnay/rust-toolchain@nightly
      
      - run: cargo install cargo-fuzz
      
      - run: cd fuzz && cargo +nightly fuzz run fuzz_scale_value -- -max_len=100 -max_total_time=60
      - run: cd fuzz && cargo +nightly fuzz run fuzz_projection -- -max_len=100 -max_total_time=60
      - run: cd fuzz && cargo +nightly fuzz run fuzz_fits_parsing -- -max_len=1000 -max_total_time=60
      
      - name: Run property tests
        run: cargo test --test property_tests
```

### Local Pre-Commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

echo "Running property tests..."
cargo test --test property_tests --

echo "Running 30s fuzz on scale_value..."
timeout 30 cargo +nightly fuzz run fuzz_scale_value || true
```

Make executable:
```bash
chmod +x .git/hooks/pre-commit
```

---

## 4. Best Practices

### Property Tests
- ✅ Test mathematical properties, not implementation details
- ✅ Use ranges that include edge cases (NaN, Inf, 0, negative)
- ✅ Add comments explaining what property is being tested
- ❌ Don't write "unit tests in disguise" (just testing one value)
- ❌ Don't make assertions that depend on order of execution

### Fuzzing
- ✅ Start simple (single function fuzz targets)
- ✅ Run for extended periods (kills DoS bugs)
- ✅ Save crash artifacts for debugging
- ✅ Use in CI but accept failures (fuzzing is inherently unlimited)
- ❌ Don't use fuzzing to replace unit tests (too slow)
- ❌ Don't ignore crashes - they're real bugs!

---

## 5. Troubleshooting

### Property Tests Won't Compile
```
error[E0433]: failed to resolve: could not find `Xyz` in scope
```
Make sure you `pub use` the symbol in `src/lib.rs`:
```rust
pub use your_module::Xyz;
```

### Fuzzing Requires Nightly
```
error: nightly features cannot be used with stable
```
Use `cargo +nightly`:
```bash
cargo +nightly fuzz run fuzz_scale_value
```

### Fuzzing Takes Too Long
Constrain execution:
```bash
# Max 10 seconds
timeout 10 cargo +nightly fuzz run fuzz_scale_value

# Max 512 MB memory
cargo +nightly fuzz run -m 512 fuzz_scale_value

# Both constraints
timeout 30 cargo +nightly fuzz run -m 256 fuzz_scale_value
```

### Property Test Flaking
If a property test sometimes passes and sometimes fails:
```bash
# Run multiple times to isolate
for i in {1..100}; do echo "Run $i"; cargo test --test property_tests || break; done
```

---

## 6. Performance Notes

| Technique | Setup | Runtime | Coverage |
|-----------|-------|---------|----------|
| Unit tests | Minutes | Fast (ms) | Explicit |
| Property tests | Minutes | Slow (256+ cases) | Automatic |
| Fuzzing | Hours | Very slow (mins+) | Maximum |
| All combined | Full | Comprehensive | 100% |

**Recommendation**: Use all three in CI:
1. Unit tests for basic functionality (fast)
2. Property tests for mathematical properties (medium)
3. Fuzzing for robustness (slow, run nightly)

---

## References
- [proptest book](https://docs.rs/proptest/latest/proptest/)
- [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [Fuzzing guide](https://github.com/google/fuzzing/blob/master/docs/structure-aware-fuzzing.md)
