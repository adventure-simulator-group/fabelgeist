//! Tangent-space relief for raised metallic thread ornament.
use crate::{DesignError, GenerateError, TextileAppearance, TextileFinish, TextilePixels};

const THREAD_EDGE_SLOPE: f32 = 0.65;

impl TextilePixels {
    /// Height-field central differences, in metres per material-chart metre.
    /// Fine cloth relief composes with metallic-thread slopes when both apply.
    pub fn surface_normals(&self, fabric: &TextileAppearance) -> Result<Vec<u8>, GenerateError> {
        self.validate_dimensions()?;
        let mut result = if fabric.finish == TextileFinish::MetallicThread {
            self.thread_normals()?
        } else {
            [128, 128, 255, 255].repeat((self.width * self.height) as usize)
        };
        let Some(relief) = &fabric.relief else {
            return Ok(result);
        };
        relief.validate()?;
        let heights = (0..self.height)
            .flat_map(|row| {
                (0..self.width).map(move |column| {
                    relief.height([
                        (column as f32 + 0.5) / self.width as f32,
                        (row as f32 + 0.5) / self.height as f32,
                    ])
                })
            })
            .collect::<Vec<_>>();
        let sample = |x: u32, y: u32| heights[(y * self.width + x) as usize];
        for y in 0..self.height {
            for x in 0..self.width {
                let slope_x = (sample((x + 1) % self.width, y)
                    - sample((x + self.width - 1) % self.width, y))
                    * self.width as f32
                    / (2. * relief.chart.across.metres());
                let slope_y = (sample(x, (y + 1).min(self.height - 1))
                    - sample(x, y.saturating_sub(1)))
                    * self.height as f32
                    / (2. * relief.chart.along.metres());
                let pixel = &mut result[((y * self.width + x) * 4) as usize..][..4];
                let base = [pixel[0], pixel[1], pixel[2]].map(|channel| {
                    if channel == 128 {
                        0.
                    } else {
                        f32::from(channel) / 127.5 - 1.
                    }
                });
                let normal = [base[0] / base[2] - slope_x, base[1] / base[2] - slope_y, 1.];
                let length = normal.iter().map(|value| value * value).sum::<f32>().sqrt();
                for channel in 0..3 {
                    pixel[channel] = ((normal[channel] / length * 0.5 + 0.5) * 255.).round() as u8;
                }
            }
        }
        Ok(result)
    }

    /// Native RGBA8 normal image. Ornament brightness supplies a raised thread
    /// mask; this is a finish approximation, not measured yarn displacement.
    /// U wraps and V clamps, matching the shared garment texture sampler.
    pub fn thread_normals(&self) -> Result<Vec<u8>, GenerateError> {
        self.validate_dimensions()?;
        self.draw_thread_normals()
    }

    fn validate_dimensions(&self) -> Result<(), GenerateError> {
        let expected = (self.width as usize)
            .checked_mul(self.height as usize)
            .and_then(|count| count.checked_mul(4));
        if self.width == 0 || self.height == 0 || expected != Some(self.color.len()) {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }

    fn draw_thread_normals(&self) -> Result<Vec<u8>, GenerateError> {
        let heights = self
            .color
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[..3].iter().map(|v| f32::from(*v)).sum::<f32>())
            .collect::<Vec<_>>();
        let low = heights.iter().copied().fold(f32::INFINITY, f32::min);
        let high = heights.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        if high <= low {
            return Ok([128, 128, 255, 255].repeat(heights.len()));
        }
        let sample = |x: u32, y: u32| heights[(y * self.width + x) as usize];
        let mut result = Vec::with_capacity(self.color.len());
        for y in 0..self.height {
            for x in 0..self.width {
                let slope_x = (sample((x + 1) % self.width, y)
                    - sample((x + self.width - 1) % self.width, y))
                    * THREAD_EDGE_SLOPE
                    / (high - low);
                let slope_y = (sample(x, (y + 1).min(self.height - 1))
                    - sample(x, y.saturating_sub(1)))
                    * THREAD_EDGE_SLOPE
                    / (high - low);
                let length = (slope_x * slope_x + slope_y * slope_y + 1.).sqrt();
                for value in [-slope_x / length, -slope_y / length, 1. / length] {
                    result.push(((value * 0.5 + 0.5) * 255.).round() as u8);
                }
                result.push(255);
            }
        }
        Ok(result)
    }
}
