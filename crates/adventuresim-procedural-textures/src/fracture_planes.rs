//! Periodic joined fracture planes with shared corner heights.
use crate::{TextureParameters, stamps::hash};
use bevy::math::{IVec2, Vec2};
crate::parameters::parameter_block! {
    pub struct Parameters {
        cells: [i32; 2] = [11, 9];
        jitter: f32 = 0.85;
        depth: f32 = 0.24;
        roundness: f32 = 0.10;
    }
}
impl Parameters {
    pub(crate) fn sample(&self, params: &TextureParameters, uv: Vec2, salt: u64) -> f32 {
        let cells = IVec2::from_array(self.cells);
        let p = uv.rem_euclid(Vec2::ONE) * cells.as_vec2();
        let base = p.floor().as_ivec2();
        // Shared corners make adjacent faces meet at exactly the same height.
        // Random face offsets with a narrow blend produced engraved double rails.
        for y in -1..=1 {
            for x in -1..=1 {
                let cell = base + IVec2::new(x, y);
                let vertices = [IVec2::ZERO, IVec2::X, IVec2::ONE, IVec2::Y].map(|offset| {
                    let corner = cell + offset;
                    let random = |s| hash(params, corner, cells, salt ^ s);
                    let position = corner.as_vec2()
                        + (Vec2::new(random(0x1247), random(0x9821)) - Vec2::splat(0.5))
                            * self.jitter
                            * 0.5;
                    (position, (random(0x8517) - 0.5) * self.depth)
                });
                let triangles = if hash(params, cell, cells, salt ^ 0x1735) < 0.5 {
                    [[0, 1, 2], [0, 2, 3]]
                } else {
                    [[0, 1, 3], [1, 2, 3]]
                };
                for triangle in triangles {
                    let [a, b, c] = triangle.map(|i| vertices[i]);
                    let ab = b.0 - a.0;
                    let ac = c.0 - a.0;
                    let ap = p - a.0;
                    let determinant = ab.perp_dot(ac);
                    let v = ap.perp_dot(ac) / determinant;
                    let w = ab.perp_dot(ap) / determinant;
                    let u = 1.0 - v - w;
                    if u >= -f32::EPSILON && v >= -f32::EPSILON && w >= -f32::EPSILON {
                        let weights = [u, v, w];
                        let softened = weights.map(|t| t * t * (3.0 - 2.0 * t));
                        let sum = softened.iter().sum::<f32>();
                        let heights = [a.1, b.1, c.1];
                        return (0..3)
                            .map(|i| {
                                heights[i]
                                    * (weights[i] * (1.0 - self.roundness)
                                        + softened[i] / sum * self.roundness)
                            })
                            .sum();
                    }
                }
            }
        }
        unreachable!("bounded jitter preserves a covering triangulation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adjoining_planes_have_no_height_ownership_jumps() {
        let params = TextureParameters::default();
        let planes = Parameters::default();
        let epsilon = 1.0 / 65536.0;
        for y in 0..128 {
            for x in 0..128 {
                let p = Vec2::new(x as f32 / 128.0, y as f32 / 128.0);
                let h = planes.sample(&params, p, 17);
                for axis in [Vec2::X, Vec2::Y] {
                    assert!((h - planes.sample(&params, p + axis * epsilon, 17)).abs() < 0.001);
                }
            }
        }
    }
    #[test]
    fn plane_fields_tile_and_retain_a_middle_scale() {
        let params = TextureParameters::default();
        let planes = Parameters::default();
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        for y in 0..32 {
            for x in 0..32 {
                let p = Vec2::new(x as f32 / 32.0, y as f32 / 32.0);
                let a = planes.sample(&params, p, 17);
                assert!((a - planes.sample(&params, p + Vec2::ONE, 17)).abs() < 0.00001);
                lo = lo.min(a);
                hi = hi.max(a);
            }
        }
        assert!(hi - lo > 0.1 && hi - lo < 0.5);
    }
}
