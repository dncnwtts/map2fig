import os
import numpy as np
import matplotlib.cm as cm
import matplotlib.pyplot as plt

N = 256

COLORMAPS = [
    "viridis",
    "plasma",
    "inferno",
    "magma",
    "cividis",
    "afmhot",
    # add more here
]

def generate_lut(name):
    cmap = plt.get_cmap(name, N)
    data = (cmap(np.linspace(0, 1, N))[:, :3] * 255).round().astype(int)

    rust_name = name.upper()
    filename = f"{name}_lut.rs"

    with open(filename, "w") as f:
        f.write("// Auto-generated from matplotlib\n")
        f.write(f"// Colormap: {name}\n\n")
        f.write(f"pub const {rust_name}_LUT: [[u8; 3]; {N}] = [\n")

        for r, g, b in data:
            f.write(f"    [{r}, {g}, {b}],\n")

        f.write("];\n")

    print(f"Wrote {filename}")

for cmap in COLORMAPS:
    generate_lut(cmap)

