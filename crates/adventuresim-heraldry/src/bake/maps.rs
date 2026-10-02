//! Shared sampling rules for GPU previews and exported map chains.
use super::unit_byte;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureKind {
    Color,
    Normal,
    Linear,
}
pub struct MipLevel {
    pub size: u32,
    pub rgba: Vec<u8>,
}
pub fn srgb_to_linear(v: u8) -> f32 {
    let v = f32::from(v) / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
pub fn linear_to_srgb(v: f32) -> u8 {
    unit_byte(if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.max(0.0).powf(1.0 / 2.4) - 0.055
    })
}
pub fn mips(rgba: &[u8], size: u32, kind: TextureKind) -> Vec<MipLevel> {
    assert!(size.is_power_of_two() && rgba.len() == (size * size * 4) as usize);
    let mut levels = vec![MipLevel {
        size,
        rgba: rgba.to_vec(),
    }];
    while levels.last().unwrap().size > 1 {
        let prev = levels.last().unwrap();
        let size = prev.size / 2;
        let mut rgba = Vec::with_capacity((size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                let mut sum = [0.0; 4];
                let mut weight = 0.0;
                for dy in 0..2 {
                    for dx in 0..2 {
                        let i = (((y * 2 + dy) * prev.size + x * 2 + dx) * 4) as usize;
                        let a = f32::from(prev.rgba[i + 3]) / 255.0;
                        for (c, channel) in sum.iter_mut().enumerate().take(3) {
                            *channel += match kind {
                                TextureKind::Color => srgb_to_linear(prev.rgba[i + c]) * a,
                                TextureKind::Normal => f32::from(prev.rgba[i + c]) / 127.5 - 1.0,
                                TextureKind::Linear => f32::from(prev.rgba[i + c]) / 255.0,
                            };
                        }
                        sum[3] += a;
                        weight += a;
                    }
                }
                let length = (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2])
                    .sqrt()
                    .max(f32::EPSILON);
                for c in sum.iter().take(3) {
                    rgba.push(match kind {
                        TextureKind::Color => linear_to_srgb(c / weight.max(f32::EPSILON)),
                        TextureKind::Normal => unit_byte(c / length * 0.5 + 0.5),
                        TextureKind::Linear => unit_byte(c * 0.25),
                    });
                }
                rgba.push(unit_byte(sum[3] * 0.25));
            }
        }
        levels.push(MipLevel { size, rgba });
    }
    levels
}
pub(super) fn normals(height: &[f32], size: u32, dimensions: [f32; 2]) -> Vec<u8> {
    let size = size as usize;
    let get = |x: usize, y: usize| height[y * size + x];
    let mut out = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let left = x.saturating_sub(1);
            let right = (x + 1).min(size - 1);
            let up = y.saturating_sub(1);
            let down = (y + 1).min(size - 1);
            let dx = (get(right, y) - get(left, y))
                / ((right - left) as f32 * dimensions[0] / size as f32);
            let dy =
                (get(x, down) - get(x, up)) / ((down - up) as f32 * dimensions[1] / size as f32);
            let length = (dx * dx + dy * dy + 1.0).sqrt();
            out.extend([
                unit_byte(-dx / length * 0.5 + 0.5),
                unit_byte(dy / length * 0.5 + 0.5),
                unit_byte(0.5 / length + 0.5),
                255,
            ]);
        }
    }
    out
}
/// Bounded color dilation keeps transparent texels from introducing dark rims.
pub(super) fn extend_edges(rgba: &mut [u8], size: u32) {
    const GUTTER_PIXELS: usize = 8;
    let size = size as usize;
    let mut valid: Vec<_> = rgba.as_chunks::<4>().0.iter().map(|p| p[3] > 0).collect();
    for _ in 0..GUTTER_PIXELS {
        let prev = valid.clone();
        let old = rgba.to_vec();
        for y in 0..size {
            for x in 0..size {
                let i = y * size + x;
                if prev[i] {
                    continue;
                }
                for (nx, ny) in [
                    (x.saturating_sub(1), y),
                    ((x + 1).min(size - 1), y),
                    (x, y.saturating_sub(1)),
                    (x, (y + 1).min(size - 1)),
                ] {
                    let j = ny * size + nx;
                    if prev[j] {
                        rgba[i * 4..i * 4 + 3].copy_from_slice(&old[j * 4..j * 4 + 3]);
                        valid[i] = true;
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_slopes_use_physical_spacing_and_positive_y_points_up() {
        let height = vec![0.0, 1.0, 2.0, 1.0, 2.0, 3.0, 2.0, 3.0, 4.0];
        let narrow = normals(&height, 3, [3.0, 3.0]);
        let broad = normals(&height, 3, [30.0, 30.0]);
        let center = &narrow[16..20];
        assert!(center[0] < 128 && center[1] > 128);
        assert!(broad[16] > center[0] && broad[17] < center[1]);
        let decode = |v: u8| f32::from(v) / 127.5 - 1.0;
        assert!((center[..3].iter().map(|v| decode(*v).powi(2)).sum::<f32>() - 1.0).abs() < 0.02);
    }
}
