//! The bracer fitted on the device to a synthetic forearm.

use fabelgeist_rig::RigJointName;
use std::collections::BTreeMap;
use std::sync::OnceLock;

use adventuresim_character_creator::bracer::{
    ForearmMorphSample, ForearmSide, ForearmSurfaceInput,
};
use adventuresim_character_creator::device_bracer::generate_bracer_on_device;
use fabelgeist_armor::{ArmorGpu, BracerDesign, GeneratedArmor, Millimeters, Permille};

const RINGS: usize = 13;
const SEGMENTS: usize = 24;
const RADIUS_M: f32 = 0.045;
const LENGTH_M: f32 = 0.3;
/// The morph sample's forearm is this much wider.
const WIDER: f32 = 1.3;

fn gpu() -> &'static ArmorGpu {
    static GPU: OnceLock<ArmorGpu> = OnceLock::new();
    GPU.get_or_init(|| ArmorGpu::open().expect("a compute device"))
}

/// A forearm-like cylinder from elbow to wrist, with an atlas seam and its
/// skin blended from the forearm joint to the wrist.
struct Forearm {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    texcoords: Vec<[f32; 2]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
    joint_names: Vec<RigJointName>,
    joints: Vec<[f32; 8]>,
    morphs: Vec<ForearmMorphSample>,
}

