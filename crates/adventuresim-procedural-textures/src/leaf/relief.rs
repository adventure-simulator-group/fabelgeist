//! Surface relief independent of the shared botanical shape and two-color albedo.
use super::LeafShape;
use bevy::math::{Vec2, Vec3};
crate::parameters::parameter_block! {
 pub struct LeafRelief {
  dome: f32 = 0.12;
  vein_height: f32 = 0.055;
  back_vein_height: f32 = 0.085;
  vein_softness: f32 = 0.004;
  curl: f32 = 0.015;
  corrugation: f32 = 0.008;
  corrugation_frequency: f32 = 28.0;
  tissue_height: f32 = 0.003;
  tissue_frequency: f32 = 75.0;
  normal_strength: f32 = 0.06;
  ao_strength: f32 = 0.4;
  roughness_gain: f32 = 1.0;
 }
}
impl LeafRelief {
    pub(super) fn height(&self, p: Vec2, shape: &LeafShape, vein: f32, underside: bool) -> f32 {
        let t = ((shape.half_length - p.y) / (2.0 * shape.half_length)).clamp(0.0, 1.0);
        let across = (p.x - shape.bend * (core::f32::consts::PI * t).sin()) / shape.half_width;
        let envelope = (core::f32::consts::PI * t).sin();
        let dome = (1.0 - across * across).max(0.0) * envelope * self.dome;
        let curl = across * across * envelope * self.curl;
        let ribs = (t * self.corrugation_frequency + across.abs()).sin() * self.corrugation;
        let tissue = (p.x * self.tissue_frequency).sin()
            * (p.y * self.tissue_frequency).cos()
            * self.tissue_height;
        (0.25
            + dome
            + curl
            + ribs
            + tissue
            + vein
                * if underside {
                    self.back_vein_height
                } else {
                    self.vein_height
                })
        .clamp(0.0, 1.0)
    }
    pub(super) fn normal(
        &self,
        heights: &[f32],
        x: usize,
        y: usize,
        size: usize,
        underside: bool,
    ) -> [u8; 4] {
        let left = heights[y * size + x.saturating_sub(1)];
        let right = heights[y * size + (x + 1).min(size - 1)];
        let up = heights[y.saturating_sub(1) * size + x];
        let down = heights[(y + 1).min(size - 1) * size + x];
        let scale = size as f32 * self.normal_strength;
        let n = crate::normal::from_image_gradient((right - left) * scale, (down - up) * scale);
        let n = if underside {
            Vec3::new(n.x, -n.y, n.z)
        } else {
            n
        };
        let v = ((n + Vec3::ONE) * 127.5).round();
        [v.x as u8, v.y as u8, v.z as u8, 255]
    }
}
/// Separable Gaussian filtering turns the shared vein coverage into a rounded ridge.
pub(super) fn soften(values: &[f32], size: usize, sigma: f32) -> Vec<f32> {
    let radius = (sigma * 3.0).ceil() as isize;
    let weights: Vec<f32> = (-radius..=radius)
        .map(|i| (-0.5 * (i as f32 / sigma).powi(2)).exp())
        .collect();
    let total: f32 = weights.iter().sum();
    let mut current = values.to_vec();
    for vertical in [false, true] {
        let mut next = vec![0.0; values.len()];
        for y in 0..size {
            for x in 0..size {
                next[y * size + x] = weights
                    .iter()
                    .enumerate()
                    .map(|(i, w)| {
                        let offset = i as isize - radius;
                        let xx = if vertical {
                            x
                        } else {
                            (x as isize + offset).clamp(0, size as isize - 1) as usize
                        };
                        let yy = if vertical {
                            (y as isize + offset).clamp(0, size as isize - 1) as usize
                        } else {
                            y
                        };
                        current[yy * size + xx] * w / total
                    })
                    .sum();
            }
        }
        current = next;
    }
    current
}
