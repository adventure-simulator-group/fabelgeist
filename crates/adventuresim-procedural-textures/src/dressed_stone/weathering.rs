//! Metric edge fractures, sparse cavities, and granular lime mortar.

use super::{hash_unit, smoothstep};

const DAMAGED_EDGE_PROBABILITY: f32 = 0.32;
const CHIP_HALF_WIDTH_METRES: [f32; 2] = [0.012, 0.045];
const CHIP_DEPTH_METRES: [f32; 2] = [0.004, 0.016];
const BEVEL_WIDTH_METRES: [f32; 2] = [0.004, 0.009];
const PORE_CELL_METRES: f32 = 0.045;
const PORE_PROBABILITY: f32 = 0.18;
const PORE_RADIUS_METRES: [f32; 2] = [0.004, 0.010];
const MINERAL_PATCH_METRES: f32 = 0.085;
const STONE_GRAIN_METRES: f32 = 0.012;
const MORTAR_AGGREGATE_METRES: f32 = 0.009;
const MORTAR_TROWEL_METRES: f32 = 0.055;

#[derive(Clone, Copy, Debug)]
pub(super) struct EdgeProfile {
    /// Signed distance in metres: negative inside the chipped stone boundary.
    pub(super) distance: f32,
    pub(super) bevel: f32,
    pub(super) exposed_chip: f32,
}

fn between(range: [f32; 2], random: f32) -> f32 {
    range[0] + (range[1] - range[0]) * random
}

pub(super) fn edge_profile(
    local: [f32; 2],
    half_size: [f32; 2],
    distances: [f32; 4],
    id: u64,
) -> EdgeProfile {
    let mut distance = f32::NEG_INFINITY;
    let mut exposed_chip = 0.0_f32;
    for (edge, original_distance) in distances.into_iter().enumerate() {
        let edge_id = id ^ (edge as u64 + 1).wrapping_mul(0x9e37_79b9);
        let mut cut = 0.0_f32;
        if hash_unit(edge_id ^ 0x4ad1) < DAMAGED_EDGE_PROBABILITY {
            let along_axis = usize::from(edge < 2);
            let along = local[along_axis] * half_size[along_axis];
            let center = (hash_unit(edge_id ^ 0xa8e3) - 0.5) * half_size[along_axis] * 2.0;
            let width = between(CHIP_HALF_WIDTH_METRES, hash_unit(edge_id ^ 0xd457));
            let envelope = (1.0 - ((along - center) / width).abs()).max(0.0);
            let fracture = 0.75 + 0.25 * noise(along, 0.0, width * 0.35, edge_id);
            cut = between(CHIP_DEPTH_METRES, hash_unit(edge_id ^ 0xf1a7)) * envelope * fracture;
            exposed_chip = exposed_chip.max(
                smoothstep(0.0, CHIP_DEPTH_METRES[0], cut)
                    * (1.0 - smoothstep(0.0, BEVEL_WIDTH_METRES[1], -(original_distance + cut))),
            );
        }
        distance = distance.max(original_distance + cut);
    }
    let bevel_width = between(BEVEL_WIDTH_METRES, hash_unit(id ^ 0x319b));
    EdgeProfile {
        distance,
        bevel: 1.0 - smoothstep(0.0, bevel_width, -distance),
        exposed_chip,
    }
}

