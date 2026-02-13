# Tectonic Installation Guide

## Overview

Tectonic is an optional but recommended LaTeX engine that provides superior rendering quality. If it's not available, map2fig automatically falls back to system `pdflatex`.

## Quick Install

The easiest way is to use the provided installation script:

```bash
./install.sh
```

This script will:
1. Detect your operating system
2. Check for required development libraries
3. Offer to install them (if you have sudo access) OR provide admin commands
4. Compile and install tectonic (if dependencies are available)
5. Build map2fig (works with or without tectonic)

## Machines Without Sudo Access

If you don't have `sudo` access:

1. Run `./install.sh`
2. When asked about tectonic, answer **yes (Y)**
3. The script will detect you lack sudo and provide the exact commands to give to your system administrator
4. Once they install the packages, run `./install.sh` again and tectonic will compile successfully
5. If dependencies aren't installed, the script will let you skip tectonic and proceed with the pdflatex fallback (fully functional)

**Example flow:**
```
⚠ tectonic not found (optional but recommended)
Tectonic requires build tools and development libraries to compile from source.

Attempt to install tectonic now? (Y/n) Y

Ubuntu/Debian detected - required packages:
  build-essential
  libharfbuzz-dev
  libharfbuzz0b
  pkg-config
  libfontconfig1-dev
  libfreetype6-dev

⚠ sudo not available
Please ask your system administrator to install these packages:
  sudo apt-get install -y build-essential libharfbuzz-dev libharfbuzz0b pkg-config libfontconfig1-dev libfreetype6-dev

Dependencies not available in this environment
You have two options:

1. Ask your system administrator to install the required packages
   (see commands above)

2. Continue building map2fig without tectonic
   (pdflatex fallback will be used - fully functional)

Continue building without tectonic? (Y/n) Y
Continuing with pdflatex fallback...
```

## Manual Installation

### Step 1: Install Build Dependencies

Ask your system administrator to run one of these commands:

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    libharfbuzz-dev \
    libharfbuzz0b \
    pkg-config \
    libfontconfig1-dev \
    libfreetype6-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install -y \
    gcc \
    gcc-c++ \
    harfbuzz-devel \
    pkg-config \
    fontconfig-devel \
    freetype-devel
```

**macOS:**
```bash
brew install harfbuzz pkg-config
```

### Step 2: Install Tectonic

Once dependencies are installed:
```bash
cargo install tectonic
```

This will take several minutes as it compiles from source.

### Step 3: Verify Installation

```bash
tectonic --version
```

You should see something like: `Tectonic 0.15.0`

## Troubleshooting

### Error: `failed to copy header: No such file or directory`

**Cause:** Missing harfbuzz development headers

**Solution:** Install the required development libraries (see "Install Build Dependencies" above), then try again:
```bash
cargo install tectonic
```

### Error: `harfbuzz.h not found`

**Cause:** libharfbuzz-dev not installed

**Solution:** Ask your system administrator to install it, then retry.

### Compilation taking too long

Tectonic is large and compiles from source. This is normal and can take 10-30 minutes depending on your system. Be patient!

You can monitor progress with:
```bash
RUST_BACKTRACE=1 cargo install tectonic
```

## Fallback Behavior

If tectonic doesn't work or you don't want to install it, map2fig will automatically use system `pdflatex`:

```bash
# Works even without tectonic - uses pdflatex fallback
./target/release/map2fig -f data.fits -o map.pdf --latex --units '$K_{CMB}$'
```

The main difference is:
- **With tectonic**: More reliable baseline handling, potentially better spacing
- **With pdflatex**: Fully functional, widely available fallback

Both produce identical output quality for most use cases.

## Verifying Tectonic is Being Used

After successful installation, you'll see no warning during build:

```bash
cargo build --release
# No "tectonic not found" warning = tectonic is available
```

When rendering LaTeX, tectonic will be used automatically in the rendering pipeline.
