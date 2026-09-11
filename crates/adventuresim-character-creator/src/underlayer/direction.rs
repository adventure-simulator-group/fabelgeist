//! Offset directions must point outside every incident body face.
use super::*;
use crate::surface_cut::dot;

const OUTWARD_MARGIN: f32 = 0.05;
const PROJECTION_PASSES: usize = 32;

pub(super) type Constraint = ([u32; 3], [f32; 3]);

pub(super) fn directions(body: &Wearer<'_>) -> Vec<[f32; 3]> {
    constrained(body, &[])
}

pub(super) fn constrained(body: &Wearer<'_>, extra: &[Constraint]) -> Vec<[f32; 3]> {
    let mut physical = std::collections::BTreeMap::new();
    let ids = body
        .positions
        .iter()
        .map(|p| {
            let next = physical.len();
            *physical
                .entry(p.map(|v| (v * 1_000_000.0).round() as i64))
                .or_insert(next)
        })
        .collect::<Vec<_>>();
    let mut directions = vec![[0.; 3]; physical.len()];
    for (normal, &id) in body.normals.iter().zip(&ids) {
        directions[id] = *normal;
    }
    let constraints = constraints(body)
        .into_iter()
        .chain(extra.iter().copied())
        .map(|(face, normal)| (face.map(|v| ids[v as usize]), normal))
        .collect::<Vec<_>>();
    // Shading normals may point through an adjacent face at a tight fold.
    // Project onto the intersection of its outward half-spaces, keeping UV
    // duplicates together. The margin leaves room for numerical roundoff.
    for _ in 0..PROJECTION_PASSES {
        let mut changed = false;
        for (face, normal) in &constraints {
            for &id in face {
                let deficit = OUTWARD_MARGIN - dot(directions[id], *normal);
                if deficit > 0. {
                    let direction =
                        std::array::from_fn(|i| directions[id][i] + deficit * normal[i]);
                    let length = dot(direction, direction).sqrt();
                    directions[id] = direction.map(|v| v / length);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    ids.into_iter().map(|id| directions[id]).collect()
}

pub(super) fn constraints(body: &Wearer<'_>) -> Vec<Constraint> {
    body.faces
        .iter()
        .filter_map(|face| {
            let [a, b, c] = face.map(|i| body.positions[i as usize]);
            let u: [f32; 3] = std::array::from_fn(|i| b[i] - a[i]);
            let v: [f32; 3] = std::array::from_fn(|i| c[i] - a[i]);
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let length = dot(n, n).sqrt();
            (length > 0.).then(|| (*face, n.map(|v| v / length)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shaped_fold_must_not_turn_a_frozen_offset_inward() {
        let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        let faces = [[0, 1, 2]];
        let normals = [[0.8, 0., 0.6]; 3];
        let body = Wearer {
            positions: &positions,
            faces: &faces,
            normals: &normals,
            joints: &[],
            joint_indices: &[],
            joint_weights: &[],
            joint_names: &[],
        };
        let shaped_normal = [-0.8, 0., 0.6];
        assert!(dot(directions(&body)[0], shaped_normal) < 0.);
        for direction in constrained(&body, &[([0, 1, 2], shaped_normal)]) {
            assert!(dot(direction, shaped_normal) > 0.0499);
            assert!(direction[2] > 0.0499);
        }
    }
}
