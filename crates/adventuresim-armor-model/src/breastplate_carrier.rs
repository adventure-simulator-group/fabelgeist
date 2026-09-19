//! Boundary-aligned front and rear breastplate carriers.
//!
//! The plates are authored as smooth low-dimensional lofts. Neck, armscye,
//! and skirt boundaries are evaluated in each loft's material
//! chart, with a separate upper armscye curve and fitted lateral laps.

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
#[path = "breastplate_carrier/fluting.rs"]
mod fluting;
use fluting::*;
#[path = "breastplate_carrier/shell.rs"]
mod shell;
use shell::*;
#[path = "breastplate_carrier/fit.rs"]
mod fit;
use fit::*;
#[path = "breastplate_carrier/seating.rs"]
mod seating;
use seating::*;
#[path = "breastplate_carrier/sampling.rs"]
mod sampling;
use sampling::*;
#[path = "breastplate_carrier/anime.rs"]
mod anime;

use std::collections::BTreeMap;

use crate::{
    ArmorMorph, BreastplateDesign, GenerateError, GeneratedArmor, TorsoClearancePose,
    TorsoShoulderSample, TorsoSurface, TorsoUpperRigAnchors, breastplate_design_hash,
    validate_breastplate,
};

const U_SAMPLES: usize = 49;
const V_SAMPLES: usize = 33;
const SKIRT_SAMPLES: usize = 9;
const RUNTIME_FLUTED_U_INTERVALS: usize = 12;
const RUNTIME_FLUTED_V_INTERVALS: usize = 24;
const REFERENCE_RIG_NECK_HEIGHT: f32 = 1.441_910_7;
const REFERENCE_CARRIER_TOP_HEIGHT: f32 = 1.480;
const REFERENCE_SEMANTIC_HEIGHT: f32 = 0.502_445;
const REFERENCE_SHOULDER_HALF_WIDTH: f32 = 0.175_861_03;
const REFERENCE_TORSO_HALF_WIDTH: f32 = 0.187_162_74;
const REFERENCE_SECTION_CENTER_DEPTH: f32 = 0.020_309_05;
const REFERENCE_SECTION_RADIUS: f32 = 0.121_758_32;
// Padding reserve for interpolation and negative identity-morph blends.
const FIT_SURFACE_MARGIN: f32 = 0.006;
const MAX_FIT_CORRECTION: f32 = 0.060;

const FRONT_HEIGHTS: [f32; 8] = [1.038, 1.105, 1.185, 1.285, 1.355, 1.400, 1.445, 1.480];
const FRONT_RADIUS_X: [f32; 8] = [0.178, 0.180, 0.181, 0.183, 0.185, 0.174, 0.142, 0.132];
const FRONT_RADIUS_Z: [f32; 8] = [0.143, 0.153, 0.156, 0.151, 0.136, 0.115, 0.080, 0.050];
const FRONT_TRIM_HEIGHTS: [f32; 10] = [
    1.038, 1.105, 1.185, 1.250, 1.300, 1.335, 1.370, 1.400, 1.445, 1.480,
];
const FRONT_LIMIT_DEGREES: [f32; 10] = [92.0, 94.0, 94.0, 82.0, 68.0, 60.0, 52.0, 60.0, 72.0, 96.0];
const FRONT_NECK_Y: [f32; 3] = [1.425, 1.433, 1.458];

const BACK_HEIGHTS: [f32; 8] = [1.040, 1.120, 1.200, 1.280, 1.340, 1.400, 1.445, 1.480];
const BACK_RADIUS_X: [f32; 8] = [0.168, 0.170, 0.180, 0.195, 0.210, 0.205, 0.170, 0.150];
const BACK_RADIUS_Z: [f32; 8] = [0.080, 0.086, 0.092, 0.097, 0.099, 0.098, 0.088, 0.078];
const BACK_TRIM_HEIGHTS: [f32; 9] = [
    1.040, 1.120, 1.200, 1.280, 1.340, 1.380, 1.420, 1.460, 1.480,
];
const BACK_LIMIT_DEGREES: [f32; 9] = [89.0, 89.0, 74.0, 54.0, 46.0, 48.0, 46.0, 65.0, 70.0];
const BACK_NECK_Y: [f32; 3] = [1.420, 1.430, 1.455];
const NECK_U: [f32; 3] = [0.0, 0.25, 0.50];

