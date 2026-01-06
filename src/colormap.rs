use image::Rgb;

// Lookup tables generated externally (256 × RGB)
include!("../colormap/viridis_lut.rs");
include!("../colormap/plasma_lut.rs");
include!("../colormap/inferno_lut.rs");

// A sampled colormap backed by a fixed RGB lookup table
#[derive(Debug)]
pub struct Colormap {
    /// Canonical name (used by CLI, logging, etc.)
    pub name: &'static str,

    /// RGB lookup table (typically length 256)
    pub lut: &'static [[u8; 3]],
}

/* ------------------------------------------------------------------------- */
/*  Built-in colormaps                                                        */
/* ------------------------------------------------------------------------- */

pub static VIRIDIS: Colormap = Colormap {
    name: "viridis",
    lut: &VIRIDIS_LUT,
};

pub static PLASMA: Colormap = Colormap {
    name: "plasma",
    lut: &PLASMA_LUT,
};

pub static INFERNO: Colormap = Colormap {
    name: "inferno",
    lut: &INFERNO_LUT,
};

// Registry of all available colormaps
//
// Adding a new colormap requires:
//   1. include!("foo_lut.rs")
//   2. pub static FOO: Colormap = ...
//   3. add it to this slice
pub static COLORMAPS: &[&Colormap] = &[
    &VIRIDIS,
    &PLASMA,
    &INFERNO,
];

/* ------------------------------------------------------------------------- */
/*  Colormap API                                                              */
/* ------------------------------------------------------------------------- */

impl Colormap {
    /// Sample the colormap at `t ∈ [0, 1]`
    #[inline]
    pub fn sample(&self, t: f64) -> Rgb<u8> {
        let n = self.lut.len() - 1;
        let i = (t.clamp(0.0, 1.0) * n as f64).round() as usize;
        Rgb(self.lut[i])
    }

    /// Color for values below the data range
    #[inline]
    pub fn under(&self) -> Rgb<u8> {
        Rgb(self.lut[0])
    }

    /// Color for values above the data range
    #[inline]
    pub fn over(&self) -> Rgb<u8> {
        Rgb(self.lut[self.lut.len() - 1])
    }
}

/* ------------------------------------------------------------------------- */
/*  Lookup helpers                                                            */
/* ------------------------------------------------------------------------- */

// Find a colormap by name (case-insensitive)
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

// Return all available colormap names (for CLI help, --list, etc.)
pub fn available_colormaps() -> Vec<&'static str> {
    COLORMAPS.iter().map(|c| c.name).collect()
}