fn cell_id(x: i32, y: i32, salt: u64) -> u64 {
    salt ^ (x as u32 as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (y as u32 as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
}

fn noise(x: f32, y: f32, cell_metres: f32, salt: u64) -> f32 {
    let x = x / cell_metres;
    let y = y / cell_metres;
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let tx = smoothstep(0.0, 1.0, x - x.floor());
    let ty = smoothstep(0.0, 1.0, y - y.floor());
    let bottom =
        hash_unit(cell_id(ix, iy, salt)) * (1.0 - tx) + hash_unit(cell_id(ix + 1, iy, salt)) * tx;
    let top = hash_unit(cell_id(ix, iy + 1, salt)) * (1.0 - tx)
        + hash_unit(cell_id(ix + 1, iy + 1, salt)) * tx;
    (bottom * (1.0 - ty) + top * ty) * 2.0 - 1.0
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FaceDetail {
    pub(super) pore: f32,
    pub(super) mineral: f32,
    pub(super) grain: f32,
}

pub(super) fn face_detail(x: f32, y: f32, id: u64) -> FaceDetail {
    let cell_x = (x / PORE_CELL_METRES).floor() as i32;
    let cell_y = (y / PORE_CELL_METRES).floor() as i32;
    let mut pore = 0.0_f32;
    for iy in (cell_y - 1)..=(cell_y + 1) {
        for ix in (cell_x - 1)..=(cell_x + 1) {
            let pore_id = cell_id(ix, iy, id ^ 0x8c29);
            if hash_unit(pore_id) >= PORE_PROBABILITY {
                continue;
            }
            let dx = x - (ix as f32 + hash_unit(pore_id ^ 0x173d)) * PORE_CELL_METRES;
            let dy = y - (iy as f32 + hash_unit(pore_id ^ 0x913b)) * PORE_CELL_METRES;
            let radius = between(PORE_RADIUS_METRES, hash_unit(pore_id ^ 0xa741));
            let aspect = 0.65 + hash_unit(pore_id ^ 0xb53d) * 0.70;
            let radial_distance = ((dx * aspect).powi(2) + (dy / aspect).powi(2)).sqrt();
            pore = pore.max(1.0 - smoothstep(radius * 0.2, radius, radial_distance));
        }
    }
    FaceDetail {
        pore,
        mineral: noise(x, y, MINERAL_PATCH_METRES, id ^ 0x6ca1),
        grain: noise(x, y, STONE_GRAIN_METRES, id ^ 0x738b),
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct MortarDetail {
    pub(super) height: f32,
    pub(super) tone: f32,
    pub(super) roughness: f32,
}

pub(super) fn mortar_detail(x: f32, y: f32) -> MortarDetail {
    // World-aligned grains cross block ownership boundaries without a new random seed.
    // Trigonometric wrapping embeds the repeating tile in the noise domain continuously.
    let tau = std::f32::consts::TAU;
    let tile = super::DRESSED_STONE_TILE_METRES;
    let (sx, cx) = (x / tile * tau).sin_cos();
    let (sy, cy) = (y / tile * tau).sin_cos();
    let aggregate = (noise(sx, sy, MORTAR_AGGREGATE_METRES / tile * tau, 0x483d)
        + noise(cx, cy, MORTAR_AGGREGATE_METRES / tile * tau, 0x72f1))
        * 0.5;
    let trowel = noise(sx + cy, sy + cx, MORTAR_TROWEL_METRES / tile * tau, 0x19b7);
    MortarDetail {
        height: 0.18 + aggregate * 0.032 + trowel * 0.014,
        tone: aggregate * 12.0 + trowel * 4.0,
        roughness: 236.0 + aggregate * 13.0 - trowel * 4.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractures_remove_boundary_area_but_leave_some_edges_intact() {
        let mut chipped = 0;
        let mut intact = 0;
        for id in 0..128 {
            let mut lost_boundary = false;
            for step in 0..128 {
                let y = step as f32 / 64.0 - 1.0;
                let profile = edge_profile([1.0, y], [0.3, 0.16], [-0.6, -0.002, -0.16, -0.16], id);
                lost_boundary |= profile.distance > 0.0;
            }
            chipped += usize::from(lost_boundary);
            intact += usize::from(!lost_boundary);
        }
        assert!(
            chipped > 10 && intact > 40,
            "chipped={chipped}, intact={intact}"
        );
    }

    #[test]
    fn cavities_are_sparse_and_have_resolvable_metric_width() {
        let samples = (0..200)
            .flat_map(|y| {
                (0..200).map(move |x| face_detail(x as f32 * 0.003, y as f32 * 0.003, 37).pore)
            })
            .collect::<Vec<_>>();
        let coverage =
            samples.iter().filter(|pore| **pore > 0.2).count() as f32 / samples.len() as f32;
        assert!(
            (0.002..0.04).contains(&coverage),
            "cavity coverage={coverage}"
        );
        assert!(
            samples
                .windows(2)
                .any(|pair| pair[0] > 0.4 && pair[1] > 0.4)
        );
    }

    #[test]
    fn mortar_retains_aggregate_variation_and_wraps_continuously() {
        let mut low = f32::INFINITY;
        let mut high = f32::NEG_INFINITY;
        for index in 0..256 {
            let y = index as f32 * super::super::DRESSED_STONE_TILE_METRES / 256.0;
            let a = mortar_detail(0.0, y);
            let b = mortar_detail(super::super::DRESSED_STONE_TILE_METRES, y);
            assert!((a.height - b.height).abs() < 0.0001);
            low = low.min(a.roughness);
            high = high.max(a.roughness);
        }
        assert!(high - low > 8.0);
    }
}
