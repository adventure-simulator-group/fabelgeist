//! Native bilinear sampling and its signed-address admission contract.

use super::{Engraving, EngravingError, ReliefImage};
use crate::material::MetalTextureSize;

pub(crate) const WRAP: &str = r#"fn wrap(i: i32, n: u32) -> u32 {
    let m = i32(n);
    return u32(((i % m) + m) % m);
}"#;

pub(crate) const COORDINATES: &str = r#"    let size = f32(n);
    let px = host_fence(host_div(f32(index % n) + 0.5, size) - 0.5);
    let py = host_fence(host_div(f32(index / n) + 0.5, size) - 0.5);
    let s = params.sin_rotation;
    let c = params.cos_rotation;
    // Turn the tile back into the image, then repeat the image.
    let qx = host_fence(host_mul(c, px) + host_mul(s, py));
    let qy = host_fence(host_mul(-s, px) + host_mul(c, py));
    let width = f32(params.image_width);
    let height = f32(params.image_height);
    let x = host_fence(host_mul(host_fence(host_mul(qx, params.tiles) + 0.5), width) - 0.5);
    let y = host_fence(host_mul(host_fence(host_mul(qy, params.tiles) + 0.5), height) - 0.5);
"#;

/// One native sampler row: coefficients in normalized tile coordinates.
#[derive(Clone, Copy)]
struct SampleAxis {
    coefficients: [f32; 2],
    image_side: u32,
}

impl ReliefImage {
    pub(super) fn admit_samples(
        &self,
        size: MetalTextureSize,
        engraving: &Engraving,
        coefficients: [f32; 2],
    ) -> Result<(), EngravingError> {
        let [width, height] = self.dimensions();
        let [sine, cosine] = coefficients;
        let side = f64::from(u32::from(size));
        // These are the native tile's actual first and last pixel centers.
        // The denominator is at most 1024; rounding the quotient to f32 here
        // agrees with the shader's host_div before its fenced subtraction.
        let first = (0.5 / side) as f32;
        let last = ((side - 0.5) / side) as f32;
        let low = (f64::from(first) - 0.5) as f32;
        let high = (f64::from(last) - 0.5) as f32;
        let axes = [
            SampleAxis {
                coefficients: [cosine, sine],
                image_side: width,
            },
            SampleAxis {
                coefficients: [-sine, cosine],
                image_side: height,
            },
        ];
        // Each fenced affine operation is monotone in either coordinate;
        // therefore the four corners include both extrema on each axis.
        for point in [[low, low], [low, high], [high, low], [high, high]] {
            for axis in axes {
                let coordinate = axis.coordinate(point, engraving.tiles);
                let first = f64::from(coordinate.floor());
                if !coordinate.is_finite()
                    || first < f64::from(i32::MIN)
                    || first + 1.0 > f64::from(i32::MAX)
                {
                    return Err(EngravingError::SampleAddress {
                        width,
                        height,
                        size,
                    });
                }
            }
        }
        Ok(())
    }
}

impl SampleAxis {
    fn coordinate(self, point: [f32; 2], tiles: f32) -> f32 {
        let [horizontal, vertical] = point;
        let [first, second] = self.coefficients;
        // A product of f32 operands is exact in f64. Explicit f32 projections
        // materialize each fenced shader operation without host contraction.
        let horizontal = (f64::from(first) * f64::from(horizontal)) as f32;
        let vertical = (f64::from(second) * f64::from(vertical)) as f32;
        let rotated = (f64::from(horizontal) + f64::from(vertical)) as f32;
        let tiled = (f64::from(rotated) * f64::from(tiles)) as f32;
        let centered = (f64::from(tiled) + 0.5) as f32;
        let pixel = (f64::from(centered) * f64::from(self.image_side as f32)) as f32;
        (f64::from(pixel) - 0.5) as f32
    }
}

pub(super) fn admit_extent(width: u32, height: u32) -> Result<(), EngravingError> {
    // wrap(i,n) uses signed ((i % n) + n) % n. Both n and its largest
    // positive remainder plus n must fit, independently of sample casts.
    if width == 0
        || height == 0
        || 2 * u64::from(width) - 1 > i32::MAX as u64
        || 2 * u64::from(height) - 1 > i32::MAX as u64
    {
        return Err(EngravingError::ImageExtent { width, height });
    }
    Ok(())
}
