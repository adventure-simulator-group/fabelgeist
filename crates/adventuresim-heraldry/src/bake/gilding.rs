//! Qualitative application profiles, not measured reconstructions of objects.
//! Sources and the limits of the RGB glaze approximation are in REFERENCES.md.
use super::{srgb_to_linear, workmanship::Sample};
use crate::{artwork::PaintTone, document::*};

// Linear RGB conductor reflectance, independent of the heraldic pigment palette.
// Values tabulated by PlayCanvas; also the Khronos gold material example.
const GOLD_REFLECTANCE: [f32; 3] = [1.0, 0.766, 0.336];
const SILVER_REFLECTANCE: [f32; 3] = [0.972, 0.960, 0.915];
// Artist response coefficients. They express the documented contrast between
// polished water gilding and unburnished adhesive gilding, not measured BRDFs.
const WATER_ROUGHNESS: f32 = 0.34;
const BURNISHED_ROUGHNESS: f32 = 0.14;
const BURNISH_SMOOTHING: f32 = 0.65;
const OIL_ROUGHNESS: f32 = 0.38;
const MORDANT_ROUGHNESS: f32 = 0.42;
const OIL_CREASE_RELIEF_MM: f32 = 0.002;
const OIL_BRUSH_RELIEF_MM: f32 = 0.004;
const MORDANT_BRUSH_VARIATION: f32 = 0.15;
const PAINT_ROUGHNESS_VARIATION: f32 = 0.035;
const MODELING_FILM_FRACTION: f32 = 0.15;
// One-pass optical absorption. The return through the glaze doubles this path.
// This yellow spectrum is authored; no recipe-specific absorption was measured.
const YELLOW_ABSORPTION: [f32; 3] = [0.04, 0.28, 1.1];

pub(super) struct Layer {
    pub color: [f32; 3],
    pub height: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub coat: f32,
}
impl Layer {
    pub fn at(t: Tincture, tone: PaintTone, s: &PaintedSurface, work: &Sample, paint: f32) -> Self {
        if tone != PaintTone::Base {
            let mut layer = Self::at(t, PaintTone::Base, s, work, paint);
            layer.color = s.palette[t]
                .color(tone)
                .map(|v| srgb_to_linear(v) * work.pigment);
            layer.height += s.pigment.0 * MODELING_FILM_FRACTION;
            layer.roughness = (s.palette[t].appearance().roughness.0
                + work.brush * PAINT_ROUGHNESS_VARIATION)
                .clamp(0.04, 1.0);
            layer.metallic = 0.0;
            layer.coat = s.glaze.0;
            return layer;
        }
        let finish = match t {
            Tincture::Or => s.gold,
            Tincture::Argent => s.silver,
            _ => MetalFinish::Pigment,
        };
        let mut layer = Self {
            color: if t == Tincture::Argent {
                SILVER_REFLECTANCE
            } else {
                GOLD_REFLECTANCE
            },
            height: work.support,
            roughness: OIL_ROUGHNESS,
            metallic: 1.0,
            coat: s.glaze.0,
        };
        match finish {
            MetalFinish::Pigment => {
                layer.color = s.palette[t]
                    .color(tone)
                    .map(|v| srgb_to_linear(v) * work.pigment);
                layer.height += paint * s.pigment.0 + work.brush * s.brush_relief.0;
                layer.roughness =
                    s.palette[t].appearance().roughness.0 + work.brush * PAINT_ROUGHNESS_VARIATION;
                layer.metallic = 0.0;
            }
            MetalFinish::WaterGilding { burnish } => {
                layer.height *= 1.0 - BURNISH_SMOOTHING * burnish.0;
                layer.roughness =
                    WATER_ROUGHNESS + (BURNISHED_ROUGHNESS - WATER_ROUGHNESS) * burnish.0;
            }
            MetalFinish::OilGilding | MetalFinish::YellowGlazedSilver { .. } => {
                layer.height += paint * s.pigment.0
                    + work.brush * OIL_BRUSH_RELIEF_MM
                    + work.crease * OIL_CREASE_RELIEF_MM;
                if let MetalFinish::YellowGlazedSilver { depth } = finish {
                    layer.color = std::array::from_fn(|c| {
                        SILVER_REFLECTANCE[c] * (-2.0 * YELLOW_ABSORPTION[c] * depth.0).exp()
                    });
                    // Even a colorless glaze at depth zero has a dielectric surface.
                    layer.coat = 1.0;
                }
            }
            MetalFinish::MordantGilding { relief } => {
                layer.height +=
                    paint * s.pigment.0 + relief.0 * (1.0 + work.brush * MORDANT_BRUSH_VARIATION);
                layer.roughness = MORDANT_ROUGHNESS;
            }
        }
        layer.roughness = layer.roughness.clamp(0.04, 1.0);
        layer
    }
}
