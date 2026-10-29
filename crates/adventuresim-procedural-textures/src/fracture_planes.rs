//! Periodic joined fracture planes with shared corner heights.
use crate::{TextureParameters, stamps::hash};
use bevy::math::{IVec2, Vec2};
use fabelgeist_determinism::Seed;
use fabelgeist_determinism::StreamId;

const RNG_TEXTURE_FRACTURE_PLANES_TRIANGULATION: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("texture.fracture-planes.triangulation");
const TRIANGLE_WEIGHT_ROUNDOFF_ULPS: f32 = 8.0;
crate::parameters::parameter_block! {
    pub struct Parameters {
        cells: [i32; 2] = [11, 9];
        jitter: f32 = 0.85;
        depth: f32 = 0.24;
        roundness: f32 = 0.10;
    }
}
impl Parameters {
    pub(crate) fn sample(&self, params: &TextureParameters, uv: Vec2, field_seed: Seed) -> f32 {
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
                    let random = |purpose: StreamId| {
                        hash(params, corner, cells, purpose.seed(field_seed, &[]))
                    };
                    let position = corner.as_vec2()
                        + (Vec2::new(
                            random(StreamId::new("texture.fracture-planes.corner-x")),
                            random(StreamId::new("texture.fracture-planes.corner-y")),
                        ) - Vec2::splat(0.5))
                            * self.jitter
                            * 0.5;
                    (
                        position,
                        (random(StreamId::new("texture.fracture-planes.corner-height")) - 0.5)
                            * self.depth,
                    )
                });
                let triangles = if hash(
                    params,
                    cell,
                    cells,
                    params.field_seed(
                        RNG_TEXTURE_FRACTURE_PLANES_TRIANGULATION,
                        &[field_seed.to_u64()],
                    ),
                ) < 0.5
                {
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
                    // Bound subtraction, cross products and division at the
                    // actual cell-coordinate scale and triangle condition.
                    // The unit term covers forming u = 1 - v - w.
                    let magnitude = [p, a.0, b.0, c.0]
                        .map(|point| point.abs().max_element())
                        .into_iter()
                        .fold(0.0, f32::max);
                    let roundoff = TRIANGLE_WEIGHT_ROUNDOFF_ULPS
                        * f32::EPSILON
                        * (magnitude * (ab.length() + ac.length() + ap.length())
                            / determinant.abs()
                            + 1.0
                            + v.abs()
                            + w.abs());
                    if u >= -roundoff && v >= -roundoff && w >= -roundoff {
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
    fn shared_grid_edges_and_corners_remain_covered_at_each_cell_scale() {
        let params = TextureParameters::default();
        for cells in [[1, 1], [11, 9], [1024, 1024]] {
            let planes = Parameters {
                cells,
                jitter: 0.0,
                ..Default::default()
            };
            for coordinate in [
                0.0,
                0.5_f32.next_down(),
                0.5,
                0.5_f32.next_up(),
                1.0_f32.next_down(),
            ] {
                for point in [Vec2::splat(coordinate), Vec2::new(coordinate, 0.0)] {
                    assert!(
                        planes
                            .sample(&params, point, Seed::from_u64(17))
                            .is_finite()
                    );
                }
            }
        }
    }
    #[test]
    fn adjoining_planes_have_no_height_ownership_jumps() {
        let params = TextureParameters::default();
        let planes = Parameters::default();
        let epsilon = 1.0 / 65536.0;
        for y in 0..128 {
            for x in 0..128 {
                let p = Vec2::new(x as f32 / 128.0, y as f32 / 128.0);
                let h = planes.sample(&params, p, fabelgeist_determinism::Seed::from_u64(17));
                for axis in [Vec2::X, Vec2::Y] {
                    assert!(
                        (h - planes.sample(
                            &params,
                            p + axis * epsilon,
                            fabelgeist_determinism::Seed::from_u64(17)
                        ))
                        .abs()
                            < 0.001
                    );
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
                let a = planes.sample(&params, p, fabelgeist_determinism::Seed::from_u64(17));
                assert!(
                    (a - planes.sample(
                        &params,
                        p + Vec2::ONE,
                        fabelgeist_determinism::Seed::from_u64(17)
                    ))
                    .abs()
                        < 0.00001
                );
                lo = lo.min(a);
                hi = hi.max(a);
            }
        }
        assert!(hi - lo > 0.1 && hi - lo < 0.5);
    }
}
