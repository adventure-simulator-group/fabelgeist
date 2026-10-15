//! Observable garment binding across the leg boundary and fitting orientation.
use adventuresim_character_creator::{
    garment::{DrapeInput, GarmentSelection, pattern::shapes, transfer_skin},
    skin_transfer::SkinWeights,
};
use fabelgeist_rig::RigJointName;

fn body() -> DrapeInput {
    DrapeInput {
        selection: GarmentSelection::from_shape(&shapes::DRESS),
        under_plate: None,
        settled: None,
        obstacles: vec![],
        positions: vec![
            [-0.1, 0., 0.],
            [-0.1, 1., 0.],
            [-0.1, 0., 1.],
            [0.1, 0., 0.],
            [0.1, 1., 0.],
            [0.1, 0., 1.],
        ],
        faces: vec![[0, 1, 2], [3, 4, 5]],
        names: vec![
            RigJointName::L_UPLEG,
            RigJointName::R_UPLEG,
            RigJointName::ROOT,
            RigJointName::L_LOWLEG,
            RigJointName::R_LOWLEG,
        ],
        joints: vec![
            [-0.1, 1., 0., 0., 0., 0., 1., 1.],
            [0.1, 1., 0., 0., 0., 0., 1., 1.],
            [0., 1., 0., 0., 0., 0., 1., 1.],
            [-0.1, 0.5, 0., 0., 0., 0., 1., 1.],
            [0.1, 0.5, 0., 0., 0., 0., 1., 1.],
        ],
        indices: vec![[0; 8], [0; 8], [0; 8], [1; 8], [1; 8], [1; 8]],
        weights: vec![[1., 0., 0., 0., 0., 0., 0., 0.]; 6],
    }
}

#[test]
fn turning_the_fitting_body_preserves_garment_binding() {
    let input = body();
    let points = [
        [-0.1, 0.5, 0.1],
        [-0.02, 0.5, 0.1],
        [0.02, 0.5, 0.1],
        [0.1, 0.5, 0.1],
    ];
    let reference = transfer_skin(&input, &points).unwrap();
    let turn = |p: [f32; 3]| [p[2], p[1], -p[0]];
    let mut turned = input.clone();
    turned.positions = input.positions.iter().copied().map(turn).collect();
    for joint in &mut turned.joints {
        let point = turn([joint[0], joint[1], joint[2]]);
        joint[..3].copy_from_slice(&point);
    }
    let actual = transfer_skin(&turned, &points.map(turn)).unwrap();
    assert_eq!(actual.indices, reference.indices);
    for (actual, expected) in actual
        .weights
        .iter()
        .flatten()
        .zip(reference.weights.iter().flatten())
    {
        assert!((actual - expected).abs() < 1e-6);
    }
}

#[test]
fn opposing_leg_rotations_do_not_split_the_garment_center() {
    let input = body();
    for height in [0.999, 0.95, 0.5] {
        let points = [[-0.00001, height, 0.1], [0.00001, height, 0.1]];
        let SkinWeights { indices, weights } = transfer_skin(&input, &points).unwrap();
        let pose = |vertex: usize| {
            let mut output = [0.; 3];
            for (&joint, &weight) in indices[vertex].iter().zip(&weights[vertex]) {
                let hip = &input.joints[joint as usize];
                let angle = match joint {
                    0 => 0.65f32,
                    1 => -0.65,
                    _ => 0.0,
                };
                let (sin, cos) = angle.sin_cos();
                let dy = points[vertex][1] - hip[1];
                let dz = points[vertex][2] - hip[2];
                let moved = [
                    points[vertex][0],
                    hip[1] + cos * dy - sin * dz,
                    hip[2] + sin * dy + cos * dz,
                ];
                for axis in 0..3 {
                    output[axis] += moved[axis] * weight;
                }
            }
            output
        };
        let a = pose(0);
        let b = pose(1);
        let separation = a
            .iter()
            .zip(b)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt();
        assert!(separation < 0.001, "center split by {separation} metres");
        for row in weights {
            assert!(row.iter().all(|w| w.is_finite() && *w >= 0.0));
            assert!((row.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        }
    }
}

#[test]
fn malformed_binding_inputs_fail_before_surface_queries() {
    let mut input = body();
    input.weights.pop();
    assert!(transfer_skin(&input, &[[0., 0.5, 0.1]]).is_err());
    let input = body();
    assert!(transfer_skin(&input, &[[f32::NAN, 0.5, 0.1]]).is_err());
    let mut input = body();
    input.indices[0][0] = u32::MAX;
    assert!(transfer_skin(&input, &[[0., 0.5, 0.1]]).is_err());
    let mut input = body();
    input.joints.pop();
    assert!(transfer_skin(&input, &[[0., 0.5, 0.1]]).is_err());
}

#[test]
fn both_legs_survive_the_four_influence_export_budget() {
    let mut input = body();
    input.names = (0..19)
        .map(|i| match i {
            0 => RigJointName::L_UPLEG,
            8 => RigJointName::R_UPLEG,
            16 => RigJointName::ROOT,
            17 => RigJointName::L_LOWLEG,
            18 => RigJointName::R_LOWLEG,
            _ => RigJointName::from(format!("helper_{i}")),
        })
        .collect();
    let left = input.joints[0];
    let right = input.joints[1];
    input.joints = (0..19)
        .map(|i| match i {
            16 => [0., 1., 0., 0., 0., 0., 1., 1.],
            17 => [-0.1, 0.5, 0., 0., 0., 0., 1., 1.],
            18 => [0.1, 0.5, 0., 0., 0., 0., 1., 1.],
            _ if i < 8 => left,
            _ => right,
        })
        .collect();
    input.indices = (0..6)
        .map(|v| std::array::from_fn(|i| i as u32 + if v < 3 { 0 } else { 8 }))
        .collect();
    input.weights = vec![[0.125; 8]; 6];
    let skin = transfer_skin(&input, &[[-0.00001, 0.95, 0.1], [0.00001, 0.95, 0.1]]).unwrap();
    for vertex in 0..2 {
        let active: std::collections::BTreeSet<_> = skin.indices[vertex]
            .iter()
            .zip(&skin.weights[vertex])
            .filter(|(_, w)| **w > 0.0)
            .map(|(i, _)| *i)
            .collect();
        assert_eq!(active, [0, 8, 16].into_iter().collect());
    }
    for joint in [0, 8, 16] {
        let weight = |v: usize| {
            skin.indices[v]
                .iter()
                .zip(skin.weights[v])
                .filter(|(i, _)| **i == joint)
                .map(|(_, w)| w)
                .sum::<f32>()
        };
        assert!((weight(0) - weight(1)).abs() < 0.001);
    }
}

#[test]
fn lower_skirt_ownership_is_independent_of_depth_and_surface_helpers() {
    let input = body();
    let points = [[0.01, 0.5, -0.5], [0.01, 0.5, 0.5]];
    let skin = transfer_skin(&input, &points).unwrap();
    assert_eq!(skin.indices[0], skin.indices[1]);
    assert_eq!(skin.weights[0], skin.weights[1]);
}
