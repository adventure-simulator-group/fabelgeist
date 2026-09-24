//! CPU material construction in millimetres. Lighting is deliberately absent.
mod gilding;
mod maps;
mod wire;
mod workmanship;
use crate::{
    Error,
    artwork::{Artwork, PaintRole, PaintTone},
    document::*,
};
pub use maps::{MipLevel, TextureKind, linear_to_srgb, mips, srgb_to_linear};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    Draft,
    Preview,
    High,
    Final,
}
impl Resolution {
    pub const ALL: [Self; 4] = [Self::Draft, Self::Preview, Self::High, Self::Final];
    pub fn pixels(self) -> u32 {
        match self {
            Self::Draft => 128,
            Self::Preview => 512,
            Self::High => 1024,
            Self::Final => 2048,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Baked {
    pub size: u32,
    pub stamp: String,
    /// Straight alpha, sRGB, without construction variation or illumination.
    pub flat: Vec<u8>,
    /// Straight alpha, sRGB. RGB outside the silhouette is extended for filtering.
    pub albedo: Vec<u8>,
    /// Tangent-space, linear, +Y up. Includes physical paint relief.
    pub normal: Vec<u8>,
    /// Linear R=occlusion (1), G=perceptual roughness, B=metallic.
    pub orm: Vec<u8>,
    /// Linear R=clearcoat coverage, G=coat roughness, B=0, A=1.
    pub coat: Vec<u8>,
    /// Surface microrelief in millimetres, excluding the curved support.
    pub height: Vec<f32>,
}
/// The bake identity excludes camera and lighting controls.
pub fn stamp(d: &Document, r: Resolution) -> String {
    // A catalog calibration can change while its stable recipe ID stays put.
    let paints = Tincture::ALL.map(|t| d.surface.palette[t].appearance());
    let bytes = serde_json::to_vec(&(
        &d.arms,
        &d.drawing,
        &d.surface,
        paints,
        crate::paint::mixing::calibration_id(),
        r,
    ))
    .expect("finite validated document");
    blake3::hash(&bytes).to_hex().to_string()
}
impl Baked {
    pub fn generate(d: &Document, resolution: Resolution) -> Result<Self, Error> {
        d.validate()?;
        let size = resolution.pixels();
        let art = Artwork::compose(d)?;
        let weights = coverage(&art, size)?;
        let relief = art
            .raster_with(size, 1, |shape, _| {
                if shape.role == PaintRole::Field {
                    [0; 3]
                } else {
                    [255; 3]
                }
            })?
            .remove(0);
        let count = (size * size) as usize;
        let mut out = Self {
            size,
            stamp: stamp(d, resolution),
            flat: Vec::with_capacity(count * 4),
            albedo: Vec::with_capacity(count * 4),
            normal: vec![],
            orm: Vec::with_capacity(count * 4),
            coat: Vec::with_capacity(count * 4),
            height: Vec::with_capacity(count),
        };
        for i in 0..count {
            out.pixel(i, d, &weights, &relief);
        }
        out.normal = maps::normals(&out.height, size, [d.surface.width.0, d.surface.height.0]);
        maps::extend_edges(&mut out.albedo, size);
        Ok(out)
    }
    fn pixel(&mut self, i: usize, d: &Document, weights: &[Vec<u8>], relief: &[u8]) {
        let alpha = weights[0][i * 4 + 3];
        let s = &d.surface;
        let mut color = [0.0; 3];
        let paint = f32::from(relief[i * 4]) / 255.0;
        let sample = workmanship::sample(
            s,
            [
                ((i % self.size as usize) as f32 + 0.5) * s.width.0 / self.size as f32,
                ((i / self.size as usize) as f32 + 0.5) * s.height.0 / self.size as f32,
            ],
            s.width.0.max(s.height.0) / self.size as f32,
        );
        let mut physical = [0.0; 3];
        let (mut height, mut roughness, mut metallic, mut coat) = (0.0, 0.0, 0.0, 0.0);
        for tone in PaintTone::ALL {
            for t in Tincture::ALL {
                let index = tone.index() * Tincture::ALL.len() + t.index();
                let weight = f32::from(weights[index / 3][i * 4 + index % 3]) / 255.0;
                if weight == 0.0 {
                    continue;
                }
                let layer = gilding::Layer::at(t, tone, s, &sample, paint);
                let pigment = s.palette[t].color(tone);
                for c in 0..3 {
                    color[c] += srgb_to_linear(pigment[c]) * weight;
                    physical[c] += layer.color[c] * weight;
                }
                height += layer.height * weight;
                roughness += layer.roughness * weight;
                metallic += layer.metallic * weight;
                coat += layer.coat * weight;
            }
        }
        self.flat.extend(color.map(linear_to_srgb));
        self.flat.push(alpha);
        self.albedo.extend(physical.map(linear_to_srgb));
        self.albedo.push(alpha);
        self.height.push(height);
        self.orm
            .extend([255, unit_byte(roughness), unit_byte(metallic), 255]);
        self.coat
            .extend([unit_byte(coat), unit_byte(s.glaze_roughness.0), 0, 255]);
    }
    pub fn matches(&self, d: &Document, resolution: Resolution) -> bool {
        self.size == resolution.pixels() && self.stamp == stamp(d, resolution)
    }
}
fn coverage(art: &Artwork, size: u32) -> Result<Vec<Vec<u8>>, Error> {
    let channels = (Tincture::ALL.len() * PaintTone::ALL.len()).div_ceil(3);
    art.raster_with(size, channels, |shape, group| {
        let slot = shape.role.tone().index() * Tincture::ALL.len() + shape.tincture.index();
        std::array::from_fn(|c| if slot == group * 3 + c { 255 } else { 0 })
    })
}
pub(crate) fn unit_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
