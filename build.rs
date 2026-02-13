/// Build script for map2fig
/// Checks for optional dependencies (like tectonic) and provides installation guidance
use std::process::Command;

fn main() {
    // Check if tectonic is available
    if !check_command_exists("tectonic") {
        println!("cargo:warning=tectonic not found in PATH");
        println!("cargo:warning=For improved LaTeX rendering, install tectonic:");
        println!("cargo:warning=  Run: ./install.sh (recommended - handles all dependencies)");
        println!("cargo:warning=  Or:  cargo install tectonic");
        println!("cargo:warning=Note: pdflatex will be used as fallback (already functional)");
    } else {
        println!("cargo:warning=✓ tectonic found in PATH");
    }

    // Also warn if pdflatex is missing (more critical)
    if !check_command_exists("pdflatex") {
        println!("cargo:warning=WARNING: pdflatex not found in PATH");
        println!("cargo:warning=Install it to enable LaTeX rendering:");
        println!(
            "cargo:warning=  Ubuntu/Debian: sudo apt-get install texlive-latex-base texlive-latex-extra"
        );
        println!("cargo:warning=  macOS: brew install basictex");
        println!("cargo:warning=  Fedora: sudo dnf install texlive-latex");
    }
}

/// Check if a command exists in PATH
fn check_command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
