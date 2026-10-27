//! What hair is made of: the parameters of the Chiang hair BSDF.
//!
//! DiffLocks shades its strands with Blender's Principled Hair BSDF in the
//! Chiang model (Chiang et al. 2016): a dielectric cylinder with cuticle
//! scales, its colour the absorption of the melanin in its cortex. Colour
//! comes from how much melanin there is and how much of it is the red
//! pheomelanin rather than the dark eumelanin, as Blender parameterises it,
//! plus an optional dye. [`HairMaterial::absorption`] is that coefficient;
//! [`HairMaterial::base_color`] the colour it gives a strand (Chiang's
//! mapping, inverted), which the real-time shading uses.
//!
//! Along the strand DiffLocks' material varies the melanin two ways, which
//! its metadata records: darker towards the root (`root_darkness_start`,
//! `_end`, `_strength`, as shares of the strand) and in waves along it
//! (`material_wave_scale`, `_phase_offset`, `_strength`); and it takes the
//! BSDF's `coat`. [`HairMaterial`] has the same, under those names.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HairMaterial {
    /// 0 is white, 0.2 blond, 0.5 brown, 0.8 dark brown, 1 black.
    pub melanin: f32,
    /// Share of pheomelanin: 0 ashen, 1 red.
    pub redness: f32,
    /// A dye's colour; white is none.
    pub tint: [f32; 3],
    /// Longitudinal roughness: how far highlights spread along the strand.
    pub roughness: f32,
    /// Azimuthal roughness: how soft the look across it is.
    pub radial_roughness: f32,
    /// The cuticle tilt, radians (about 2 to 4 degrees).
    pub cuticle_tilt: f32,
    pub ior: f32,
    /// How much each strand's melanin varies.
    pub random_color: f32,
    /// How much each strand's roughness varies.
    pub random_roughness: f32,
    /// How much lighter the tips are than the roots (sun-bleached), 0 to 1.
    #[serde(default)]
    pub tip_lightness: f32,
    /// Share of strands with no pigment left: grey.
    #[serde(default)]
    pub grey: f32,
    /// Darker towards the root: fully `root_darkness_strength` more melanin
    /// before `root_darkness_start` along the strand (0 root, 1 tip), none
    /// past `root_darkness_end`.
    #[serde(default)]
    pub root_darkness_start: f32,
    #[serde(default)]
    pub root_darkness_end: f32,
    #[serde(default)]
    pub root_darkness_strength: f32,
    /// Melanin rising and falling in waves along the strand: `wave_scale`
    /// waves from root to tip, starting `wave_phase` degrees in, up to
    /// `wave_strength` either way.
    #[serde(default)]
    pub wave_scale: f32,
    #[serde(default)]
    pub wave_phase: f32,
    #[serde(default)]
    pub wave_strength: f32,
    /// The Principled Hair BSDF's coat: a clear layer over the strand, as
    /// on fur. Recorded for an offline renderer; the real-time shading has
    /// no coat lobe.
    #[serde(default)]
    pub coat: f32,
    /// How the radius falls to the tip.
    pub taper: Taper,
}

/// Radius along a strand as `Set Hair Curve Profile` shapes it: the root
/// radius times a factor running from 1 at the root to `tip` at the tip,
/// by `shape_range(1 - factor, shape)`.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Taper {
    pub tip: f32,
    pub shape: f32,
}

impl Default for Taper {
    fn default() -> Self {
        Self {
            tip: 0.2,
            shape: 0.85,
        }
    }
}

impl Taper {
    /// The radius factor at `factor` along the strand.
    pub fn at(&self, factor: f32) -> f32 {
        let f = crate::ops::shape_range(1.0 - factor, 0.0, 1.0, self.shape, 2.0);
        self.tip + (1.0 - self.tip) * f
    }
}

impl Default for HairMaterial {
    fn default() -> Self {
        Self {
            melanin: 0.65,
            redness: 0.35,
            tint: [1.0; 3],
            roughness: 0.3,
            radial_roughness: 0.3,
            cuticle_tilt: 2.5f32.to_radians(),
            ior: 1.55,
            random_color: 0.3,
            random_roughness: 0.2,
            tip_lightness: 0.15,
            grey: 0.0,
            root_darkness_start: 0.0,
            root_darkness_end: 0.1,
            root_darkness_strength: 0.0,
            wave_scale: 3.0,
            wave_phase: 0.0,
            wave_strength: 0.0,
            coat: 0.0,
            taper: Taper::default(),
        }
    }
}

