//! Limit the complete layer stack against nearby opposing body surfaces.
use crate::surface_cut::dot;
use std::collections::{BTreeMap, BTreeSet};

const RAY_ROUNDOFF_M: f32 = 1e-6;
const GAP_ALLOCATION: f32 = 0.25;

pub(super) fn constrain(
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
    directions: &[[f32; 3]],
    room: &mut [f32],
    envelope: f32,
) {
    let reach = envelope * 2.;
    let cell = |p: [f32; 3]| p.map(|v| (v / reach).floor() as i32);
    let mut grid = BTreeMap::<[i32; 3], Vec<usize>>::new();
    for (i, face) in faces.iter().enumerate() {
        let p = face.map(|v| positions[v as usize]);
        let low = cell(std::array::from_fn(|k| {
            p.iter().map(|p| p[k]).fold(f32::INFINITY, f32::min)
        }));
        let high = cell(std::array::from_fn(|k| {
            p.iter().map(|p| p[k]).fold(f32::NEG_INFINITY, f32::max)
        }));
        for key in cells(low, high) {
            grid.entry(key).or_default().push(i);
        }
    }
    for (i, (&origin, &direction)) in positions.iter().zip(directions).enumerate() {
        let end: [f32; 3] = std::array::from_fn(|k| origin[k] + direction[k] * reach);
        let low = cell(std::array::from_fn(|k| origin[k].min(end[k])));
        let high = cell(std::array::from_fn(|k| origin[k].max(end[k])));
        let candidates = cells(low, high)
            .into_iter()
            .filter_map(|key| grid.get(&key))
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>();
        for index in candidates {
            if faces[index].contains(&(i as u32)) {
                continue;
            }
            let triangle = faces[index].map(|v| positions[v as usize]);
            if let Some(distance) = facing_hit(origin, direction, triangle)
                && distance > RAY_ROUNDOFF_M
                && distance < reach
            {
                room[i] = room[i].min(distance * GAP_ALLOCATION);
            }
        }
    }
}

fn cells(low: [i32; 3], high: [i32; 3]) -> Vec<[i32; 3]> {
    let mut result = Vec::new();
    for x in low[0]..=high[0] {
        for y in low[1]..=high[1] {
            for z in low[2]..=high[2] {
                result.push([x, y, z]);
            }
        }
    }
    result
}

fn facing_hit(origin: [f32; 3], direction: [f32; 3], triangle: [[f32; 3]; 3]) -> Option<f32> {
    let sub = |a: [f32; 3], b: [f32; 3]| std::array::from_fn(|k| a[k] - b[k]);
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let e = sub(triangle[1], triangle[0]);
    let f = sub(triangle[2], triangle[0]);
    let p = cross(direction, f);
    let determinant = dot(e, p);
    if determinant <= 0. {
        return None;
    }
    let t = sub(origin, triangle[0]);
    let u = dot(t, p) / determinant;
    let q = cross(t, e);
    let v = dot(direction, q) / determinant;
    if u < 0. || v < 0. || u + v > 1. {
        return None;
    }
    Some(dot(f, q) / determinant)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parallel_surfaces_share_the_gap_even_without_local_curvature() {
        let positions = [
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [0., 0., 0.005],
            [0., 1., 0.005],
            [1., 0., 0.005],
        ];
        let faces = [[0, 1, 2], [3, 4, 5]];
        let directions = [
            [0., 0., 1.],
            [0., 0., 1.],
            [0., 0., 1.],
            [0., 0., -1.],
            [0., 0., -1.],
            [0., 0., -1.],
        ];
        let mut room = [0.018; 6];
        constrain(&positions, &faces, &directions, &mut room, 0.018);
        for value in room {
            assert!(value > 0. && value < 0.0025, "overlapping stacks: {value}");
        }
    }
}
