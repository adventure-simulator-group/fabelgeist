//! Explicit asset-backed parity test for the portable skeletal basis.

use adventuresim_character_creator::proportions::{joint_bases, model_parameters};
use adventuresim_core::character_proportions::{BodyProportion, CharacterProportions};
use burn::tensor::{Device, Tensor, TensorData};
use fabelgeist_mhr::{
    Mhr, MhrConfig,
    math::{Transform, rotate_vector},
};

fn evaluate(
    model: &Mhr,
    proportions: CharacterProportions,
    bent: bool,
) -> (Vec<Transform>, Vec<[f64; 3]>) {
    let device = Device::default();
    let mut parameters = model_parameters(model, proportions).unwrap();
    if bent {
        for name in ["l_elbow_bend", "r_elbow_bend", "l_knee_bend", "r_knee_bend"] {
            parameters[model.parameter_transform.parameter_index(name).unwrap()] = 0.4;
        }
    }
    let output = model
        .forward_with(
            Tensor::from_data(TensorData::new(vec![0.15; 45], [1, 45]), &device),
            Tensor::from_data(
                TensorData::new(parameters, [1, model.num_model_parameters()]),
                &device,
            ),
            None,
            false,
        )
        .unwrap();
    let skeleton = output.skeleton_state.into_data().into_vec::<f32>().unwrap();
    let vertices = output.vertices.into_data().into_vec::<f32>().unwrap();
    let transforms = skeleton
        .as_chunks::<8>()
        .0
        .iter()
        .map(|state| Transform {
            translation: [
                state[0] as f64 / 100.0,
                state[1] as f64 / 100.0,
                state[2] as f64 / 100.0,
            ],
            rotation: [
                state[3] as f64,
                state[4] as f64,
                state[5] as f64,
                state[6] as f64,
            ],
            scale: state[7] as f64,
        })
        .collect();
    (
        transforms,
        vertices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|v| {
                [
                    v[0] as f64 / 100.0,
                    v[1] as f64 / 100.0,
                    v[2] as f64 / 100.0,
                ]
            })
            .collect(),
    )
}

#[test]
#[ignore = "requires the pinned MHR assets and a GPU; run with --ignored"]
fn skeletal_proportions_match_mhr_joints_and_skinned_vertices_in_motion() {
    let assets = std::env::var_os("MHR_ASSETS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/mhr-assets/v1.0.1/assets")
        });
    let model = Mhr::from_files(
        &assets,
        MhrConfig {
            lod: 1,
            pose_correctives: false,
        },
        &Device::default(),
    )
    .unwrap();
    let reference = CharacterProportions::default();
    let bases = joint_bases(&model, reference).unwrap();
    let (neutral, vertices) = evaluate(&model, reference, false);
    for sign in [-1.0, 1.0] {
        let mut proportions = reference;
        for p in BodyProportion::ALL {
            proportions.set(p, sign * p.limit()).unwrap();
        }
        for bent in [false, true] {
            let (source_pose, _) = evaluate(&model, reference, bent);
            let (expected, expected_vertices) = evaluate(&model, proportions, bent);
            let mut actual: Vec<Transform> = Vec::new();
            for (joint, source) in source_pose.iter().enumerate() {
                let parent = model.character.skeleton.parents[joint];
                let mut local = if parent < 0 {
                    *source
                } else {
                    source_pose[parent as usize].inverse().compose(source)
                };
                for (axis, delta) in bases[joint].translation(proportions).iter().enumerate() {
                    local.translation[axis] += f64::from(*delta);
                }
                actual.push(if parent < 0 {
                    local
                } else {
                    actual[parent as usize].compose(&local)
                });
                for axis in 0..3 {
                    assert!(
                        (actual[joint].translation[axis] - expected[joint].translation[axis]).abs()
                            < 0.00002,
                        "joint {joint}, axis {axis}"
                    );
                }
            }
            let skin: Vec<_> = actual
                .iter()
                .zip(&neutral)
                .map(|(pose, bind)| pose.compose(&bind.inverse()))
                .collect();
            let mut max_error = 0.0_f64;
            for (vertex, (reference, expected)) in
                vertices.iter().zip(&expected_vertices).enumerate()
            {
                let mut actual = [0.0; 3];
                for (joint, weight) in model.character.skin_weights.index[vertex]
                    .iter()
                    .zip(model.character.skin_weights.weight[vertex])
                {
                    let transform = skin[*joint as usize];
                    let rotated =
                        rotate_vector(transform.rotation, reference.map(|v| v * transform.scale));
                    for axis in 0..3 {
                        actual[axis] +=
                            (rotated[axis] + transform.translation[axis]) * f64::from(weight);
                    }
                }
                for axis in 0..3 {
                    max_error = max_error.max((actual[axis] - expected[axis]).abs());
                }
            }
            assert!(
                max_error < 0.0001,
                "skin must deform once, max error {max_error} metres"
            );
            println!("sign={sign}, bent={bent}, maximum skin error={max_error} metres");
        }
    }
}