#[derive(Clone, Copy)]
struct Frame {
    lateral: [f32; 3],
    vertical: [f32; 3],
    front: [f32; 3],
}

struct CarrierPose<'a> {
    detail: crate::ArmorDetail,
    positions: &'a [[f32; 3]],
    semantic: &'a [[f32; 2]],
    front: [f32; 3],
    anchors: TorsoUpperRigAnchors,
    coronal_depths: &'a [f32],
    shoulder_envelope: &'a [TorsoShoulderSample],
}

#[derive(Clone, Copy)]
struct Wearer<'a> {
    detail: crate::ArmorDetail,
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

struct MidMesh {
    main_rows: usize,
    skirt_rows: usize,
    positions: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    skirt_face_start: Option<usize>,
    main_columns: usize,
    extrusion_normals: Option<Vec<[f32; 3]>>,
    morph_carrier: Option<MorphCarrier>,
    morph_samples: Option<Vec<MorphSample>>,
    medial_crease: Vec<bool>,
    crease_right: Vec<bool>,
}

impl Default for MidMesh {
    fn default() -> Self {
        Self {
            main_rows: V_SAMPLES,
            skirt_rows: SKIRT_SAMPLES,
            positions: Vec::new(),
            faces: Vec::new(),
            skirt_face_start: None,
            main_columns: 0,
            extrusion_normals: None,
            morph_carrier: None,
            morph_samples: None,
            medial_crease: Vec::new(),
            crease_right: Vec::new(),
        }
    }
}

struct MorphCarrier {
    positions: Vec<[f32; 3]>,
    samples: Vec<(usize, f32)>,
}

#[derive(Clone, Copy)]
struct MorphSample {
    endpoints: [SourceSample; 4],
    weights: [f32; 4],
}

