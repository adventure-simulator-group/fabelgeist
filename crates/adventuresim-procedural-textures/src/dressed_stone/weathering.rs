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
const BEVEL_WIDTH_VARIATION: f32 = 0.55;
const FRACTURE_SHOULDER_RATIO: f32 = 0.65;
const FRACTURE_FACETS: [(f32, f32); 3] = [(1.0, 0.0), (0.52, -0.73), (0.31, 0.87)];
const EDGE_BREAKUP_METRES: f32 = 0.075;
const SPALL_WIDTH_METRES: f32 = 0.042;
const CORNER_FRACTURE_PROBABILITY: f32 = 0.22;
const CORNER_FRACTURE_METRES: [f32; 2] = [0.010, 0.036];

mod mortar;
pub(super) use mortar::mortar_detail;

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
    let mut bevel = 0.0_f32;
    for (edge, original_distance) in distances.into_iter().enumerate() {
        let edge_id = id ^ (edge as u64 + 1).wrapping_mul(0x9e37_79b9);
        let along_axis = usize::from(edge < 2);
        let along = local[along_axis] * half_size[along_axis];
        let (cut, spall) = edge_fracture(along, half_size[along_axis], original_distance, edge_id);
        let altered_distance = original_distance + cut;
        let width = between(BEVEL_WIDTH_METRES, hash_unit(edge_id ^ 0x319b))
            * (1.0 + BEVEL_WIDTH_VARIATION * noise(along, 0.0, EDGE_BREAKUP_METRES, edge_id));
        bevel = bevel.max(1.0 - smoothstep(0.0, width, -altered_distance));
        exposed_chip = exposed_chip.max(spall);
        distance = distance.max(altered_distance);
    }
    // Occasional diagonal fracture planes remove corners; the remaining ashlar
    // stays planar. This is a second shape family, separate from edge notches.
    for (corner, (horizontal, vertical)) in [(0, 2), (0, 3), (1, 2), (1, 3)].into_iter().enumerate()
    {
        let corner_id = id ^ (corner as u64 + 1).wrapping_mul(0x73d1);
        if hash_unit(corner_id) >= CORNER_FRACTURE_PROBABILITY {
            continue;
        }
        let cut = between(CORNER_FRACTURE_METRES, hash_unit(corner_id ^ 0x471b));
        let diagonal =
            (distances[horizontal] + distances[vertical] + cut) * std::f32::consts::FRAC_1_SQRT_2;
        distance = distance.max(diagonal);
        bevel = bevel.max(1.0 - smoothstep(0.0, BEVEL_WIDTH_METRES[0], -diagonal));
        exposed_chip = exposed_chip.max(1.0 - smoothstep(0.0, SPALL_WIDTH_METRES, -diagonal));
    }
    EdgeProfile {
        distance,
        bevel,
        exposed_chip,
    }
}

fn edge_fracture(along: f32, half_length: f32, distance: f32, id: u64) -> (f32, f32) {
    if hash_unit(id ^ 0x4ad1) >= DAMAGED_EDGE_PROBABILITY {
        return (0.0, 0.0);
    }
    let center = (hash_unit(id ^ 0xa8e3) - 0.5) * half_length * 2.0;
    let width = between(CHIP_HALF_WIDTH_METRES, hash_unit(id ^ 0xd457));
    let depth = between(CHIP_DEPTH_METRES, hash_unit(id ^ 0xf1a7));
    let mut cut = 0.0_f32;
    let mut spall = 0.0_f32;
    // A main break and two overlapping smaller facets share a damage region.
    // Flat-sided profiles preserve fracture character instead of rounded dents.
    for (scale, shift) in FRACTURE_FACETS {
        let offset = (along - center - shift * width) / (width * scale);
        let envelope = (1.0 - offset.abs()).max(0.0);
        cut = cut.max(depth * scale * envelope);
        let shoulder = (1.0 - (offset * FRACTURE_SHOULDER_RATIO).abs()).max(0.0);
        let inward = (1.0 + distance / (SPALL_WIDTH_METRES * scale)).clamp(0.0, 1.0);
        spall = spall.max(shoulder * inward * scale);
    }
    (cut, spall)
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
            let grouping = (noise(
                ix as f32 * PORE_CELL_METRES,
                iy as f32 * PORE_CELL_METRES,
                MINERAL_PATCH_METRES * 2.0,
                id ^ 0x331a,
            ) + 1.0)
                .clamp(0.0, 1.0);
            if hash_unit(pore_id) >= PORE_PROBABILITY * grouping {
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
            let a = mortar_detail(0.0, y, 0.008);
            let b = mortar_detail(super::super::DRESSED_STONE_TILE_METRES, y, 0.008);
            assert!((a.height - b.height).abs() < 0.0001);
            low = low.min(a.height);
            high = high.max(a.height);
        }
        assert!(high - low > 0.03);
    }
}
