//! Smooth attachment weights keep both walls of a thin strap moving together.
use super::*;
use std::collections::BTreeMap;

const ATTACHMENT_BLEND_M: f32 = 0.025;

#[path = "suspender_skin.rs"]
mod suspenders;
pub(super) use suspenders::attach_suspenders;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> GeneratedArmor {
        GeneratedArmor {
            construction_faces: Vec::new(),
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
    fn posed_hangers_blend_continuously_and_keep_mounted_metal_rigid() {
        use adventuresim_armor_model::{ArmorComponent, ArmorComponentRole};
        let component = |role, vertices, indices| ArmorComponent {
            role,
            vertices,
            indices,
            hinge: None,
            material: None,
        };
        for span in [0.06, 0.12, 0.24] {
            for width in [0.012, 0.024] {
                let mut plate = triangle();
                plate.positions = vec![
                    [-0.1, span, 0.0],
                    [0.1, span, 0.0],
                    [0.0, span + 0.1, 0.0],
                    [-0.1, 0.0, 0.0],
                    [0.1, 0.0, 0.0],
                    [0.0, -0.1, 0.0],
                ];
                plate.indices = vec![0, 1, 2, 3, 4, 5];
                plate.joint_indices = vec![[0; 8]; 3];
                plate.joint_indices.extend([[1; 8]; 3]);
                plate.joint_weights = vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 6];
                plate.components = vec![
                    component(ArmorComponentRole::Fauld, 0..3, 0..3),
                    component(ArmorComponentRole::Tassets, 3..6, 3..6),
                ];
                let mut hanger = triangle();
                hanger.positions = (0..=64)
                    .flat_map(|i| [0.003, 0.005].map(|z| [0.0, span * i as f32 / 64.0, z]))
                    .collect();
                let metal = hanger.positions.len();
                hanger.positions.extend([
                    [-width, 0.0, 0.007],
                    [width, 0.0, 0.007],
                    [0.0, width, 0.007],
                ]);
                hanger.indices = vec![metal as u32, metal as u32 + 1, metal as u32 + 2];
                hanger.components = vec![component(
                    ArmorComponentRole::Buckles,
                    metal..metal + 3,
                    0..3,
                )];
                hanger.joint_indices = vec![[0; 8]; hanger.positions.len()];
                hanger.joint_weights = vec![[0.0; 8]; hanger.positions.len()];
                attach_suspenders(&plate, &mut hanger, std::slice::from_ref(&(0..metal))).unwrap();
                for angle in [-0.5_f32, 0.17, 0.5] {
                    let rotation = bevy::math::Quat::from_rotation_z(angle);
                    let posed = |i: usize| {
                        let p = Vec3::from_array(hanger.positions[i]);
                        hanger.joint_indices[i]
                            .iter()
                            .zip(hanger.joint_weights[i])
                            .map(|(&joint, weight)| {
                                if joint == 0 {
                                    p * weight
                                } else {
                                    (rotation * p + Vec3::new(0.03, 0.0, 0.0)) * weight
                                }
                            })
                            .sum::<Vec3>()
                    };
                    for row in 1..=64 {
                        assert!(
                            (posed(row * 2).x - posed((row - 1) * 2).x).abs() < 0.006,
                            "pose creates a lateral step at a plate ownership boundary"
                        );
                        assert_eq!(
                            hanger.joint_indices[row * 2],
                            hanger.joint_indices[row * 2 + 1]
                        );
                        assert_eq!(
                            hanger.joint_weights[row * 2],
                            hanger.joint_weights[row * 2 + 1]
                        );
                    }
                    for i in metal..metal + 3 {
                        for j in metal..metal + 3 {
                            let original = Vec3::from_array(hanger.positions[i])
                                .distance(Vec3::from_array(hanger.positions[j]));
                            assert!((posed(i).distance(posed(j)) - original).abs() < 1e-6);
                        }
                    }
                }
            }
        }
    }
}
