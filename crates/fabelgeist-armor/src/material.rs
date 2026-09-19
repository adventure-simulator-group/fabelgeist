use crate::engraving::{EngravedTile, Engraving};
use serde::{Deserialize, Serialize};

/// Scratched steel: a base color and roughness with a tiling scratch map, and
/// an optional engraving cut into the same tile. Every rigid armor piece is
/// shaded with it, whatever its construction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metal {
    pub color: [f32; 3],
    pub roughness: f32,
    pub scratch_density: u32,
    pub scratch_length: f32,
    pub scratch_width: f32,
    pub scratch_depth: f32,
    pub scratch_angle: f32,
    pub scratch_spread: f32,
    pub seed: u32,
    pub engraving: Option<Engraving>,
}
impl Default for Metal {
    /// Polished steel.
    fn default() -> Self {
        Self {
            color: [0.769, 0.776, 0.776],
            roughness: 0.20,
            scratch_density: 250,
            scratch_length: 0.045,
            scratch_width: 0.65,
            scratch_depth: 0.06,
            scratch_angle: 0.4,
            scratch_spread: 2.0,
            seed: 1544,
            engraving: None,
        }
    }
}
impl Metal {
    /// Texture repeats per metre of armor surface.
    pub const TILES_PER_METRE: f32 = 4.0;
    /// The texture size previews and exports bake.
    pub const TEXTURE_SIZE: u32 = 512;
    /// Roughness added at the bottom of the deepest scratch.
    const SCRATCH_ROUGHNESS: f32 = 0.45;

    pub fn validate(&self) -> Result<(), String> {
        let values = [
            (self.roughness, 0.08, 0.9),
            (self.scratch_length, 0.005, 0.4),
            (self.scratch_width, 0.5, 3.0),
            (self.scratch_depth, 0.0, 1.0),
            (
                self.scratch_angle,
                -std::f32::consts::PI,
                std::f32::consts::PI,
            ),
            (self.scratch_spread, 0.0, std::f32::consts::PI),
        ];
        if self.scratch_density > 3000
            || values
                .iter()
                .any(|(x, l, h)| !x.is_finite() || x < l || x > h)
            || self
                .color
                .iter()
                .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
        {
            return Err("Invalid metal parameters".into());
        }
        self.engraving.as_ref().map_or(Ok(()), Engraving::validate)
    }

    /// Bake the metal's maps onto a `size` square tile, reading the engraving
    /// image if there is one.
    pub fn textures(&self, size: u32) -> Result<MetalTextures, String> {
        self.validate()?;
        if !(32..=1024).contains(&size) {
            return Err("Texture size must be 32–1024".into());
        }
        let engraved = self
            .engraving
            .as_ref()
            .map(|engraving| {
                engraving
                    .load()
                    .map(|image| engraving.rasterize(&image, size))
            })
            .transpose()?;
        Ok(self.bake(size, engraved.as_ref()))
    }

    /// Bake the maps with an already rasterized engraving.
    fn bake(&self, size: u32, engraved: Option<&EngravedTile>) -> MetalTextures {
        let n = size as usize;
        let mut state = self.seed;
        let mut random = || {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            (state >> 8) as f32 / 16777216.0
        };
        let scratches = self.scratches(size, &mut random);
        let mut normal = Vec::with_capacity(n * n * 4);
        let mut metal_roughness = Vec::with_capacity(n * n * 4);
        let mut depth = Vec::with_capacity(n * n * 4);
        for y in 0..n {
            for x in 0..n {
                let index = y * n + x;
                let h = scratches[index];
                let mut dx =
                    (scratches[y * n + (x + 1) % n] - scratches[y * n + (x + n - 1) % n]) * 2.0;
                let mut dy =
                    (scratches[((y + 1) % n) * n + x] - scratches[((y + n - 1) % n) * n + x]) * 2.0;
                let (recess, recess_roughness) = engraved.map_or((0.0, 0.0), |tile| {
                    dx += tile.slopes[index][0];
                    dy += tile.slopes[index][1];
                    (tile.recess[index], tile.recess_roughness)
                });
                // glTF tangent space: +X right, +Y up the image, so a surface
                // rising to the right or down the image tilts the normal away.
                let length = (dx * dx + dy * dy + 1.0).sqrt();
                normal.extend([
                    ((-dx / length * 0.5 + 0.5) * 255.0) as u8,
                    ((dy / length * 0.5 + 0.5) * 255.0) as u8,
                    ((1.0 / length * 0.5 + 0.5) * 255.0) as u8,
                    255,
                ]);
                let roughness = (self.roughness
                    + h * Self::SCRATCH_ROUGHNESS
                    + recess * recess_roughness
                    + (random() - 0.5) * 0.025)
                    .clamp(0.0, 1.0);
                metal_roughness.extend([255, (roughness * 255.0) as u8, 255, 255]);
                let cut = (recess * 255.0) as u8;
                depth.extend([cut, cut, cut, 255]);
            }
        }
        MetalTextures {
            size,
            normal,
            metal_roughness,
            depth: engraved
                .and_then(|tile| tile.depth_uv)
                .map(|uv_scale| DepthMap {
                    pixels: depth,
                    uv_scale,
                }),
        }
    }

