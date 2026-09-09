//! Boundary-aligned front and rear breastplate carriers.
//!
//! The plates are authored as smooth low-dimensional lofts. Neck, armscye,
//! shoulder-band and skirt boundaries are evaluated in each loft's material
//! chart, so changing an opening cannot fold an otherwise regular carrier.

#[path = "breastplate_carrier/math.rs"]
mod math;
use math::*;
#[path = "breastplate_carrier/wearer.rs"]
mod wearer;
use wearer::*;
#[path = "breastplate_carrier/shape.rs"]
mod shape;
use shape::*;
#[path = "breastplate_carrier/grid.rs"]
mod grid;
use grid::*;
#[path = "breastplate_carrier/shell.rs"]
mod shell;
use shell::*;
#[path = "breastplate_carrier/fit.rs"]
mod fit;
use fit::*;
#[path = "breastplate_carrier/sampling.rs"]
mod sampling;
use sampling::*;

use std::collections::BTreeMap;

use crate::{
    ArmorMorph, BreastplateDesign, GenerateError, GeneratedArmor, TorsoClearancePose,
    TorsoShoulderSample, TorsoSurface, TorsoUpperRigAnchors, breastplate_design_hash,
    validate_breastplate,
};

const U_SAMPLES: usize = 49;
const V_SAMPLES: usize = 33;
const BAND_SAMPLES: usize = 9;
const SKIRT_SAMPLES: usize = 9;
const REFERENCE_RIG_NECK_HEIGHT: f32 = 1.441_910_7;
const REFERENCE_CARRIER_TOP_HEIGHT: f32 = 1.480;
const REFERENCE_SEMANTIC_HEIGHT: f32 = 0.502_445;
const REFERENCE_SHOULDER_HALF_WIDTH: f32 = 0.175_861_03;
const REFERENCE_TORSO_HALF_WIDTH: f32 = 0.187_162_74;
const REFERENCE_SECTION_CENTER_DEPTH: f32 = 0.020_309_05;
const REFERENCE_SECTION_RADIUS: f32 = 0.121_758_32;
const FIT_SURFACE_MARGIN: f32 = 0.001_5;
const MAX_FIT_CORRECTION: f32 = 0.060;

const FRONT_HEIGHTS: [f32; 8] = [1.038, 1.105, 1.185, 1.285, 1.355, 1.400, 1.445, 1.480];
const FRONT_RADIUS_X: [f32; 8] = [0.178, 0.180, 0.181, 0.183, 0.185, 0.174, 0.142, 0.132];
const FRONT_RADIUS_Z: [f32; 8] = [0.143, 0.153, 0.156, 0.151, 0.136, 0.115, 0.080, 0.050];
const FRONT_TRIM_HEIGHTS: [f32; 10] = [
    1.038, 1.105, 1.185, 1.250, 1.300, 1.335, 1.370, 1.400, 1.445, 1.480,
];
const FRONT_LIMIT_DEGREES: [f32; 10] = [92.0, 94.0, 94.0, 92.0, 78.0, 68.0, 58.0, 60.0, 72.0, 96.0];
const FRONT_NECK_Y: [f32; 3] = [1.425, 1.433, 1.458];

const BACK_HEIGHTS: [f32; 8] = [1.040, 1.120, 1.200, 1.280, 1.340, 1.400, 1.445, 1.480];
const BACK_RADIUS_X: [f32; 8] = [0.168, 0.170, 0.180, 0.195, 0.210, 0.205, 0.170, 0.150];
const BACK_RADIUS_Z: [f32; 8] = [0.096, 0.072, 0.085, 0.106, 0.109, 0.105, 0.088, 0.078];
const BACK_TRIM_HEIGHTS: [f32; 9] = [
    1.040, 1.120, 1.200, 1.280, 1.340, 1.380, 1.420, 1.460, 1.480,
];
const BACK_LIMIT_DEGREES: [f32; 9] = [75.0, 76.0, 78.0, 68.0, 60.0, 55.0, 52.0, 65.0, 70.0];
const BACK_NECK_Y: [f32; 3] = [1.420, 1.430, 1.455];
const NECK_U: [f32; 3] = [0.0, 0.25, 0.50];

#[derive(Clone, Copy)]
struct Frame {
    lateral: [f32; 3],
    vertical: [f32; 3],
    front: [f32; 3],
}

struct CarrierPose<'a> {
    positions: &'a [[f32; 3]],
    semantic: &'a [[f32; 2]],
    front: [f32; 3],
    anchors: TorsoUpperRigAnchors,
    coronal_depths: &'a [f32],
    shoulder_envelope: &'a [TorsoShoulderSample],
}

#[derive(Clone, Copy)]
struct Wearer<'a> {
    frame: Frame,
    anchors: TorsoUpperRigAnchors,
    clearance: &'a TorsoClearancePose,
    source_faces: &'a [[u32; 3]],
    torso_faces: &'a [[u32; 3]],
    x_scale: f32,
    shoulder_x_scale: f32,
    y_scale: f32,
    z_scale: f32,
    lateral_origin: f32,
    coronal_origin: f32,
}

#[derive(Default)]
struct MidMesh {
    positions: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    skirt_face_start: Option<usize>,
}

struct SolidMesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
    source_mid_indices: Vec<usize>,
}

#[derive(Clone, Copy)]
struct SourceSample {
    face: usize,
    weights: [f32; 3],
}