struct SolidMesh {
    construction_faces: Vec<std::ops::Range<usize>>,
    plate_edges: Vec<[u32; 2]>,
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

struct SurfaceAttributes {
    texcoords: Vec<[f32; 2]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
    morphs: Vec<ArmorMorph>,
}

fn combine(front: SolidMesh, back: SolidMesh) -> SolidMesh {
    let mut construction_faces = front.construction_faces;
    construction_faces.extend(
        back.construction_faces
            .into_iter()
            .map(|range| range.start + front.indices.len()..range.end + front.indices.len()),
    );
    let offset = front.positions.len() as u32;
    let front_mid_count = front
        .source_mid_indices
        .iter()
        .copied()
        .max()
        .map_or(0, |index| index + 1);
    let mut plate_edges = front.plate_edges;
    plate_edges.extend(
        back.plate_edges
            .into_iter()
            .map(|edge| edge.map(|i| i + offset)),
    );
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
        construction_faces,
        plate_edges,
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
    let source_hash = breastplate_design_hash(design)?;
    let runtime_fluting = matches!(surface.detail, crate::ArmorDetail::Runtime(_))
        .then_some(design.fluting)
        .flatten();
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
            detail: surface.detail,
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
    let eligible_faces = eligible_torso_faces(surface)?;
    let source_sampler = SourceSampler::new(base_wearer, &eligible_faces);
    let (front_mid, back_mid) = build_pair(base_wearer, design)?;
    let mut front_mid = anime::articulate(front_mid, false, base_wearer, design, &source_sampler)?;
    let back_mid = anime::articulate(back_mid, true, base_wearer, design, &source_sampler)?;
    front_mid.triangulate_left_cut();
    let fluting_chart = runtime_breastplate_fluting_chart(
        runtime_fluting,
        &front_mid,
        &back_mid,
        base_wearer.frame,
    );
    let mid_positions = front_mid
        .positions
        .iter()
        .chain(&back_mid.positions)
        .copied()
        .collect::<Vec<_>>();
    let mut morph_samples = carrier_samples(&front_mid, &source_sampler, base_wearer.frame);
    morph_samples.extend(carrier_samples(
        &back_mid,
        &source_sampler,
        base_wearer.frame,
    ));
    let base = combine(
        solidify(front_mid, design.wall_thickness.metres())?,
        solidify(back_mid, design.wall_thickness.metres())?,
    );
    let mut attributes = transfer_surface_attributes(
        surface,
        &base,
        &source_sampler,
        base_wearer.frame,
        &mid_positions,
        &morph_samples,
    )?;
    let normal_map = breastplate_normal_map(
        runtime_fluting,
        fluting_chart,
        &base,
        base_wearer,
        &mut attributes,
    );
    Ok(GeneratedArmor {
        construction_faces: base.construction_faces,
        plate_edges: base.plate_edges,
        components: Vec::new(),
        design_hash: source_hash,
        surface_domain: surface.domain.clone(),
        positions: base.positions,
        normals: base.normals,
        texcoords: attributes.texcoords,
        normal_map,
        joint_indices: attributes.joint_indices,
        joint_weights: attributes.joint_weights,
        indices: base.indices,
        morphs: attributes.morphs,
    })
}

fn runtime_breastplate_fluting_chart(
    pattern: Option<crate::PlateFluting>,
    front: &MidMesh,
    back: &MidMesh,
    frame: Frame,
) -> Option<Vec<(usize, [f32; 2])>> {
    pattern.map(|pattern| {
        let mut samples = breastplate_fluting_chart(front, frame, pattern)
            .into_iter()
            .map(|coordinate| (0, coordinate))
            .collect::<Vec<_>>();
        samples.extend((0..back.positions.len()).map(|_| (0, [0.0, 0.0])));
        samples
    })
}

fn breastplate_normal_map(
    pattern: Option<crate::PlateFluting>,
    chart: Option<Vec<(usize, [f32; 2])>>,
    base: &SolidMesh,
    wearer: Wearer<'_>,
    attributes: &mut SurfaceAttributes,
) -> Option<crate::GeneratedNormalMap> {
    chart.map(|chart| {
        let samples = base
            .source_mid_indices
            .iter()
            .map(|index| chart[*index])
            .collect::<Vec<_>>();
        let tiles = [crate::fluting_texture::FlutingTile::new(
            pattern.expect("fluting chart requires a pattern"),
            0.38 * wearer.x_scale,
            0.44 * wearer.y_scale,
        )];
        let (normal_map, texcoords) = crate::fluting_texture::atlas(&tiles, &samples);
        attributes.texcoords = texcoords;
        normal_map
    })
}

fn breastplate_fluting_chart(
    mesh: &MidMesh,
    frame: Frame,
    pattern: crate::PlateFluting,
) -> Vec<[f32; 2]> {
    let main_vertices = mesh.positions.len().min(mesh.main_rows * mesh.main_columns);
    let heights = mesh.positions[..main_vertices]
        .iter()
        .map(|position| local(*position, frame)[1])
        .collect::<Vec<_>>();
    let bottom = heights.iter().copied().fold(f32::INFINITY, f32::min);
    let top = heights.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let reference_start = mesh.main_rows / 2 * mesh.main_columns;
    let reference = mesh.positions[reference_start..reference_start + mesh.main_columns]
        .iter()
        .map(|position| local(*position, frame)[0])
        .collect::<Vec<_>>();
    let reference_center = reference[mesh.main_columns / 2];
    let half_width = ((reference[mesh.main_columns - 1] - reference[0]) * 0.5)
        .min(reference_center - reference[0])
        .min(reference[mesh.main_columns - 1] - reference_center);
    let mut coordinates = Vec::with_capacity(mesh.positions.len());
    for row in 0..mesh.main_rows {
        let row_start = row * mesh.main_columns;
        if row_start == main_vertices {
            break;
        }
        let row_end = (row_start + mesh.main_columns).min(main_vertices);
        let lateral = mesh.positions[row_start..row_end]
            .iter()
            .map(|position| local(*position, frame)[0])
            .collect::<Vec<_>>();
        let center = lateral[lateral.len() / 2];
        for (column, lateral) in lateral.iter().enumerate() {
            let index = row_start + column;
            let v = ((heights[index] - bottom) / (top - bottom)).clamp(0.0, 1.0);
            let mapped = (0.5 + (*lateral - center) / (2.0 * half_width)).clamp(0.0, 1.0);
            coordinates.push([pattern.unfan_coordinate(mapped, v), v]);
        }
    }
    coordinates.resize(mesh.positions.len(), [0.0, 0.0]);
    coordinates
}

fn transfer_surface_attributes(
    surface: &TorsoSurface,
    base: &SolidMesh,
    source_sampler: &SourceSampler,
    frame: Frame,
    mid_positions: &[[f32; 3]],
    morph_samples: &[MorphSample],
) -> Result<SurfaceAttributes, GenerateError> {
    let samples = mid_positions
        .iter()
        .zip(morph_samples)
        .map(|(point, morph)| {
            if morph.weights == [1.0, 0.0, 0.0, 0.0] {
                morph.endpoints[0]
            } else {
                source_sampler.sample(*point, frame)
            }
        })
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
    let solid_morph_samples = base
        .source_mid_indices
        .iter()
        .map(|index| morph_samples[*index])
        .collect::<Vec<_>>();
    let morphs = generate_morphs(surface, base, &solid_morph_samples)?;
    Ok(SurfaceAttributes {
        texcoords,
        joint_indices: skin.iter().map(|v| v.0).collect(),
        joint_weights: skin.iter().map(|v| v.1).collect(),
        morphs,
    })
}

/// Transfer identity displacement through fixed body correspondence. Refitting
/// each positive endpoint independently introduces nonlinear fitting corrections
/// that accumulate when many signed identity channels are combined.
fn generate_morphs(
    surface: &TorsoSurface,
    base: &SolidMesh,
    samples: &[MorphSample],
) -> Result<Vec<ArmorMorph>, GenerateError> {
    surface
        .morphs
        .iter()
        .zip(&surface.clearance_mesh.morphs)
        .map(|(morph, clearance)| {
            let position_deltas = samples
                .iter()
                .map(|sample| {
                    let displacement = sample.endpoints.map(|endpoint| {
                        surface.clearance_mesh.enclosure_faces[endpoint.face]
                            .into_iter()
                            .zip(endpoint.weights)
                            .fold([0.0; 3], |sum, (index, weight)| {
                                let original = surface.clearance_mesh.base.enclosure_vertices
                                    [index as usize]
                                    .position;
                                let target = clearance.enclosure_vertices[index as usize].position;
                                add(sum, scale(sub(target, original), weight))
                            })
                    });
                    displacement
                        .into_iter()
                        .zip(sample.weights)
                        .fold([0.0; 3], |sum, (delta, weight)| {
                            add(sum, scale(delta, weight))
                        })
                })
                .collect::<Vec<_>>();
            let direct_positions = base
                .positions
                .iter()
                .zip(&position_deltas)
                .map(|(point, delta)| add(*point, *delta))
                .collect::<Vec<_>>();
            let normals = vertex_normals(&direct_positions, base.indices.as_chunks::<3>().0)?;
            Ok(ArmorMorph {
                name: morph.name.clone(),
                direct_positions,
                position_deltas,
                normal_deltas: normals
                    .iter()
                    .zip(&base.normals)
                    .map(|(a, b)| sub(*a, *b))
                    .collect(),
            })
        })
        .collect()
}

const REPORT_FIT_ENV: &str = "BREASTPLATE_REPORT_FIT";

fn report_fit() -> bool {
    std::env::var_os(REPORT_FIT_ENV).is_some()
}