    /// The scratch height field, 0 on the untouched surface.
    fn scratches(&self, size: u32, random: &mut impl FnMut() -> f32) -> Vec<f32> {
        let n = size as usize;
        let mut height = vec![0.0f32; n * n];
        for _ in 0..self.scratch_density {
            let x = random() * size as f32;
            let y = random() * size as f32;
            let angle = self.scratch_angle + (random() - 0.5) * self.scratch_spread * 2.0;
            let length = self.scratch_length * size as f32 * (0.3 + random() * 0.7);
            let depth = self.scratch_depth * (0.4 + random() * 0.6);
            let steps = (length * 2.0).ceil() as u32;
            for step in 0..=steps {
                let t = step as f32 / steps.max(1) as f32;
                let px = x + angle.cos() * length * t;
                let py = y + angle.sin() * length * t;
                for dy in -3..=3 {
                    for dx in -3..=3 {
                        let ix = px.floor() as i32 + dx;
                        let iy = py.floor() as i32 + dy;
                        let distance = ((ix as f32 - px).powi(2) + (iy as f32 - py).powi(2)).sqrt();
                        let groove = (1.0 - distance / self.scratch_width).max(0.0)
                            * depth
                            * (std::f32::consts::PI * t).sin();
                        let index = iy.rem_euclid(size as i32) as usize * n
                            + ix.rem_euclid(size as i32) as usize;
                        height[index] = height[index].max(groove);
                    }
                }
            }
        }
        height
    }
}

pub struct MetalTextures {
    pub size: u32,
    /// Tangent-space normals, RGBA8.
    pub normal: Vec<u8>,
    /// glTF metallic-roughness, RGBA8 with roughness in green.
    pub metal_roughness: Vec<u8>,
    /// The engraving's cut below the surface, when it has a depth.
    pub depth: Option<DepthMap>,
}

/// A parallax depth map: 0 on the surface, 255 at the floor of the cut.
pub struct DepthMap {
    pub pixels: Vec<u8>,
    /// The depth of a full cut in texture coordinate units.
    pub uv_scale: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engraving::{Relief, tests::groove_png};

    const FLAT_NORMAL: [u8; 4] = [127, 127, 255, 255];

    #[test]
    fn scratches_are_deterministic_and_change_normal_and_roughness() {
        let mut m = Metal::default();
        let a = m.textures(64).unwrap();
        let b = m.textures(64).unwrap();
        assert_eq!(a.normal, b.normal);
        assert_eq!(a.metal_roughness, b.metal_roughness);
        assert!(a.depth.is_none());
        m.seed += 1;
        assert_ne!(a.normal, m.textures(64).unwrap().normal);
        m.scratch_density = 0;
        let smooth = m.textures(64).unwrap();
        assert!(smooth.normal.chunks_exact(4).all(|p| p == FLAT_NORMAL));
        assert_ne!(a.normal, smooth.normal);
        assert_ne!(a.metal_roughness, smooth.metal_roughness);
        assert!(m.textures(u32::MAX).is_err());
    }

    #[test]
    fn a_groove_rising_to_the_right_tilts_the_normal_left() {
        let mut m = Metal::default();
        m.scratch_density = 0;
        let mut engraving = Engraving::new("groove.png");
        engraving.relief = Relief::Height {
            depth: Relief::MAX_DEPTH,
        };
        let tile = engraving.rasterize(&engraving.decode(&groove_png()).unwrap(), 64);
        let baked = m.bake(64, Some(&tile));
        let normal = |x: usize| &baked.normal[x * 4..x * 4 + 4];
        // Descending into the groove the surface falls to the right: normal
        // tilts right. Climbing out it rises to the right: normal tilts left.
        assert!(
            normal(24)[0] > 140 && normal(40)[0] < 115,
            "{:?} {:?}",
            normal(24),
            normal(40)
        );
        assert_eq!(normal(0), FLAT_NORMAL);
        let depth = baked.depth.expect("a height map has depth");
        assert!((depth.uv_scale - Relief::MAX_DEPTH * Metal::TILES_PER_METRE).abs() < 1e-6);
        assert!(depth.pixels[32 * 4] > 250 && depth.pixels[0] < 3);
        // The floor of the cut is rougher than the polished surface.
        let roughness = |x: usize| baked.metal_roughness[x * 4 + 1];
        assert!(roughness(32) > roughness(0) + 60);
        m.engraving = Some(engraving);
        assert!(m.textures(32).is_err(), "the image path does not exist");
    }
}
