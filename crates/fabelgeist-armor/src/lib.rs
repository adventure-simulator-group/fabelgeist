#[allow(dead_code)]
mod csg;
pub mod engraving;
pub mod material;
mod mesh;
pub mod pattern;
pub use mesh::{ArmorMesh, ArmorPart, build};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Construction {
    Solid,
    Lamellar,
    Scale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plate {
    pub gap: f32,
    pub bevel: f32,
    pub width: f32,
    pub height: f32,
    pub roundness: f32,
    pub overlap: f32,
    pub stagger: f32,
    pub hole_radius: f32,
    pub hole_pairs: u32,
}
/// Parameters used to generate uniform overlapping fauld layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fauld {
    pub construction: Construction,
    pub layer_count: u32,
    pub layer_height: f32,
    pub overlap: f32,
    pub flare: f32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Armor {
    pub construction: Construction,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub waist: f32,
    pub neck: f32,
    pub arm_cut: f32,
    pub thickness: f32,
    pub ridge: f32,
    pub ridge_sharpness: f32,
    pub center_point: f32,
    pub plate: Plate,
    pub fauld: Fauld,
    pub metal: material::Metal,
    pub translation: [f32; 3],
}
impl Default for Armor {
    fn default() -> Self {
        Self {
            construction: Construction::Solid,
            // Leaves at least 5 mm along the plate normal over the default
            // MHR chest and hips: room for an armored mail underlayer.
            width: 0.47,
            height: 0.43,
            depth: 0.16,
            waist: 0.78,
            neck: 0.075,
            arm_cut: 0.065,
            thickness: 0.003,
            ridge: 0.035,
            ridge_sharpness: 2.0,
            center_point: 0.025,
            plate: Plate {
                gap: 0.0015,
                bevel: 0.0006,
                width: 0.045,
                height: 0.065,
                roundness: 0.75,
                overlap: 0.25,
                stagger: 0.5,
                hole_radius: 0.003,
                hole_pairs: 2,
            },
            fauld: Fauld {
                construction: Construction::Solid,
                layer_count: 3,
                layer_height: 0.065,
                overlap: 0.25,
                flare: 0.015,
            },
            metal: material::Metal::default(),
            translation: [0.0, 1.05, 0.025],
        }
    }
}
impl Armor {
    pub fn validate(&self) -> Result<(), String> {
        fn range(v: f32, lo: f32, hi: f32) -> bool {
            v.is_finite() && (lo..=hi).contains(&v)
        }
        if !range(self.width, 0.2, 0.8)
            || !range(self.height, 0.2, 0.8)
            || !range(self.depth, 0.03, 0.35)
            || !range(self.waist, 0.5, 1.2)
            || !range(self.neck, 0.0, 0.12)
            || !range(self.arm_cut, 0.0, 0.09)
            || !range(self.thickness, 0.001, 0.012)
            || !range(self.ridge, 0.0, 0.12)
            || !range(self.ridge_sharpness, 1.0, 6.0)
            || !range(self.center_point, 0.0, 0.08)
            || self.translation.iter().any(|x| !range(*x, -3.0, 3.0))
        {
            return Err("Armor dimensions are outside supported bounds".into());
        }
        let p = &self.plate;
        if !range(p.gap, 0.0, 0.005)
            || !range(p.bevel, 0.0, self.thickness * 0.45)
            || !range(p.width, 0.025, 0.15)
            || !range(p.height, 0.03, 0.2)
            || !range(p.roundness, 0.0, 1.0)
            || !range(p.overlap, 0.0, 0.5)
            || !range(p.stagger, 0.0, 1.0)
            || !range(p.hole_radius, 0.0, 0.006)
            || p.height > self.height
            || p.width > self.width
            || p.hole_pairs > 3
            || p.hole_radius * 5.0 >= p.width.min(p.height)
        {
            return Err("Plate dimensions or hole spacing are invalid".into());
        }
        let f = &self.fauld;
        if f.layer_count > 12
            || !range(f.layer_height, 0.025, 0.15)
            || !range(f.overlap, 0.0, 0.5)
            || !range(f.flare, 0.0, 0.05)
        {
            return Err("Armor supports up to twelve bounded overlapping layers".into());
        }
        self.metal.validate()
    }
}
