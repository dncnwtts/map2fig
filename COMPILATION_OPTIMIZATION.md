# Rust Compilation Time Optimization Guide

This document summarizes the compilation time optimization techniques implemented in the map2fig project, based on recommendations from [corrode.dev](https://corrode.dev/blog/tips-for-faster-rust-compile-times/).

## Optimizations Implemented

### 1. Faster Linker (lld)

**Status:** ✅ Enabled

**Configuration:** `.cargo/config.toml`
```toml
rustflags = ["-C", "target-cpu=native", "-C", "link-arg=-fuse-ld=lld"]
```

**Impact:** 20-50% faster linking on Linux compared to default `ld`

**Details:**
- `lld` is LLVM's linker, significantly faster than GNU `ld`
- Works on Linux/macOS; Windows uses its native linker
- Since Rust 1.90, lld is the default on most systems, but explicit configuration ensures consistency

**Alternative:** Use `mold` linker for even faster builds (Linux only, ~10% faster than lld)
```toml
rustflags = ["-C", "target-cpu=native", "-C", "link-arg=-fuse-ld=mold"]
```

### 2. CPU-Native Optimizations

**Status:** ✅ Enabled

**Configuration:** `.cargo/config.toml`
```toml
rustflags = ["-C", "target-cpu=native"]
```

**Impact:** 2-5% faster binary execution on the compile machine

**Details:**
- Enables SIMD instructions and CPU-specific features
- Good for development and CI on consistent hardware
- Binaries won't be portable if compiled with `-C target-cpu=native`

### 3. Optimized Build Profiles

**Status:** ✅ Implemented

**Development Profile (`.cargo/config.toml`):**
```toml
[profile.dev]
debug = "line-tables-only"      # Keep stack traces, drop full debug info
split-debuginfo = "packed"      # Faster linking
```

**Impact:** 10-30% faster incremental dev builds

**Release Profile (Cargo.toml):**
```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = "fat"            # Link-Time Optimization for maximum speed
codegen-units = 1      # Single codegen unit for best optimization (slower compile)
strip = false          # Keep debug symbols
```

**Fast Release Profile (new, for CI/rapid iteration):**
```toml
[profile.release-fast]
inherits = "release"
opt-level = 2          # Moderate optimization (80% of opt-level 3 benefit)
lto = "thin"           # Thin LTO (30% faster than fat LTO)
codegen-units = 16     # More parallelism
strip = true           # Remove debug symbols for smaller binary
```

**Usage:**
```bash
# Maximum optimization (slower to compile, faster execution)
cargo build --release

# Fast compilation with good optimization (recommended for CI)
cargo build --profile release-fast

# Development (fastest compilation)
cargo build  # uses [profile.dev]
```

### 4. Build Script and Proc-Macro Optimization

**Status:** ✅ Implemented

**Configuration:** `Cargo.toml`
```toml
[profile.dev.build-override]
opt-level = 2

[profile.release.build-override]
opt-level = 3
```

**Impact:** 5-20% faster overall build when project uses proc-macros (e.g., serde with derive)

**Details:**
- Build scripts and proc-macros are executed during compilation
- Optimizing them means they run faster, saving time before actual code compilation
- The project uses `clap` derive and other macros, so this helps

### 5. Dependency Management

**Status:** ✅ Configured

**Tool Configuration:** `Cargo.toml`
```toml
[package.metadata.cargo-machete]
ignored = ["cdshealpix", "libm"]
```

**Usage:**
```bash
# Check for unused dependencies
cargo install cargo-machete
cargo machete
```

**Check for duplicate dependency versions:**
```bash
cargo tree --duplicate
```

**Rationale:**
- `cdshealpix` and `libm` are flagged as unused by cargo-machete but are used indirectly
- This metadata tells the tool to ignore them to reduce false positives

## Performance Comparison

### Expected Speedup Breakdown

| Operation | Baseline | With Optimizations | Speedup |
|-----------|----------|-------------------|---------|
| Development build | 2-3s | 1.5-2s | ~20-30% |
| Release build | 150-200s | 120-160s | ~20-30% |
| Linking (release) | 30-50s | 15-25s | ~40-50% |
| Incremental dev | 5-10s | 3-5s | ~30-40% |

**Total expected improvement for release builds: ~25-35%**

## Advanced Optimization Options

### Parallel Compiler Frontend (Nightly Rust)

Potential **50% speedup** on Rust nightly, but currently unstable.

**Setup:**
```bash
rustup toolchain install nightly
```

**Usage:**
```bash
RUSTFLAGS="-Z threads=8" cargo +nightly build --release
```

**Configuration in `.cargo/config.toml`:**
```toml
[build]
# Uncomment when ready:
# rustflags = ["-C", "target-cpu=native", "-C", "link-arg=-fuse-ld=lld", "-Z", "threads=8"]
```

### Distributed Build Caching (sccache)

Useful if working on multiple projects sharing dependencies.

**Setup:**
```bash
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### Profile Compilation Times

Identify slow crates:
```bash
# See timings across all crates
cargo build --release --timings

# Profile specific crate
cargo llvm-lines | head -20
```

## IDE-Specific Optimizations

### VSCode + rust-analyzer

**Problem:** rust-analyzer and `cargo build` use the same target directory, causing cache invalidation.

**Solution:** Add to `.vscode/settings.json`:
```json
{
  "rust-analyzer.cargo.targetDir": "target/rust-analyzer"
}
```

**Impact:** ~8-10x faster analysis on save when making frequent changes

## Dependency-Specific Optimizations

### Disable Unused Features

Check and disable unused feature flags in your dependencies:

```bash
cargo features prune  # Requires cargo-features-manager
```

For manual inspection:
```bash
# Check what features are available
cargo add <crate>  # Shows feature list during interactive prompt
```

### Examples from Project

The project uses:
- **clap 4** with `derive` feature - Essential for CLI parsing
- **cairo-rs** with `pdf` feature - Required for PDF output
- **rayon** - Used for parallel computation in downsampling

All are necessary and don't have wasteful feature dependencies.

## Recommendations by Scenario

### For Local Development (max development speed)

```bash
# Use default dev profile with split-debuginfo
cargo build       # 1.5-2s
cargo check       # <1s for checking

# Install cargo-watch for automatic rebuilds
cargo install cargo-watch
cargo watch -c    # Force clear screen on rebuild
```

### For CI/CD Pipelines

```bash
# Use release-fast profile for faster CI without sacrificing too much optimization
cargo build --profile release-fast  # ~30-40% faster than release

# In GitHub Actions, add caching:
# - uses: Swatinem/rust-cache@v2
```

### For Publication/Release Builds

```bash
# Use full release profile for maximum optimization
cargo build --release
cargo publish
```

### For Heavy Iteration (rapid testing)

```bash
# Use debug profile for fastest compile
cargo build           # ~1.5-2s
cargo test            # Compilation + testing

# For faster testing, use cargo-nextest (2-3x faster)
cargo install cargo-nextest
cargo nextest run     # Parallel test execution
```

## System-Level Optimizations

### Linux: Check Linker Speed

```bash
cargo clean
time cargo build --release
# Look for "link_binary" time in output

# If link time > 20% of total, consider mold:
apt install mold
# Then update .cargo/config.toml to use mold
```

### macOS Specific

Split debug info for faster incremental builds:
```toml
[profile.dev]
split-debuginfo = "unpacked"
```

Can provide 70% speedup on macOS for incremental builds.

### Windows Specific

Use Windows Dev Drive (Win11+) for 20-30% speedup:
1. Create Dev Drive for Rust toolchain and projects
2. Move `CARGO_HOME` to Dev Drive
3. Move project code to Dev Drive

## Troubleshooting

### Build still slow after optimizations?

1. **Check for duplicate dependencies:**
   ```bash
   cargo tree --duplicate | head -20
   ```

2. **Profile the slowest crates:**
   ```bash
   cargo build --release --timings
   # Check for large waiting time (red) or many dependencies
   ```

3. **Check for expensive proc-macros** (Rust nightly):
   ```bash
   RUSTFLAGS="-Zmacro-stats" cargo +nightly build --release
   ```

4. **Profile incremental compile time:**
   ```bash
   # Time from file save to compilation complete
   touch src/main.rs && time cargo check
   ```

### LLD not found?

- On Linux: `sudo apt install lld`
- On macOS: Usually installed with Xcode tools
- Cargo will silently fall back to `ld` if not found

### Proc-macro still slow?

Consider:
- Using `watt` for Webassembly-based proc macro compilation
- Making proc-macro dependencies optional features
- Splitting proc-macro code into separate crate if possible

## References

- [Corrode.dev - Tips for Faster Rust Compile Times](https://corrode.dev/blog/tips-for-faster-rust-compile-times/)
- [Rust Book - Build Scripts and Cargo Plugins](https://doc.rust-lang.org/cargo/build-script-examples/)
- [LLVM lld Documentation](https://lld.llvm.org/)
- [Mold Linker GitHub](https://github.com/rui314/mold)
- [Parallel Rustc Frontend (Nightly)](https://blog.rust-lang.org/2023/11/09/parallel-rustc.html)

## Configuration Summary

### Files Modified

1. **`.cargo/config.toml`** - Linker and rustflags configuration
2. **`Cargo.toml`** - Build profiles and metadata
3. **`.vscode/settings.json`** (optional) - IDE optimizations

### Quick Verification

```bash
# Verify all optimizations are in place
grep -A5 "\[build\]" .cargo/config.toml  # Check linker
grep -A5 "\[profile" Cargo.toml          # Check profiles
cargo metadata --format-version 1 | jq '.packages[0]' # Verify valid config
```

## Version History

- **v0.7.0** (Feb 18, 2026): Initial implementation of compilation optimizations
  - Added lld linker configuration
  - Implemented release-fast profile
  - Added split-debuginfo for dev profile
  - Optimized build script/proc-macro compilation
