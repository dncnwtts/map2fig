#!/usr/bin/env python3

import os
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.cm as mpl_cm
import matplotlib

# Optional cmasher support
try:
    import cmasher as cmr
    HAS_CMASHER = True
except ImportError:
    HAS_CMASHER = False

# -----------------------
# Configuration
# -----------------------
N = 256

ROOT = os.path.dirname(os.path.abspath(__file__))
COLORMAP_DIR = os.path.join(ROOT, "../colormap")
RUST_OUT = os.path.join(ROOT, "../src/colormap.rs")

CUSTOM_DAT = {
    "planck": "planck_cmap.dat",
    "planck_log": "planck_log_cmap.dat",
    "wmap": "wmap_cmap.dat",
}

os.makedirs(COLORMAP_DIR, exist_ok=True)

# -----------------------
# Helpers
# -----------------------
def normalize_rust(name: str) -> str:
    return name.upper().replace("-", "_")

def normalize_cli(name: str) -> str:
    return name.lower().replace("_", "-")

def write_lut(path, rust_name, data, comment):
    with open(path, "w") as f:
        f.write("// Auto-generated colormap LUT\n")
        f.write(f"// Source: {comment}\n\n")
        f.write(f"pub const {rust_name}_LUT: [[u8; 3]; {len(data)}] = [\n")
        for r, g, b in data:
            f.write(f"    [{r}, {g}, {b}],\n")
        f.write("];\n")

# -----------------------
# Collect colormaps
# -----------------------
entries = []
seen = set()

def register(name, data, source):
    if name[:4] == "cmr.":
        name = name[4:]
    rust = normalize_rust(name)
    cli = normalize_cli(name)
    key = (rust, cli)
    if key in seen:
        return
    seen.add(key)

    filename = f"{rust}_LUT.rs"
    write_lut(
        os.path.join(COLORMAP_DIR, filename),
        rust,
        data,
        source,
    )
    entries.append((rust, cli, filename))

# -----------------------
# 1. matplotlib colormaps
# -----------------------
for name in sorted(matplotlib.colormaps.keys()):
    cmap = mpl_cm.get_cmap(name, N)
    data = (cmap(np.linspace(0, 1, N))[:, :3] * 255).round().astype(int)
    register(name, data, f"matplotlib '{name}'")

# -----------------------
# 2. cmasher colormaps
# -----------------------
if HAS_CMASHER:
    for name in sorted(cmr.get_cmap_list()):
        data = cmr.take_cmap_colors(f'cmr.{name}', 255)
        register(name, data, f"cmasher '{name}'")

# -----------------------
# 3. Custom .dat colormaps
# -----------------------
for name, fname in CUSTOM_DAT.items():
    print(name, fname)
    path = os.path.join(ROOT, fname)
    data = np.loadtxt(path).astype(int)

    if data.shape[1] != 3:
        raise ValueError(f"{fname} must have 3 columns (RGB)")

    if len(data) != N:
        x_old = np.linspace(0, 1, len(data))
        x_new = np.linspace(0, 1, N)
        data = np.vstack([
            np.interp(x_new, x_old, data[:, i]) for i in range(3)
        ]).T.round().astype(int)

    register(name, data, f"custom file '{fname}'")

# -----------------------
# Generate Rust colormaps.rs
# -----------------------
with open(RUST_OUT, "w") as f:
    f.write("use image::Rgb;\n\n")

    for rust, _, filename in entries:
        f.write(f'include!("../colormap/{filename}");\n')

    f.write("""
#[derive(Debug)]
pub struct Colormap {
    pub name: &'static str,
    pub lut: &'static [[u8; 3]],
}

/* ------------------------------------------------------------------------- */
/*  Built-in colormaps                                                        */
/* ------------------------------------------------------------------------- */
""")

    for rust, cli, _ in entries:
        f.write(f"""
pub static {rust}: Colormap = Colormap {{
    name: "{cli}",
    lut: &{rust}_LUT,
}};
""")

    f.write("\npub static COLORMAPS: &[&Colormap] = &[\n")
    for rust, _, _ in entries:
        f.write(f"    &{rust},\n")
    f.write("];\n")

    f.write("""
/* ------------------------------------------------------------------------- */
/*  Colormap API                                                              */
/* ------------------------------------------------------------------------- */

impl Colormap {
    #[inline]
    pub fn sample(&self, t: f64) -> Rgb<u8> {
        let n = self.lut.len() - 1;
        let i = (t.clamp(0.0, 1.0) * n as f64).round() as usize;
        Rgb(self.lut[i])
    }

    #[inline]
    pub fn under(&self) -> Rgb<u8> {
        Rgb(self.lut[0])
    }

    #[inline]
    pub fn over(&self) -> Rgb<u8> {
        Rgb(self.lut[self.lut.len() - 1])
    }
}

/* ------------------------------------------------------------------------- */
/*  Lookup helpers                                                            */
/* ------------------------------------------------------------------------- */

pub fn get_colormap(name: &str) -> &'static Colormap {
    let name = name.to_lowercase();
    COLORMAPS
        .iter()
        .copied()
        .find(|c| c.name == name)
        .unwrap_or_else(|| {
            panic!(
                "Unknown colormap '{}'. Available: {}",
                name,
                available_colormaps().join(", ")
            )
        })
}

pub fn available_colormaps() -> Vec<&'static str> {
    COLORMAPS.iter().map(|c| c.name).collect()
}
""")

print(f"Generated {len(entries)} colormaps")
print(f"Wrote {RUST_OUT}")

