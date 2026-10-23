//! A synthetic dataset in DiffLocks' image: randomized grooms, their
//! strands, their scalp density, and a picture of each.
//!
//! DiffLocks made 40K samples by running its Blender pipeline with 110
//! parameters drawn at random -- base style, geometry, material -- then
//! rendering each through a path tracer under a random HDRI, with a random
//! camera (focal length 40 to 150 mm, aimed at the head). A [`Sample`] is one
//! such draw here: the preset and its [`Look`], the material, the camera and
//! the light, all recorded, so a sample can be grown again exactly.
//!
//! On disk the dataset is laid out as DiffLocks' is ([`crate::difflocks`]),
//! so its loader reads ours: `body_data/scalp.ply` (the mannequin's scalp,
//! with the UV map every root's `root_uv` is on) and the mannequin as
//! `body_data/mannequin.ply` -- DiffLocks has the SMPL-X body there, as
//! `smplx_base.ply` -- and a directory a sample,
//! `generated_hairstyles/base_<preset>_idx_<index>/`, of
//!
//! * `full_strands.npz` -- `positions` (strands x 256 x 3), `root_uv`
//!   (strands x 2) and `root_normal` (strands x 3), `float32`, metres, `y`
//!   up. Positions are in the head's frame ([`crate::Head`]: origin between
//!   the ears, face towards `+z`), not SMPL-X's, whose head is 30 cm higher.
//! * `guide_strands.npz` -- the guides the sample grew from, likewise.
//! * `density.png` -- the scalp's strand density (DiffLocks' density map):
//!   roots per texel of the scalp UV map, blurred, normalised to 0 to 255.
//! * `metadata.json` -- DiffLocks' keys for the material
//!   ([`Sample::difflocks_metadata`]), and everything the sample was made
//!   from under its own keys.
//! * `cam.npz` -- the camera: `RT` (world to camera) and `K` (pixels).
//! * `rgb.png` -- the render, when there is a GPU to make it.
//! * `numpy_state.npz` -- last, as DiffLocks writes it: the sample is
//!   complete. Here it holds the seed and the index.

use serde::{Deserialize, Serialize};

use crate::lod::{AccumulationFrames, LodPolicy};
use crate::look::Look;
use crate::material::HairMaterial;
use crate::presets::Preset;
use crate::rng::Rng;
use crate::strands::Strands;

/// Where a sample's picture is taken from and how it is lit.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shot {
    /// Around the head from the front, and above the horizon, radians.
    pub yaw: f32,
    pub pitch: f32,
    /// Focal length on a 36 mm frame, millimetres.
    pub focal_mm: f32,
    /// How much of the frame the head fills, about one.
    pub framing: f32,
    /// The key light: around and above, radians, and its strength.
    pub key_azimuth: f32,
    pub key_elevation: f32,
    pub key_intensity: f32,
    pub exposure: f32,
    pub skin: [f32; 3],
}

impl Shot {
    pub fn random(rng: &mut Rng) -> Self {
        let skin_tone = rng.next_f32();
        let light_skin = [0.72, 0.53, 0.43];
        let dark_skin = [0.22, 0.13, 0.09];
        Self {
            yaw: rng.range(-0.7, 0.7),
            pitch: rng.range(-0.1, 0.3),
            focal_mm: rng.range(40.0, 150.0),
            framing: rng.range(0.9, 1.3),
            key_azimuth: rng.range(-2.2, 2.2),
            key_elevation: rng.range(0.1, 1.2),
            key_intensity: rng.range(1.5, 4.0),
            exposure: rng.range(0.6, 1.1),
            skin: std::array::from_fn(|k| {
                light_skin[k] + (dark_skin[k] - light_skin[k]) * skin_tone
            }),
        }
    }

    /// Vertical field of view, radians, on a 36 mm square frame.
    pub fn fov_y(&self) -> f32 {
        2.0 * (18.0 / self.focal_mm).atan()
    }

    /// Camera distance for the head to fill the frame as asked.
    pub fn distance(&self) -> f32 {
        0.2 * self.framing / (self.fov_y() * 0.5).tan()
    }
}

/// One draw of the dataset.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    pub index: u64,
    pub seed: u64,
    pub preset: Preset,
    pub look: Look,
    pub material: HairMaterial,
    pub shot: Shot,
}

