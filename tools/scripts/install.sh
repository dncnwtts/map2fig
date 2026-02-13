#!/bin/bash
# Installation script for map2fig with optional dependencies
# This script helps set up the build environment with tectonic support

set -e

echo "=== map2fig Installation Helper ==="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt-get &> /dev/null; then
            echo "ubuntu"
        elif command -v dnf &> /dev/null; then
            echo "fedora"
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

# Check if sudo is available
check_sudo() {
    sudo -n true 2>/dev/null && return 0 || return 1
}

# Try to install pre-built tectonic binary
install_tectonic_prebuilt() {
    echo -e "${BLUE}Attempting to download pre-built tectonic binary...${NC}"
    
    local temp_dir=$(mktemp -d)
    local arch=$(uname -m)
    
    if [[ "$arch" != "x86_64" ]]; then
        rm -rf "$temp_dir"
        return 1  # Only pre-built x86_64 available
    fi
    
    # Try to download from GitHub releases
    local version="0.15.0"
    local url="https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic%40${version}/tectonic-${version}-x86_64-unknown-linux-gnu.tar.gz"
    
    local download_tool=""
    if command -v curl &> /dev/null; then
        download_tool="curl"
    elif command -v wget &> /dev/null; then
        download_tool="wget"
    else
        rm -rf "$temp_dir"
        return 1
    fi
    
    # Attempt download
    if [[ "$download_tool" == "curl" ]]; then
        curl -fL -o "$temp_dir/tectonic.tar.gz" "$url" 2>/dev/null || {
            rm -rf "$temp_dir"
            return 1
        }
    else
        wget -q -O "$temp_dir/tectonic.tar.gz" "$url" 2>/dev/null || {
            rm -rf "$temp_dir"
            return 1
        }
    fi
    
    # Extract and install
    if tar -xzf "$temp_dir/tectonic.tar.gz" -C "$temp_dir" 2>/dev/null; then
        if [[ -f "$temp_dir/tectonic" ]]; then
            # Try to put in /usr/local/bin first, fallback to ~/.cargo/bin
            if sudo mv "$temp_dir/tectonic" "/usr/local/bin/" 2>/dev/null; then
                rm -rf "$temp_dir"
                return 0
            elif mv "$temp_dir/tectonic" "$HOME/.cargo/bin/" 2>/dev/null; then
                rm -rf "$temp_dir"
                return 0
            fi
        fi
    fi
    
    rm -rf "$temp_dir"
    return 1
}

# Install tectonic - try pre-built binary first, fallback to cargo install
install_tectonic() {
    echo ""
    echo -e "${BLUE}Installing tectonic...${NC}"
    echo ""
    
    # Try GitHub pre-built binary first if on Linux (much faster)
    if [[ "$OS_TYPE" == "ubuntu" || "$OS_TYPE" == "fedora" ]]; then
        if install_tectonic_prebuilt; then
            echo -e "${GREEN}✓ tectonic installed from pre-built binary${NC}"
            return 0
        fi
        echo "Pre-built not available, will compile from source..."
    fi
    
    # Fallback to cargo install
    echo "Compiling tectonic from source (this may take 10-30 minutes)..."
    if cargo install tectonic 2>&1 | tail -5; then
        if command -v tectonic &> /dev/null; then
            echo -e "${GREEN}✓ tectonic installed${NC}"
            return 0
        fi
    fi
    
    return 1
}

