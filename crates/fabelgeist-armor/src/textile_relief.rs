//! Tangent-space relief for raised metallic thread ornament.
use crate::{DesignError, GenerateError, TextileAppearance, TextileFinish, TextilePixels};

const THREAD_EDGE_SLOPE: f32 = 0.65;

impl TextilePixels {
    /// Height-field central differences, in metres per material-chart metre.
    /// Fine cloth relief composes with metallic-thread slopes when both apply.
    pub fn surface_normals(&self, fabric: &TextileAppearance) -> Result<Vec<u8>, GenerateError> {
        self.validate_dimensions()?;
        fabric.validate()?;
        if fabric.chart_boundary != self.chart_boundary {
            return Err(GenerateError::InvalidSurface);
        }
        if fabric.needlework.is_some()
            && self.needlework_heights.len() != (self.width * self.height) as usize
        {
            return Err(GenerateError::InvalidSurface);
        }
        let mut result = if fabric.finish == TextileFinish::MetallicThread {
            self.thread_normals()?
        } else {
            [128, 128, 255, 255].repeat((self.width * self.height) as usize)
        };
        if fabric.relief.is_none() && fabric.needlework.is_none() {
            return Ok(result);
        }
        let chart = fabric
            .relief
            .as_ref()
            .map(|r| r.chart)
            .or_else(|| fabric.needlework.as_ref().map(|n| n.chart))
            .ok_or(GenerateError::InvalidSurface)?;
        let heights = (0..self.height)
            .flat_map(|row| {
                (0..self.width).map(move |column| {
                    let uv = [
                        (column as f32 + 0.5) / self.width as f32,
                        (row as f32 + 0.5) / self.height as f32,
                    ];
                    fabric.relief.as_ref().map_or(0., |r| r.height(uv))
                        + if fabric.needlework.is_some() {
                            self.needlework_heights[(row * self.width + column) as usize]
                        } else {
                            0.
                        }
                })
            })
            .collect::<Vec<_>>();
        let sample = |x: u32, y: u32| heights[(y * self.width + x) as usize];
        for y in 0..self.height {
            for x in 0..self.width {
                let [left, right] = self.chart_boundary.neighbors(x, self.width);
                let slope_x = (sample(right, y) - sample(left, y)) * self.width as f32
                    / (2. * chart.across.metres());
                let slope_y = (sample(x, (y + 1).min(self.height - 1))
                    - sample(x, y.saturating_sub(1)))
                    * self.height as f32
                    / (2. * chart.along.metres());
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
    /// U follows the explicit chart boundary; V clamps at both cloth ends.
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
                let [left, right] = self.chart_boundary.neighbors(x, self.width);
                let slope_x =
                    (sample(right, y) - sample(left, y)) * THREAD_EDGE_SLOPE / (high - low);
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