impl Forearm {
    fn new() -> Self {
        let mut forearm = Self {
            positions: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
            texcoords: Vec::new(),
            joint_indices: Vec::new(),
            joint_weights: Vec::new(),
            joint_names: ["l_lowarm", "l_wrist", "l_upperarm"]
                .map(RigJointName::from)
                .to_vec(),
            joints: [
                [0.0, LENGTH_M, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 2.0 * LENGTH_M, 0.0],
            ]
            .map(|[x, y, z]| [x, y, z, 0.0, 0.0, 0.0, 1.0, 1.0])
            .to_vec(),
            morphs: Vec::new(),
        };
        for ring in 0..RINGS {
            let axial = ring as f32 / (RINGS - 1) as f32;
            for segment in 0..=SEGMENTS {
                let angle = segment as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
                let normal = [angle.cos(), 0.0, angle.sin()];
                forearm.positions.push([
                    normal[0] * RADIUS_M,
                    LENGTH_M * (1.0 - axial),
                    normal[2] * RADIUS_M,
                ]);
                forearm.normals.push(normal);
                forearm
                    .texcoords
                    .push([segment as f32 / SEGMENTS as f32, axial]);
                forearm.joint_indices.push([0, 1, 0, 0, 0, 0, 0, 0]);
                let wrist = (axial - 0.8).max(0.0) * 2.0;
                forearm
                    .joint_weights
                    .push([1.0 - wrist, wrist, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
            }
        }
        let columns = (SEGMENTS + 1) as u32;
        for ring in 0..(RINGS - 1) as u32 {
            for segment in 0..SEGMENTS as u32 {
                let a = ring * columns + segment;
                let d = a + columns;
                forearm.faces.extend([[a, d + 1, d], [a, a + 1, d + 1]]);
            }
        }
        forearm.morphs.push(ForearmMorphSample {
            name: "wider".into(),
            positions: forearm
                .positions
                .iter()
                .map(|[x, y, z]| [x * WIDER, *y, z * WIDER])
                .collect(),
            normals: forearm.normals.clone(),
            global_joint_states: forearm.joints.clone(),
            device: Default::default(),
        });
        forearm
    }

    fn input(&self) -> ForearmSurfaceInput<'_> {
        ForearmSurfaceInput {
            domain: "synthetic_forearm",
            side: ForearmSide::Left,
            positions: &self.positions,
            normals: &self.normals,
            faces: &self.faces,
            texcoords: &self.texcoords,
            texcoord_faces: &self.faces,
            joint_indices: &self.joint_indices,
            joint_weights: &self.joint_weights,
            joint_names: &self.joint_names,
            global_joint_states: &self.joints,
            morphs: &self.morphs,
        }
    }

    fn bracer(&self, design: &BracerDesign) -> GeneratedArmor {
        generate_bracer_on_device(gpu(), design, self.input()).unwrap()
    }
}

/// Every edge, with positions welded, is used once in each direction.
fn assert_closed(armor: &GeneratedArmor) {
    let mut ids = BTreeMap::new();
    let welded = armor
        .positions
        .iter()
        .map(|p| {
            let next = ids.len();
            *ids.entry(p.map(|v| (v * 1e6).round() as i64))
                .or_insert(next)
        })
        .collect::<Vec<_>>();
    let mut edges = BTreeMap::<(usize, usize), i32>::new();
    for triangle in armor.indices.as_chunks::<3>().0 {
        let [a, b, c] = [0, 1, 2].map(|i| welded[triangle[i] as usize]);
        for (from, to) in [(a, b), (b, c), (c, a)] {
            *edges.entry((from, to)).or_default() += 1;
            *edges.entry((to, from)).or_default() -= 1;
        }
    }
    assert!(edges.values().all(|count| *count == 0), "the shell is open");
    assert!(
        armor
            .positions
            .iter()
            .chain(&armor.normals)
            .flatten()
            .all(|v| v.is_finite())
    );
}

fn axial_extent(positions: &[[f32; 3]]) -> [f32; 2] {
    positions
        .iter()
        .fold([f32::INFINITY, f32::NEG_INFINITY], |[lo, hi], p| {
            [lo.min(p[1]), hi.max(p[1])]
        })
}

fn largest_radius(positions: &[[f32; 3]]) -> f32 {
    positions
        .iter()
        .map(|p| p[0].hypot(p[2]))
        .fold(0.0, f32::max)
}

#[test]
fn presets_are_closed_solids_with_their_own_coverage_and_morphs() {
    let forearm = Forearm::new();
    let presets = [
        BracerDesign::bracelet(),
        BracerDesign::default(),
        BracerDesign::full_forearm(),
    ];
    let mut spans = Vec::new();
    for design in &presets {
        let armor = forearm.bracer(design);
        assert_closed(&armor);
        assert!(largest_radius(&armor.positions) > RADIUS_M);
        assert_eq!(armor.morphs.len(), 1);
        let morph = &armor.morphs[0];
        assert_eq!(morph.direct_positions.len(), armor.positions.len());
        assert!(largest_radius(&morph.direct_positions) > largest_radius(&armor.positions));
        let [lo, hi] = axial_extent(&armor.positions);
        spans.push(hi - lo);
    }
    assert!(spans[0] < spans[1] && spans[1] < spans[2], "{spans:?}");
}

#[test]
fn wall_thickness_changes_the_shell_and_the_design_hash() {
    let forearm = Forearm::new();
    let thin = BracerDesign {
        wall_thickness: Millimeters(2),
        ..BracerDesign::default()
    };
    let thick = BracerDesign {
        wall_thickness: Millimeters(6),
        ..BracerDesign::default()
    };
    let (a, b) = (forearm.bracer(&thin), forearm.bracer(&thick));
    assert_eq!(a.indices, b.indices);
    assert_ne!(a.positions, b.positions);
    assert_ne!(a.design_hash, b.design_hash);
}

#[test]
fn coverage_and_wrist_offset_move_the_rings_along_the_forearm() {
    let forearm = Forearm::new();
    let design = BracerDesign {
        coverage: Permille(400),
        wrist_offset: Permille(0),
        ..BracerDesign::default()
    };
    let shifted = BracerDesign {
        wrist_offset: Permille(300),
        ..design.clone()
    };
    let [low, high] = axial_extent(&forearm.bracer(&design).positions);
    let [shifted_low, shifted_high] = axial_extent(&forearm.bracer(&shifted).positions);
    assert!(shifted_low > low && shifted_high > high);
    assert!(((shifted_high - shifted_low) - (high - low)).abs() < 0.02);
}

#[test]
fn inconsistent_inputs_and_missing_landmarks_are_rejected() {
    let mut forearm = Forearm::new();
    forearm.normals.pop();
    assert!(generate_bracer_on_device(gpu(), &BracerDesign::default(), forearm.input()).is_err());
    let mut forearm = Forearm::new();
    forearm.joint_names[1] = "missing_wrist".into();
    let error =
        generate_bracer_on_device(gpu(), &BracerDesign::default(), forearm.input()).unwrap_err();
    assert!(format!("{error:#}").contains("l_wrist"), "{error:#}");
}
