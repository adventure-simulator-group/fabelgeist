use serde::{Deserialize, Serialize};

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
}
impl Default for Metal {
    fn default() -> Self {
        Self {
            color: [0.62, 0.65, 0.68],
            roughness: 0.28,
            scratch_density: 250,
            scratch_length: 0.045,
            scratch_width: 0.65,
            scratch_depth: 0.06,
            scratch_angle: 0.4,
            scratch_spread: 2.0,
            seed: 1544,
        }
    }
}
impl Metal {
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
        Ok(())
    }
    pub fn textures(&self, size: u32) -> Result<MetalTextures, String> {
        self.validate()?;
        if !(32..=1024).contains(&size) {
            return Err("Texture size must be 32–1024".into());
        }
        let n = size as usize;
        let mut state = self.seed;
        let mut random = || {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            (state >> 8) as f32 / 16777216.0
        };
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
        let mut normal = Vec::with_capacity(n * n * 4);
        let mut metal_roughness = Vec::with_capacity(n * n * 4);
        for y in 0..n {
            for x in 0..n {
                let h = height[y * n + x];
                let dx = (height[y * n + (x + 1) % n] - height[y * n + (x + n - 1) % n]) * 2.0;
                let dy = (height[((y + 1) % n) * n + x] - height[((y + n - 1) % n) * n + x]) * 2.0;
                let length = (dx * dx + dy * dy + 1.0).sqrt();
                normal.extend([
                    ((dx / length * 0.5 + 0.5) * 255.0) as u8,
                    ((dy / length * 0.5 + 0.5) * 255.0) as u8,
                    ((1.0 / length * 0.5 + 0.5) * 255.0) as u8,
                    255,
                ]);
                let roughness =
                    (self.roughness + h * 0.45 + (random() - 0.5) * 0.025).clamp(0.0, 1.0);
                metal_roughness.extend([255, (roughness * 255.0) as u8, 255, 255]);
            }
        }
        Ok(MetalTextures {
            size,
            normal,
            metal_roughness,
        })
    }
}
pub struct MetalTextures {
    pub size: u32,
    pub normal: Vec<u8>,
    pub metal_roughness: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scratches_are_deterministic_and_change_normal_and_roughness() {
        let mut m = Metal::default();
        let a = m.textures(64).unwrap();
        let b = m.textures(64).unwrap();
        assert_eq!(a.normal, b.normal);
        assert_eq!(a.metal_roughness, b.metal_roughness);
        m.seed += 1;
        assert_ne!(a.normal, m.textures(64).unwrap().normal);
        m.scratch_density = 0;
        let smooth = m.textures(64).unwrap();
        assert!(
            smooth
                .normal
                .chunks_exact(4)
                .all(|p| p == [127, 127, 255, 255])
        );
        assert_ne!(a.normal, smooth.normal);
        assert_ne!(a.metal_roughness, smooth.metal_roughness);
        assert!(m.textures(u32::MAX).is_err());
    }
}
