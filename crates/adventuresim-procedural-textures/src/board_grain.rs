//! Local growth rings for cut boards: one coordinate drives relief and pigment.
use crate::{
    TextureParameters,
    stamps::{hash, noise, smooth},
};
use bevy::math::{IVec2, Vec2};
#[cfg(test)]
mod tests;

crate::parameters::parameter_block! {
    pub struct Parameters {
        ring_count: i32 = 9;
        ring_wander: f32 = 0.035;
        ring_width: f32 = 0.22;
        ring_depth: f32 = 0.014;
        dark_ring_fraction: f32 = 0.55;
        knot_fraction: f32 = 0.35;
        knot_radius: [f32; 2] = [0.13, 0.10];
        knot_flow: f32 = 1.4;
        knot_influence: f32 = 3.0;
        knot_core: f32 = 0.25;
        knot_margin: f32 = 0.2;
        ring_width_variation: f32 = 0.3;
        ring_shoulder: f32 = 0.5;
        wander_cells: [i32; 2] = [3, 7];
        sawn_arch_fraction: f32 = 0.4;
        sawn_arch_depth: f32 = 0.2;
        sawn_arch_scale: f32 = 0.35;
        fiber_count: i32 = 43;
        fiber_depth: f32 = 0.006;
        light_srgb: [u8; 3] = [115, 82, 48];
        dark_srgb: [u8; 3] = [85, 58, 32];
    }
}

#[derive(Default)]
pub(crate) struct Sample {
    pub height: f32,
    pub dark: f32,
}

impl Parameters {
    fn at(&self, params: &TextureParameters, uv: Vec2, id: u64) -> Sample {
        let random = |salt| hash(params, IVec2::ZERO, IVec2::ONE, id ^ salt);
        let mut across = uv.x;
        if random(0x175a) < self.knot_fraction {
            let center = Vec2::new(
                self.knot_margin + random(0x75ad) * (1.0 - 2.0 * self.knot_margin),
                random(0xb43d),
            );
            let d = uv - center;
            let d = Vec2::new(d.x, d.y - d.y.round());
            let radius = (d / Vec2::from_array(self.knot_radius)).length_squared();
            let envelope = 1.0 - smooth(radius.sqrt() / self.knot_influence);
            across -= d.x / (radius + self.knot_core) * self.knot_flow * envelope;
        }
        if random(0x7943) < self.sawn_arch_fraction {
            // An oblique longitudinal cut opens rings into cathedral arches.
            // Periodic sine-squared distances made repeated closed target motifs.
            let center = random(0x1247);
            let along = (uv.y - random(0x4217)) * self.sawn_arch_scale;
            across = ((across - center).powi(2) + self.sawn_arch_depth.powi(2)).sqrt() + along;
        }
        let warp = noise(
            params,
            Vec2::new(across, uv.y),
            IVec2::from_array(self.wander_cells),
            id ^ 0x73ab,
        ) - 0.5;
        let coordinate = across + warp * self.ring_wander;
        let phase = coordinate * self.ring_count as f32 + random(0x41ab);
        let ring = phase.floor() as i32;
        let width = self.ring_width
            * (1.0 - self.ring_width_variation
                + hash(
                    params,
                    IVec2::new(ring, 0),
                    IVec2::new(4096, 1),
                    id ^ 0x1249,
                ) * 2.0
                    * self.ring_width_variation);
        let t = phase.rem_euclid(1.0);
        let ridge = smooth(t / width) * (1.0 - smooth((t - width) / (width * self.ring_shoulder)));
        let colored = hash(
            params,
            IVec2::new(ring, 0),
            IVec2::new(4096, 1),
            id ^ 0x1731,
        ) < self.dark_ring_fraction;
        let dark = u8::from(colored && t < width * (1.0 + self.ring_shoulder)) as f32;
        let fiber = (coordinate * self.fiber_count as f32 * std::f32::consts::TAU).sin();
        Sample {
            height: ridge * self.ring_depth + fiber * self.fiber_depth,
            dark,
        }
    }

    pub(crate) fn filtered(
        &self,
        params: &TextureParameters,
        uv: Vec2,
        footprint: Vec2,
        id: u64,
    ) -> Sample {
        let mut out = Sample::default();
        for y in 0..4 {
            for x in 0..4 {
                let offset =
                    (Vec2::new(x as f32, y as f32) + Vec2::splat(0.5)) / 4.0 - Vec2::splat(0.5);
                let s = self.at(params, uv + offset * footprint, id);
                out.height += s.height / 16.0;
                out.dark += s.dark / 16.0;
            }
        }
        out
    }
    pub(crate) fn color(&self, dark: f32) -> [u8; 3] {
        crate::SrgbColor(self.light_srgb)
            .covered_by(crate::SrgbColor(self.dark_srgb), dark)
            .0
    }
}
