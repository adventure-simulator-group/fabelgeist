//! Compress the layer stack where converging body normals leave little room.
use super::*;
use crate::surface_cut::dot;

const LAYER_STACK_ENVELOPE_M: f32 = 0.018;
const FOCAL_DISTANCE_FRACTION: f32 = 0.25;
const PRISM_MARGIN: f32 = 0.25;
const COMPRESSION_PASSES: usize = 32;

pub(super) fn compression(body: &Wearer<'_>) -> Vec<f32> {
    let directions = super::direction::directions(body);
    along(body, &directions)
}

pub(super) fn along(body: &Wearer<'_>, directions: &[[f32; 3]]) -> Vec<f32> {
    let mut room = vec![LAYER_STACK_ENVELOPE_M; body.positions.len()];
    super::gap::constrain(
        body.positions,
        body.faces,
        directions,
        &mut room,
        LAYER_STACK_ENVELOPE_M,
    );
    for &[a, b, c] in body.faces {
        for [a, b] in [[a, b], [b, c], [c, a]] {
            let (a, b) = (a as usize, b as usize);
            let edge = std::array::from_fn(|i| body.positions[b][i] - body.positions[a][i]);
            let normal_change = std::array::from_fn(|i| directions[b][i] - directions[a][i]);
            let convergence = -dot(edge, normal_change);
            if convergence > f32::EPSILON {
                let limit = dot(edge, edge) / convergence * FOCAL_DISTANCE_FRACTION;
                room[a] = room[a].min(limit);
                room[b] = room[b].min(limit);
            }
        }
    }
    // Edge convergence alone misses shear caused by unequal vertex offsets.
    // Keep each triangle's extrusion Jacobian positive throughout the stack.
    for _ in 0..COMPRESSION_PASSES {
        synchronize_seams(body.positions, &mut room);
        let mut changed = false;
        for face in body.faces {
            let points = face.map(|v| body.positions[v as usize]);
            let offsets = face.map(|v| directions[v as usize].map(|d| d * room[v as usize]));
            if !valid_prism(points, offsets) {
                for v in face {
                    room[*v as usize] *= 0.5;
                }
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    synchronize_seams(body.positions, &mut room);
    room.into_iter()
        .map(|distance| distance / LAYER_STACK_ENVELOPE_M)
        .collect()
}

fn synchronize_seams(positions: &[[f32; 3]], room: &mut [f32]) {
    // UV duplicates must share one physical offset before checking prisms.
    let mut physical = std::collections::BTreeMap::<[i64; 3], f32>::new();
    for (point, value) in positions.iter().zip(room.iter()) {
        let key = point.map(|v| (v * 1_000_000.0).round() as i64);
        physical
            .entry(key)
            .and_modify(|v| *v = v.min(*value))
            .or_insert(*value);
    }
    for (point, value) in positions.iter().zip(room.iter_mut()) {
        *value = physical[&point.map(|v| (v * 1_000_000.0).round() as i64)];
    }
}

fn valid_prism(points: [[f32; 3]; 3], offsets: [[f32; 3]; 3]) -> bool {
    let subtract = |a: [f32; 3], b: [f32; 3]| std::array::from_fn(|i| a[i] - b[i]);
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let e = subtract(points[1], points[0]);
    let f = subtract(points[2], points[0]);
    let u = subtract(offsets[1], offsets[0]);
    let v = subtract(offsets[2], offsets[0]);
    let linear = std::array::from_fn(|i| cross(e, v)[i] + cross(u, f)[i]);
    offsets.into_iter().all(|offset| {
        let a = dot(cross(u, v), offset);
        let b = dot(linear, offset);
        let c = dot(cross(e, f), offset);
        let at = |t: f32| a * t * t + b * t + c;
        let minimum = if a > 0.0 {
            at((-b / (2.0 * a)).clamp(0.0, 1.0)).min(at(1.0))
        } else {
            c.min(at(1.0))
        };
        c > 0.0 && minimum >= c * PRISM_MARGIN
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilted_extrusion_must_keep_both_sides_separate() {
        let points = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        assert!(valid_prism(points, [[0., 0., 0.1]; 3]));
        // Every offset starts outside the source face, but the second vertex
        // sweeps across the first and reverses the displaced face.
        let offsets = [[0., 0., 0.1], [-2., 0., 0.1], [0., 0., 0.1]];
        assert!(!valid_prism(points, offsets));
        assert!(valid_prism(points, offsets.map(|v| v.map(|x| x * 0.1))));
    }
}
