//! Smooth attachment weights keep both walls of a thin strap moving together.
use super::*;
use std::collections::BTreeMap;

const ATTACHMENT_BLEND_M: f32 = 0.025;

pub(super) fn attach_suspenders(plate: &GeneratedArmor, hardware: &mut GeneratedArmor) {
    let mut front = BTreeMap::<[i64; 2], f32>::new();
    let key = |p: &[f32; 3]| [p[0], p[1]].map(|v| (v * 1e6).round() as i64);
    for p in &hardware.positions {
        front
            .entry(key(p))
            .and_modify(|z| *z = z.max(p[2]))
            .or_insert(p[2]);
    }
    let mut sample = hardware.positions.clone();
    for p in &mut sample {
        p[2] = front[&key(p)];
    }
    let actual = std::mem::replace(&mut hardware.positions, sample);
    attach(plate, hardware);
    hardware.positions = actual;
}

pub(super) fn attach(plate: &GeneratedArmor, hardware: &mut GeneratedArmor) {
    for (vertex, point) in hardware.positions.iter().enumerate() {
        let distances = plate
            .positions
            .iter()
            .map(|p| {
                (0..3)
                    .map(|axis| (p[axis] - point[axis]).powi(2))
                    .sum::<f32>()
            })
            .collect::<Vec<_>>();
        let minimum = distances.iter().copied().fold(f32::INFINITY, f32::min);
        let mut influences = BTreeMap::<u32, f32>::new();
        for (index, distance) in distances.into_iter().enumerate() {
            let proximity = (-(distance - minimum) / ATTACHMENT_BLEND_M.powi(2)).exp();
            if proximity < 0.0001 {
                continue;
            }
            for (&joint, &weight) in plate.joint_indices[index]
                .iter()
                .zip(&plate.joint_weights[index])
            {
                if weight > 0.0 {
                    *influences.entry(joint).or_default() += weight * proximity;
                }
            }
        }
        let mut influences = influences.into_iter().collect::<Vec<_>>();
        influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        // GLB skinning retains four influences. Normalize that exact set here,
        // before residual morph correction, rather than losing weight at export.
        influences.truncate(4);
        let sum: f32 = influences.iter().map(|(_, weight)| weight).sum();
        hardware.joint_indices[vertex] = [0; 8];
        hardware.joint_weights[vertex] = [0.0; 8];
        for (slot, (joint, weight)) in influences.into_iter().enumerate() {
            hardware.joint_indices[vertex][slot] = joint;
            hardware.joint_weights[vertex][slot] = weight / sum;
        }
    }
}