/// Chiang's polynomial in the azimuthal roughness, relating a colour to its
/// absorption.
fn chiang(beta_n: f32) -> f32 {
    let b = beta_n;
    5.969 - 0.215 * b + 2.532 * b.powi(2) - 10.73 * b.powi(3)
        + 5.574 * b.powi(4)
        + 0.245 * b.powi(5)
}

impl HairMaterial {
    pub fn named(name: &str) -> Option<Self> {
        let (melanin, redness) = match name {
            "black" => (0.95, 0.1),
            "dark brown" => (0.8, 0.25),
            "brown" => (0.7, 0.35),
            "auburn" => (0.55, 0.75),
            "red" => (0.4, 0.95),
            "blond" => (0.22, 0.4),
            "platinum" => (0.08, 0.2),
            "grey" => (0.25, 0.02),
            _ => return None,
        };
        Some(Self {
            melanin,
            redness,
            ..Default::default()
        })
    }

    /// Absorption coefficient per unit of fibre diameter, Blender's melanin
    /// model: eumelanin and pheomelanin in proportion `1 - redness :
    /// redness`, their total `-ln(1 - melanin)`, and a dye on top.
    pub fn absorption(&self) -> [f32; 3] {
        let quantity = -(1.0 - self.melanin).max(1e-4).ln();
        let eumelanin = quantity * (1.0 - self.redness);
        let pheomelanin = quantity * self.redness;
        const EU: [f32; 3] = [0.506, 0.841, 1.653];
        const PHEO: [f32; 3] = [0.343, 0.733, 1.924];
        let f = chiang(self.radial_roughness);
        std::array::from_fn(|k| {
            let dye = (self.tint[k].clamp(1e-3, 1.0).ln() / f).powi(2);
            eumelanin * EU[k] + pheomelanin * PHEO[k] + dye
        })
    }

    /// The colour a real-time model should give a strand of this material.
    ///
    /// Chiang's mapping between colour and absorption, inverted, over a
    /// quarter of the absorption: over the whole of it, the melanin
    /// quantities Blender calls brown come out near black and saturated,
    /// where Cycles renders them a mid brown. The quarter is calibrated by
    /// eye against Cycles' renders of the named colours; offline rendering
    /// should take [`HairMaterial::absorption`] itself.
    pub fn base_color(&self) -> [f32; 3] {
        let f = chiang(self.radial_roughness);
        let melanin = Self {
            tint: [1.0; 3],
            ..*self
        }
        .absorption();
        std::array::from_fn(|k| {
            (-(melanin[k].max(0.0) * REAL_TIME_ABSORPTION).sqrt() * f).exp()
                * self.tint[k].clamp(0.0, 1.0)
        })
    }
}

/// The share of the absorption [`HairMaterial::base_color`] takes.
const REAL_TIME_ABSORPTION: f32 = 0.25;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn more_melanin_is_darker_and_redness_is_red() {
        let light = HairMaterial::named("blond").unwrap().base_color();
        let dark = HairMaterial::named("black").unwrap().base_color();
        assert!(light.iter().zip(&dark).all(|(l, d)| l > d));
        let red = HairMaterial::named("red").unwrap().base_color();
        assert!(red[0] > red[1] && red[1] > red[2], "{red:?}");
    }

    #[test]
    fn a_dye_alone_gives_back_its_colour() {
        let dyed = HairMaterial {
            melanin: 0.0,
            tint: [0.2, 0.5, 0.9],
            ..Default::default()
        };
        for (c, t) in dyed.base_color().iter().zip(dyed.tint) {
            assert!((c - t).abs() < 1e-3, "{c} {t}");
        }
    }

    #[test]
    fn the_taper_runs_from_one_to_the_tip() {
        let taper = Taper::default();
        assert!((taper.at(0.0) - 1.0).abs() < 1e-4);
        assert!((taper.at(1.0) - taper.tip).abs() < 1e-3);
    }
}