# Install system dependencies for tectonic compilation
install_tectonic_deps() {
    local os=$1
    echo -e "${BLUE}Tectonic build dependencies needed${NC}"
    
    case $os in
        ubuntu)
            echo "Ubuntu/Debian detected - required packages:"
            echo "  build-essential"
            echo "  libharfbuzz-dev"
            echo "  libharfbuzz0b"
            echo "  pkg-config"
            echo "  libfontconfig1-dev"
            echo "  libfreetype6-dev"
            echo ""
            
            if check_sudo; then
                echo "✓ sudo available - installing automatically..."
                sudo apt-get update
                sudo apt-get install -y \
                    build-essential \
                    libharfbuzz-dev \
                    libharfbuzz0b \
                    pkg-config \
                    libfontconfig1-dev \
                    libfreetype6-dev
                echo -e "${GREEN}✓ Dependencies installed${NC}"
                return 0
            else
                echo -e "${YELLOW}⚠ sudo not available${NC}"
                echo "Please ask your system administrator to install these packages:"
                echo "  sudo apt-get install -y build-essential libharfbuzz-dev libharfbuzz0b pkg-config libfontconfig1-dev libfreetype6-dev"
                return 1
            fi
            ;;
        fedora)
            echo "Fedora/RHEL detected - required packages:"
            echo "  gcc"
            echo "  gcc-c++"
            echo "  harfbuzz-devel"
            echo "  pkg-config"
            echo "  fontconfig-devel"
            echo "  freetype-devel"
            echo ""
            
            if check_sudo; then
                echo "✓ sudo available - installing automatically..."
                sudo dnf install -y \
                    gcc \
                    gcc-c++ \
                    harfbuzz-devel \
                    pkg-config \
                    fontconfig-devel \
                    freetype-devel
                echo -e "${GREEN}✓ Dependencies installed${NC}"
                return 0
            else
                echo -e "${YELLOW}⚠ sudo not available${NC}"
                echo "Please ask your system administrator to install these packages:"
                echo "  sudo dnf install -y gcc gcc-c++ harfbuzz-devel pkg-config fontconfig-devel freetype-devel"
                return 1
            fi
            ;;
        macos)
            echo "macOS detected - checking Homebrew..."
            if ! command -v brew &> /dev/null; then
                echo -e "${RED}✗ Homebrew not found - please install from https://brew.sh${NC}"
                return 1
            fi
            echo "Installing with Homebrew..."
            brew install harfbuzz pkg-config
            echo -e "${GREEN}✓ Dependencies installed${NC}"
            return 0
            ;;
        *)
            echo -e "${RED}✗ Unknown OS - please manually install build dependencies${NC}"
            return 1
            ;;
    esac
}

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Rust not found${NC}"
    echo "Install from https://rustup.rs/"
    exit 1
fi
echo -e "${GREEN}✓ Rust installed${NC}"

# Detect OS once at the start
OS_TYPE=$(detect_os)

# Check pdflatex (critical)
if ! command -v pdflatex &> /dev/null; then
    echo -e "${YELLOW}⚠ pdflatex not found${NC}"
    echo "Install LaTeX packages:"
    case $OS_TYPE in
        ubuntu)
            echo "  Ubuntu/Debian: sudo apt-get install texlive-latex-base texlive-latex-extra"
            ;;
        fedora)
            echo "  Fedora: sudo dnf install texlive-latex"
            ;;
        macos)
            echo "  macOS: brew install basictex"
            ;;
        *)
            echo "  Please install LaTeX for your system"
            ;;
    esac
    read -p "Continue without pdflatex? (y/N) " -n 1 -r
    echo
    [[ ! $REPLY =~ ^[Yy]$ ]] && exit 1
else
    echo -e "${GREEN}✓ pdflatex installed${NC}"
fi

# Check tectonic (optional but recommended)
if command -v tectonic &> /dev/null; then
    echo -e "${GREEN}✓ tectonic already installed${NC}"
else
    echo -e "${YELLOW}⚠ tectonic not found (optional but recommended)${NC}"
    echo ""
    echo "Tectonic requires build tools and development libraries to compile from source."
    echo ""
    read -p "Attempt to install tectonic now? (Y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        if [[ "$OS_TYPE" == "unknown" ]]; then
            echo -e "${YELLOW}Could not detect OS automatically${NC}"
            echo "Attempting tectonic installation..."
            install_tectonic
        else
            install_tectonic_deps "$OS_TYPE"
            if [[ $? -eq 0 ]]; then
                # Dependencies available - try installing
                install_tectonic
            else
                # Dependencies not available
                echo ""
                echo -e "${YELLOW}Dependencies not available in this environment${NC}"
                echo "You have two options:"
                echo ""
                echo "1. Ask your system administrator to install the required packages"
                echo "   (see commands above)"
                echo ""
                echo "2. Continue building map2fig without tectonic"
                echo "   (pdflatex fallback will be used - fully functional)"
                echo ""
                read -p "Continue building without tectonic? (Y/n) " -n 1 -r
                echo
                if [[ ! $REPLY =~ ^[Nn]$ ]]; then
                    echo "Continuing with pdflatex fallback..."
                else
                    echo "Cancelling installation"
                    exit 1
                fi
            fi
        fi
        
        if command -v tectonic &> /dev/null; then
            echo -e "${GREEN}✓ tectonic installed successfully${NC}"
        else
            echo -e "${YELLOW}⚠ tectonic not available - using pdflatex fallback${NC}"
            echo "map2fig will still work correctly with system LaTeX"
        fi
    fi
fi

echo ""
echo "=== Building map2fig ==="
cargo build --release

echo ""
echo -e "${GREEN}✓ Installation complete!${NC}"
echo "Run: ./target/release/map2fig --help"