impl Sample {
    /// Sample `index` of a dataset seeded `seed`, from `presets`.
    pub fn draw(seed: u64, index: u64, presets: &[Preset]) -> Self {
        let mut rng = Rng::new(seed ^ index.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let preset = presets[rng.index(presets.len())];
        let look = Look::random(preset, &mut rng);
        let material = HairMaterial {
            // Darker hair is the more common.
            melanin: rng.next_f32().powf(0.6),
            redness: rng.range(0.0, 1.0).powf(1.5),
            tint: if rng.chance(0.08) {
                [
                    rng.range(0.2, 1.0),
                    rng.range(0.2, 1.0),
                    rng.range(0.2, 1.0),
                ]
            } else {
                [1.0; 3]
            },
            roughness: rng.range(0.2, 0.45),
            radial_roughness: rng.range(0.25, 0.4),
            random_color: rng.range(0.05, 0.2),
            random_roughness: rng.range(0.1, 0.3),
            tip_lightness: rng.range(0.0, 0.35),
            grey: if rng.chance(0.12) {
                rng.range(0.1, 0.9)
            } else {
                0.0
            },
            ..Default::default()
        };
        let mut material = material;
        // DiffLocks' variations along the strand: roots darker, melanin in
        // waves, now and then a coat.
        if rng.chance(0.4) {
            material.root_darkness_start = rng.range(0.0, 0.05);
            material.root_darkness_end = material.root_darkness_start + rng.range(0.05, 0.3);
            material.root_darkness_strength = rng.range(0.1, 0.8);
        }
        if rng.chance(0.35) {
            material.wave_scale = rng.range(1.0, 30.0);
            material.wave_phase = rng.range(0.0, 360.0);
            material.wave_strength = rng.range(0.05, 0.3);
        }
        if rng.chance(0.2) {
            material.coat = rng.range(0.0, 0.4);
        }
        Self {
            index,
            seed,
            preset,
            look,
            material,
            shot: Shot::random(&mut rng),
        }
    }

    /// The material under the names DiffLocks' `metadata.json` gives it,
    /// which its loaders read.
    pub fn difflocks_metadata(&self) -> serde_json::Value {
        let m = &self.material;
        serde_json::json!({
            "mirror_hair": false,
            "material_melanin_amount": m.melanin,
            "bsdf_melanin_redness": m.redness,
            "bsdf_roughness": m.roughness,
            "bsdf_radial_roughness": m.radial_roughness,
            "bsdf_coat": m.coat,
            "root_darkness_start": m.root_darkness_start,
            "root_darkness_end": m.root_darkness_end,
            "root_darkness_strength": m.root_darkness_strength,
            "material_wave_scale": m.wave_scale,
            "material_wave_phase_offset": m.wave_phase,
            "material_wave_strength": m.wave_strength,
        })
    }

    /// The camera's eye and target, in the head's frame.
    pub fn eye(&self) -> ([f32; 3], [f32; 3]) {
        let shot = &self.shot;
        let d = shot.distance();
        (
            [
                d * shot.pitch.cos() * shot.yaw.sin(),
                d * shot.pitch.sin(),
                d * shot.pitch.cos() * shot.yaw.cos(),
            ],
            [0.0; 3],
        )
    }

    /// The policy a sample is drawn with: every strand, accumulated.
    pub fn policy(frames: AccumulationFrames) -> LodPolicy {
        let mut policy = LodPolicy::cinematic();
        policy.quality.accumulate = frames.target();
        policy
    }
}

/// The scalp's strand density: roots per texel of a `size` square over the
/// UV map, blurred once, scaled so the densest texel is 255.
pub fn density_map(strands: &Strands, size: usize) -> Vec<u8> {
    let mut counts = vec![0.0f32; size * size];
    for uv in &strands.root_uv {
        let x = ((uv[0].clamp(0.0, 1.0) * size as f32) as usize).min(size - 1);
        let y = ((uv[1].clamp(0.0, 1.0) * size as f32) as usize).min(size - 1);
        counts[y * size + x] += 1.0;
    }
    let mut blurred = vec![0.0f32; size * size];
    for y in 0..size {
        for x in 0..size {
            let mut sum = 0.0;
            let mut n = 0.0;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                    if xx >= 0 && yy >= 0 && (xx as usize) < size && (yy as usize) < size {
                        sum += counts[yy as usize * size + xx as usize];
                        n += 1.0;
                    }
                }
            }
            blurred[y * size + x] = sum / n;
        }
    }
    let max = blurred.iter().copied().fold(0.0, f32::max).max(1e-6);
    blurred
        .iter()
        .map(|v| (v / max * 255.0).round() as u8)
        .collect()
}

/// A grey image as binary PGM.
pub fn pgm(width: usize, height: usize, pixels: &[u8]) -> Vec<u8> {
    let mut out = format!("P5\n{width} {height}\n255\n").into_bytes();
    out.extend_from_slice(pixels);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_are_reproducible_and_varied() {
        let a = Sample::draw(7, 3, &Preset::ALL);
        let b = Sample::draw(7, 3, &Preset::ALL);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        let presets: std::collections::HashSet<_> = (0..40)
            .map(|i| Sample::draw(7, i, &Preset::ALL).preset)
            .collect();
        assert!(presets.len() > 5);
        let shot = &a.shot;
        assert!((40.0..=150.0).contains(&shot.focal_mm));
        assert!(
            shot.distance() > 0.2 && shot.distance() < 3.0,
            "{}",
            shot.distance()
        );
    }
}
