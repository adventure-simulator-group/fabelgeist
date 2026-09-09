use adventuresim_character_creator::{
    bracer::ForearmMorphSample,
    breastplate::{TorsoSurfaceInput, build_front_torso_surface},
};

struct TorsoFixture {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    texcoords: Vec<[f32; 2]>,
    joint_names: Vec<String>,
    states: Vec<[f32; 8]>,
    indices: Vec<[u32; 8]>,
    weights: Vec<[f32; 8]>,
    morphs: Vec<ForearmMorphSample>,
}

impl TorsoFixture {
    fn new() -> Self {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut faces = Vec::new();
        let mut texcoords = Vec::new();
        for depth in [-0.1, 0.1] {
            let offset = positions.len() as u32;
            for row in 0..=16 {
                for column in 0..=16 {
                    positions.push([column as f32 / 32.0 - 0.25, row as f32 / 16.0, depth]);
                    normals.push([0.0, 0.0, depth.signum()]);
                    texcoords.push([column as f32 / 16.0, row as f32 / 16.0]);
                }
            }
            for row in 0..16 {
                for column in 0..16 {
                    let a = offset + row * 17 + column;
                    faces.extend([[a, a + 1, a + 17], [a + 1, a + 18, a + 17]]);
                }
            }
        }
        let joints = [
            ("c_spine0", [0.0, 0.0, 0.0]),
            ("c_neck", [0.0, 1.0, 0.0]),
            ("l_clavicle", [0.12, 0.9, 0.0]),
            ("r_clavicle", [-0.12, 0.9, 0.0]),
            ("l_uparm", [0.2, 0.85, 0.0]),
            ("r_uparm", [-0.2, 0.85, 0.0]),
            ("c_head", [0.0, 1.2, 0.0]),
            ("l_eye", [0.03, 1.2, 0.05]),
            ("r_eye", [-0.03, 1.2, 0.05]),
        ];
        let states = joints
            .iter()
            .map(|(_, [x, y, z])| [*x, *y, *z, 0.0, 0.0, 0.0, 1.0, 1.0])
            .collect::<Vec<_>>();
        let morphs = vec![ForearmMorphSample {
            name: "depth_shift".into(),
            positions: positions
                .iter()
                .map(|[x, y, z]| [*x, *y, z + 0.01])
                .collect(),
            normals: normals.clone(),
            global_joint_states: states
                .iter()
                .map(|state| {
                    let mut state = *state;
                    state[2] += 0.01;
                    state
                })
                .collect(),
        }];
        Self {
            indices: vec![[0; 8]; positions.len()],
            weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; positions.len()],
            joint_names: joints.iter().map(|(name, _)| (*name).into()).collect(),
            states,
            positions,
            normals,
            faces,
            texcoords,
            morphs,
        }
    }

    fn input(&self) -> TorsoSurfaceInput<'_> {
        TorsoSurfaceInput {
            domain: "synthetic_torso",
            positions: &self.positions,
            normals: &self.normals,
            faces: &self.faces,
            texcoords: &self.texcoords,
            texcoord_faces: &self.faces,
            joint_indices: &self.indices,
            joint_weights: &self.weights,
            joint_names: &self.joint_names,
            global_joint_states: &self.states,
            morphs: &self.morphs,
        }
    }
}

#[test]
fn torso_domain_preserves_translated_morph_correspondence() {
    let fixture = TorsoFixture::new();
    let surface = build_front_torso_surface(fixture.input()).unwrap();
    assert!(!surface.vertices.is_empty());
    assert_eq!(surface.morphs.len(), 1);
    assert_eq!(surface.morphs[0].positions.len(), surface.vertices.len());
    for (base, morphed) in surface.vertices.iter().zip(&surface.morphs[0].positions) {
        assert_eq!(base.position[..2], morphed[..2]);
        assert!((morphed[2] - base.position[2] - 0.01).abs() < 1e-6);
    }
    for (base, morph) in surface
        .coronal_anchors
        .iter()
        .zip(&surface.morph_coronal_depths[0])
    {
        assert!((*morph - base.depth - 0.01).abs() < 1e-6);
    }
    assert_eq!(
        surface.shoulder_envelope.len(),
        surface.morph_shoulder_envelopes[0].len()
    );
    for pair in surface.shoulder_envelope.windows(2) {
        assert!(pair[1].position[0] > pair[0].position[0]);
    }
    assert_eq!(
        surface.morph_semantic_coordinates[0].len(),
        surface.vertices.len()
    );
}

#[test]
fn torso_domain_rejects_inconsistent_arrays_and_missing_landmarks() {
    let mut fixture = TorsoFixture::new();
    fixture.normals.pop();
    assert!(
        build_front_torso_surface(fixture.input())
            .unwrap_err()
            .contains("inconsistent")
    );
    fixture.normals.push([0.0, 0.0, 1.0]);
    fixture.joint_names[1] = "missing_neck".into();
    assert!(
        build_front_torso_surface(fixture.input())
            .unwrap_err()
            .contains("c_neck")
    );
}
