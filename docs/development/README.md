# Development Guides & Setup

This directory contains guides for setting up, building, testing, and benchmarking the HEALPix Plotter project.

## Quick Start

**Get started developing:**

```bash
# Clone and setup
git clone https://github.com/dncnwtts/map2fig.git
cd map2fig

# Build
cargo build --release

# Run tests
cargo test

# Benchmark
cargo bench
```

See individual guides below for detailed instructions.

## Setup & Installation

- **[README.md](README.md)** - Developer guide overview and project setup
- **[TECTONIC_INSTALL.md](TECTONIC_INSTALL.md)** - LaTeX/Tectonic installation and configuration
- **[TEST_REQUIREMENTS.md](TEST_REQUIREMENTS.md)** - Test dependencies and environment setup

## Development & Testing

- **[BENCHMARK_GUIDE.md](BENCHMARK_GUIDE.md)** - Comprehensive benchmarking setup and usage
- **[SPARSE_FITS_TESTING.md](SPARSE_FITS_TESTING.md)** - Testing with sparse FITS files
- **[CLI_BUILDER_GUIDE.md](CLI_BUILDER_GUIDE.md)** - CLI argument builder and interface development

## Common Tasks

| Task | Guide |
|------|-------|
| **Build from source** | [README.md](README.md) |
| **Run benchmarks** | [BENCHMARK_GUIDE.md](BENCHMARK_GUIDE.md) |
| **Add CLI arguments** | [CLI_BUILDER_GUIDE.md](CLI_BUILDER_GUIDE.md) |
| **Test FITS handling** | [SPARSE_FITS_TESTING.md](SPARSE_FITS_TESTING.md) |
| **Set up PDF rendering** | [TECTONIC_INSTALL.md](TECTONIC_INSTALL.md) |

## Development Workflow

1. **Setup**: Follow [README.md](README.md) from the project root
2. **Make changes**: Edit code in `src/`
3. **Test**: Run `cargo test`
4. **Benchmark**: Use [BENCHMARK_GUIDE.md](BENCHMARK_GUIDE.md) to profile changes
5. **Verify**: Run integration tests before committing

## For New Contributors

Start with:
1. [README.md](README.md) - Project overview and setup
2. [TEST_REQUIREMENTS.md](TEST_REQUIREMENTS.md) - Dependencies
3. Your feature's design doc in [../architecture/](../architecture/)
4. [BENCHMARK_GUIDE.md](BENCHMARK_GUIDE.md) - How to measure impact

---

**Last Updated**: February 2026  
**See Also**: [../README.md](../README.md) for full documentation hub