/// Suspenders follow their attachment plates' shape targets. Re-solving a
/// convex drape independently at each endpoint would not commute with a blend
/// of those targets and can pull a hanger inside the fauld between endpoints.
pub(super) fn bind_suspenders(plate: &GeneratedArmor, hardware: &mut GeneratedArmor) -> Result<()> {
    // Opposite walls share one material attachment. Sampling them independently
    // can invert the leather gauge in a blend even when each endpoint is valid.
    let mut front = BTreeMap::<[i64; 2], f32>::new();
    let key = |p: &[f32; 3]| [p[0], p[1]].map(|v| (v * 1e6).round() as i64);
    for p in &hardware.positions {
        front
            .entry(key(p))
            .and_modify(|z| *z = z.max(p[2]))
            .or_insert(p[2]);
    }
    let bindings = hardware
        .positions
        .iter()
        .map(|point| {
            let point = [point[0], point[1], front[&key(point)]];
            let distances = plate
                .positions
                .iter()
                .enumerate()
                .map(|(index, p)| {
                    let distance = (0..3)
                        .map(|axis| (p[axis] - point[axis]).powi(2))
                        .sum::<f32>();
                    (index, distance)
                })
                .collect::<Vec<_>>();
            let minimum = distances
                .iter()
                .map(|(_, d)| *d)
                .fold(f32::INFINITY, f32::min);
            let mut weights = distances
                .into_iter()
                .map(|(i, d)| (i, (-(d - minimum) / ATTACHMENT_BLEND_M.powi(2)).exp()))
                .filter(|(_, w)| *w >= 0.0001)
                .collect::<Vec<_>>();
            let sum: f32 = weights.iter().map(|(_, w)| w).sum();
            for (_, w) in &mut weights {
                *w /= sum;
            }
            weights
        })
        .collect::<Vec<_>>();
    for (target, source) in hardware.morphs.iter_mut().zip(&plate.morphs) {
        for (vertex, binding) in bindings.iter().enumerate() {
            let delta = std::array::from_fn(|axis| {
                binding
                    .iter()
                    .map(|(i, w)| {
                        w * (source.direct_positions[*i][axis] - plate.positions[*i][axis])
                    })
                    .sum::<f32>()
            });
            target.position_deltas[vertex] = delta;
            target.direct_positions[vertex] =
                std::array::from_fn(|axis| hardware.positions[vertex][axis] + delta[axis]);
        }
        let mut mesh = adventuresim_armor_model::PartMesh::new();
        mesh.positions = target.direct_positions.clone();
        mesh.indices = hardware.indices.clone();
        target.normal_deltas = mesh
            .normals()?
            .iter()
            .zip(&hardware.normals)
            .map(|(a, b)| std::array::from_fn(|axis| a[axis] - b[axis]))
            .collect();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> GeneratedArmor {
        GeneratedArmor {
            design_hash: [0; 32],
            surface_domain: String::new(),
            plate_edges: Vec::new(),
            positions: vec![[-0.01, 0.0, 0.0], [0.01, 0.0, 0.0], [0.0, 0.02, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            texcoords: vec![[0.0; 2]; 3],
            indices: vec![0, 1, 2],
            components: Vec::new(),
            morphs: Vec::new(),
            joint_indices: vec![[0; 8], [1; 8], [1; 8]],
            joint_weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 3],
        }
    }

    #[test]
    fn a_thin_wall_crossing_a_source_skin_boundary_has_continuous_normalized_weights() {
        let plate = triangle();
        let mut strap = triangle();
        strap.positions = vec![
            [-0.0001, 0.0, 0.003],
            [0.0001, 0.0, 0.003],
            [0.0, 0.002, 0.003],
        ];
        attach(&plate, &mut strap);
        for weights in &strap.joint_weights {
            assert!((weights.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        }
        let influence = |i: usize, bone| {
            strap.joint_indices[i]
                .iter()
                .zip(&strap.joint_weights[i])
                .filter(|(j, _)| **j == bone)
                .map(|(_, w)| w)
                .sum::<f32>()
        };
        assert!((influence(0, 0) - influence(1, 0)).abs() < 0.01);
        assert!(influence(0, 0) > 0.1 && influence(0, 0) < 0.9);
    }

    #[test]
    fn hanger_binding_preserves_plate_translation_and_surface_standoff() {
        let mut plate = triangle();
        let mut hanger = triangle();
        for p in &mut hanger.positions {
            p[2] += 0.004;
        }
        let movement = [0.01, 0.02, 0.03];
        plate.morphs.push(adventuresim_armor_model::ArmorMorph {
            name: "translation".into(),
            position_deltas: vec![movement; 3],
            normal_deltas: vec![[0.0; 3]; 3],
            direct_positions: plate
                .positions
                .iter()
                .map(|p| std::array::from_fn(|i| p[i] + movement[i]))
                .collect(),
        });
        hanger.morphs = plate.morphs.clone();
        bind_suspenders(&plate, &mut hanger).unwrap();
        for (base, target) in hanger
            .positions
            .iter()
            .zip(&hanger.morphs[0].direct_positions)
        {
            for axis in 0..3 {
                assert!((target[axis] - base[axis] - movement[axis]).abs() < 1e-6);
            }
        }
    }
}