fn combine(front: SolidMesh, back: SolidMesh) -> SolidMesh {
    let offset = front.positions.len() as u32;
    let front_mid_count = front
        .source_mid_indices
        .iter()
        .copied()
        .max()
        .map_or(0, |index| index + 1);
    let mut positions = front.positions;
    positions.extend(back.positions);
    let mut normals = front.normals;
    normals.extend(back.normals);
    let mut indices = front.indices;
    indices.extend(back.indices.into_iter().map(|index| index + offset));
    let mut source_mid_indices = front.source_mid_indices;
    source_mid_indices.extend(
        back.source_mid_indices
            .into_iter()
            .map(|index| index + front_mid_count),
    );
    SolidMesh {
        positions,
        normals,
        indices,
        source_mid_indices,
    }
}

pub fn generate_breastplate(
    design: &BreastplateDesign,
    surface: &TorsoSurface,
) -> Result<GeneratedArmor, GenerateError> {
    validate_breastplate(design)?;
    if !valid_surface(surface) {
        return Err(GenerateError::InvalidSurface);
    }
    let base_positions = surface
        .vertices
        .iter()
        .map(|v| v.position)
        .collect::<Vec<_>>();
    let base_semantic = surface
        .vertices
        .iter()
        .map(|v| [v.lateral, v.vertical])
        .collect::<Vec<_>>();
    let base_coronal = surface
        .coronal_anchors
        .iter()
        .map(|v| v.depth)
        .collect::<Vec<_>>();
    let base_wearer = Wearer::new(
        CarrierPose {
            positions: &base_positions,
            semantic: &base_semantic,
            front: surface.front,
            anchors: surface.upper_rig_anchors,
            coronal_depths: &base_coronal,
            shoulder_envelope: &surface.shoulder_envelope,
        },
        &surface.clearance_mesh.base,
        &surface.clearance_mesh.enclosure_faces,
        &surface.clearance_mesh.enclosure_torso_faces,
    )?;
    let (front_mid, back_mid) = build_pair(base_wearer, design)?;
    let mid_positions = front_mid
        .positions
        .iter()
        .chain(&back_mid.positions)
        .copied()
        .collect::<Vec<_>>();
    let base = combine(
        solidify(front_mid, design.wall_thickness.metres())?,
        solidify(back_mid, design.wall_thickness.metres())?,
    );
    let source_face_indices = surface
        .clearance_mesh
        .enclosure_faces
        .iter()
        .copied()
        .enumerate()
        .map(|(index, face)| (face, index))
        .collect::<BTreeMap<_, _>>();
    let eligible_faces = surface
        .clearance_mesh
        .enclosure_torso_faces
        .iter()
        .map(|face| {
            source_face_indices
                .get(face)
                .copied()
                .ok_or(GenerateError::InvalidSurface)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let samples = mid_positions
        .iter()
        .map(|point| source_sample(*point, base_wearer, &eligible_faces))
        .collect::<Vec<_>>();
    let solid_samples = base
        .source_mid_indices
        .iter()
        .map(|index| samples[*index])
        .collect::<Vec<_>>();
    let texcoords = solid_samples
        .iter()
        .map(|sample| sampled_uv(*sample, surface))
        .collect::<Vec<_>>();
    let skin = solid_samples
        .iter()
        .map(|sample| sampled_skin(*sample, surface))
        .collect::<Vec<_>>();
    let morphs = generate_morphs(surface, design, &base)?;
    Ok(GeneratedArmor {
        design_hash: breastplate_design_hash(design)?,
        surface_domain: surface.domain.clone(),
        positions: base.positions,
        normals: base.normals,
        texcoords,
        joint_indices: skin.iter().map(|v| v.0).collect(),
        joint_weights: skin.iter().map(|v| v.1).collect(),
        indices: base.indices,
        morphs,
    })
}

fn generate_morphs(
    surface: &TorsoSurface,
    design: &BreastplateDesign,
    base: &SolidMesh,
) -> Result<Vec<ArmorMorph>, GenerateError> {
    surface
        .morphs
        .iter()
        .zip(&surface.morph_semantic_coordinates)
        .zip(&surface.morph_fronts)
        .zip(&surface.morph_upper_rig_anchors)
        .zip(&surface.morph_coronal_depths)
        .zip(&surface.clearance_mesh.morphs)
        .zip(&surface.morph_shoulder_envelopes)
        .map(
            |((((((morph, semantic), front), anchors), coronal), clearance), shoulders)| {
                let wearer = Wearer::new(
                    CarrierPose {
                        positions: &morph.positions,
                        semantic,
                        front: *front,
                        anchors: *anchors,
                        coronal_depths: coronal,
                        shoulder_envelope: shoulders,
                    },
                    clearance,
                    &surface.clearance_mesh.enclosure_faces,
                    &surface.clearance_mesh.enclosure_torso_faces,
                )?;
                let (front_mid, back_mid) = build_pair(wearer, design)?;
                let target = combine(
                    solidify(front_mid, design.wall_thickness.metres())?,
                    solidify(back_mid, design.wall_thickness.metres())?,
                );
                if target.indices != base.indices || target.positions.len() != base.positions.len()
                {
                    return Err(GenerateError::InvalidSurface);
                }
                Ok(ArmorMorph {
                    name: morph.name.clone(),
                    direct_positions: target.positions.clone(),
                    position_deltas: target
                        .positions
                        .iter()
                        .zip(&base.positions)
                        .map(|(a, b)| sub(*a, *b))
                        .collect(),
                    normal_deltas: target
                        .normals
                        .iter()
                        .zip(&base.normals)
                        .map(|(a, b)| sub(*a, *b))
                        .collect(),
                })
            },
        )
        .collect::<Result<Vec<_>, GenerateError>>()
}

const REPORT_FIT_ENV: &str = "BREASTPLATE_REPORT_FIT";

fn report_fit() -> bool {
    std::env::var_os(REPORT_FIT_ENV).is_some()
}
