use super::*;
use crate::presentation::grass_cover_mask_pixels;

/// CPU-side sampler over the same feathered cover mask the legacy renderer
/// binds as a texture.
pub(super) struct CoverageMask {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    ground_width: f32,
    ground_depth: f32,
}

impl CoverageMask {
    pub(super) fn new(ground: &SceneGround, seed: u64) -> Self {
        let (width, height, pixels) = grass_cover_mask_pixels(ground, seed);
        Self {
            width: width as usize,
            height: height as usize,
            pixels,
            ground_width: ground.width(),
            ground_depth: ground.depth(),
        }
    }

    pub(super) fn coverage_byte(&self, world: Vec2) -> u8 {
        let u = (world.x / self.ground_width + 0.5).clamp(0.0, 1.0);
        let v = (world.y / self.ground_depth + 0.5).clamp(0.0, 1.0);
        let x = ((u * self.width as f32) as usize).min(self.width - 1);
        let y = ((v * self.height as f32) as usize).min(self.height - 1);
        self.pixels[y * self.width + x]
    }
}
