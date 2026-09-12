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
        attach_suspenders(&plate, &mut hanger).unwrap();
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
    #[test]
    fn suspension_uses_its_physical_plate_instead_of_blending_a_nearby_layer() {
        let mut plate = triangle();
        plate.joint_indices.fill([0; 8]);
        let back = plate
            .positions
            .iter()
            .map(|p| [p[0], p[1], -0.008])
            .collect::<Vec<_>>();
        plate.positions.extend(back);
        plate.indices.extend([3, 4, 5]);
        plate.joint_indices.extend([[1; 8]; 3]);
        plate
            .joint_weights
            .extend([[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 3]);
        let motion = [0.0, 0.02, 0.01];
        plate.morphs.push(adventuresim_armor_model::ArmorMorph {
            name: "independent_plate_motion".into(),
            position_deltas: Vec::new(),
            normal_deltas: Vec::new(),
            direct_positions: plate
                .positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    std::array::from_fn(|axis| {
                        p[axis] + if i < 3 { motion[axis] } else { -motion[axis] }
                    })
                })
                .collect(),
        });
        let mut hanger = triangle();
        hanger.positions = vec![
            [-0.003, 0.003, 0.003],
            [-0.003, 0.003, 0.005],
            [0.003, 0.004, 0.005],
        ];
        hanger.morphs.push(adventuresim_armor_model::ArmorMorph {
            name: "independent_plate_motion".into(),
            position_deltas: vec![[0.0; 3]; 3],
            normal_deltas: vec![[0.0; 3]; 3],
            direct_positions: hanger.positions.clone(),
        });
        let reference = hanger.positions.clone();
        attach_suspenders(&plate, &mut hanger).unwrap();
        assert_eq!(
            hanger.positions, reference,
            "binding must preserve the reference silhouette"
        );
        for vertex in 0..3 {
            assert_eq!(hanger.joint_indices[vertex][0], 0);
            assert!((hanger.joint_weights[vertex][0] - 1.0).abs() < 1e-6);
            for (axis, expected) in motion.into_iter().enumerate() {
                assert!((hanger.morphs[0].position_deltas[vertex][axis] - expected).abs() < 1e-6);
            }
        }
        assert_eq!(
            hanger.morphs[0].position_deltas[0], hanger.morphs[0].position_deltas[1],
            "opposite leather walls must retain one attachment"
        );
    }
}
