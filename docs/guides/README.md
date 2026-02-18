# User Guides & Tutorials

Step-by-step guides for using map2fig effectively.

## Contents

- **[CLI_UI_IMPROVEMENTS.md](CLI_UI_IMPROVEMENTS.md)** - Command-line interface improvements
  - Positional argument syntax
  - Auto-filename generation
  - Clean help output
  - Examples of common use cases

## Getting Started

### Basic Usage
```bash
# Most simple: input file will auto-generate output filename
map2fig input.fits

# With explicit output filename
map2fig input.fits output.pdf

# With options
map2fig data.fits output.pdf --colormap plasma --log-scale
```

For complete CLI documentation and examples, see [CLI_UI_IMPROVEMENTS.md](CLI_UI_IMPROVEMENTS.md).

## What's New

- **Positional Arguments**: `map2fig input.fits output.png` instead of `-f input.fits -o output.png`
- **Auto Filenames**: If OUTPUT is omitted, automatically generates `input.png` from `input.fits`
- **Cleaner Help**: Common options visible, advanced options hidden but functional
- **Intuitive Usage**: `Usage: map2fig [FITS] [OUTPUT] [OPTIONS]`

See [CLI_UI_IMPROVEMENTS.md](CLI_UI_IMPROVEMENTS.md) for detailed examples and backward compatibility notes.
