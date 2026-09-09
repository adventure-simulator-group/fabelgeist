use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Millimeters(pub u16);

impl Millimeters {
    pub fn metres(self) -> f32 {
        f32::from(self.0) / 1_000.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Permille(pub u16);

impl Permille {
    pub fn unit(self) -> f32 {
        f32::from(self.0) / 1_000.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct BracerDesign {
    pub catalog_id: String,
    /// Fraction of the canonical elbow-to-wrist forearm span.
    pub coverage: Permille,
    /// Gap between the distal edge and the wrist, along the same span.
    pub wrist_offset: Permille,
    /// Radial air/padding gap between skin and the inner metal surface.
    pub clearance: Millimeters,
    /// Physical distance between the inner and outer surfaces.
    pub wall_thickness: Millimeters,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct BreastplateDesign {
    pub catalog_id: String,
    /// Scale of the neck opening around the wearer-derived default.
    pub neck_width: Permille,
    /// Scale of the front and rear neckline drop.
    pub neck_depth: Permille,
    /// Scale of the armscye depth.
    pub arm_opening_depth: Permille,
    /// Scale of the lower plate width around the wearer-derived default.
    pub waist_width: Permille,
    /// Scale of neck-to-waist plate length.
    pub plate_length: Permille,
    /// Scale of the side return toward the coronal torso plane.
    pub side_return: Permille,
    /// Additional smooth longitudinal crown on the front plate.
    pub front_crown: Millimeters,
    /// Physical width of each floating shoulder band.
    pub shoulder_band_width: Millimeters,
    /// Scale of the default short skirt length.
    pub skirt_length: Permille,
    /// Outward flare of the skirt's lower edge.
    pub skirt_flare: Millimeters,
    /// Inner-surface clearance for the front plate.
    pub front_clearance: Millimeters,
    /// Inner-surface clearance for the rear plate.
    pub back_clearance: Millimeters,
    /// Minimum depth-plane separation between the independent plates.
    pub plate_gap: Millimeters,
    pub wall_thickness: Millimeters,
}

impl Default for BreastplateDesign {
    fn default() -> Self {
        Self {
            catalog_id: "breastplate".into(),
            neck_width: Permille(1_000),
            neck_depth: Permille(1_000),
            arm_opening_depth: Permille(1_000),
            waist_width: Permille(1_000),
            plate_length: Permille(1_000),
            side_return: Permille(1_000),
            front_crown: Millimeters(6),
            shoulder_band_width: Millimeters(30),
            skirt_length: Permille(1_000),
            skirt_flare: Millimeters(30),
            front_clearance: Millimeters(10),
            back_clearance: Millimeters(14),
            plate_gap: Millimeters(8),
            wall_thickness: Millimeters(4),
        }
    }
}

impl Default for BracerDesign {
    fn default() -> Self {
        Self {
            catalog_id: "vambrace".into(),
            coverage: Permille(650),
            wrist_offset: Permille(20),
            clearance: Millimeters(12),
            wall_thickness: Millimeters(3),
        }
    }
}

impl BracerDesign {
    pub fn bracelet() -> Self {
        Self {
            coverage: Permille(120),
            wrist_offset: Permille(10),
            wall_thickness: Millimeters(2),
            ..Self::default()
        }
    }

    pub fn full_forearm() -> Self {
        Self {
            coverage: Permille(1_000),
            wrist_offset: Permille(0),
            ..Self::default()
        }
    }

    pub fn axial_interval(&self) -> (f32, f32) {
        let end = 1.0 - self.wrist_offset.unit();
        (end - self.coverage.unit(), end)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceVertex {
    /// Stable coordinate in the canonical anatomical atlas.
    pub uv: [f32; 2],
    /// Normalized elbow-to-wrist coordinate, independent of body dimensions.
    pub axial: f32,
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub joint_indices: [u32; 8],
    pub joint_weights: [f32; 8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceMorph {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnatomicalSurface {
    pub domain: String,
    pub vertices: Vec<SurfaceVertex>,
    pub faces: Vec<[u32; 3]>,
    pub morphs: Vec<SurfaceMorph>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsoVertex {
    pub uv: [f32; 2],
    /// Left-to-right coordinate centered on the sternum.
    pub lateral: f32,
    /// Stomach-to-neck coordinate defined by spinal landmarks.
    pub vertical: f32,
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub joint_indices: [u32; 8],
    pub joint_weights: [f32; 8],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsoCoronalAnchor {
    pub vertical: f32,
    pub depth: f32,
}

pub const TORSO_SHOULDER_ENVELOPE_SAMPLES: usize = 65;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsoShoulderSample {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct TorsoClearancePose {
    /// Cropped support/query vertices; these do not define a closed solid.
    pub vertices: Vec<TorsoShoulderSample>,
    /// Full body vertices in the separate enclosure face index domain.
    pub enclosure_vertices: Vec<TorsoShoulderSample>,
}

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ClearancePoseError {
    #[error("body enclosure position and normal arrays do not correspond")]
    MismatchedArrays,
    #[error("cropped clearance vertex is outside the body enclosure index domain")]
    InvalidCroppedIndex,
}

impl TorsoClearancePose {
    pub fn from_full_body(
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        cropped_indices: &[usize],
    ) -> Result<Self, ClearancePoseError> {
        if positions.len() != normals.len() {
            return Err(ClearancePoseError::MismatchedArrays);
        }
        if cropped_indices.iter().any(|i| *i >= positions.len()) {
            return Err(ClearancePoseError::InvalidCroppedIndex);
        }
        let enclosure_vertices = positions
            .iter()
            .zip(normals)
            .map(|(position, normal)| TorsoShoulderSample {
                position: *position,
                normal: *normal,
            })
            .collect::<Vec<_>>();
        let vertices = cropped_indices
            .iter()
            .map(|i| enclosure_vertices[*i])
            .collect();
        Ok(Self {
            vertices,
            enclosure_vertices,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TorsoClearanceMesh {
    pub base: TorsoClearancePose,
    /// Cropped upper-body faces, indexing each pose's `vertices`.
    pub faces: Vec<[u32; 3]>,
    /// Full body enclosure faces, indexing each pose's `enclosure_vertices`.
    pub enclosure_faces: Vec<[u32; 3]>,
    /// Torso-supported subset of the enclosure topology used for smooth
    /// front/back fit rays without treating the upper arms as torso volume.
    pub enclosure_torso_faces: Vec<[u32; 3]>,
    /// Full-body UV coordinates and face-corner indices, parallel to the
    /// enclosure topology. These preserve seams for rear armor sampling.
    pub enclosure_texcoords: Vec<[f32; 2]>,
    pub enclosure_texcoord_faces: Vec<[u32; 3]>,
    /// Full-body skin source, indexed by `enclosure_faces`.
    pub enclosure_joint_indices: Vec<[u32; 8]>,
    pub enclosure_joint_weights: Vec<[f32; 8]>,
    /// Paired cropped/enclosure poses, parallel to `TorsoSurface::morphs`.
    pub morphs: Vec<TorsoClearancePose>,
}

impl TorsoClearanceMesh {
    pub fn has_corresponding_domains(&self, morph_count: usize) -> bool {
        self.morphs.len() == morph_count
            && self.morphs.iter().all(|pose| {
                pose.vertices.len() == self.base.vertices.len()
                    && pose.enclosure_vertices.len() == self.base.enclosure_vertices.len()
            })
            && self
                .faces
                .iter()
                .flatten()
                .all(|i| (*i as usize) < self.base.vertices.len())
            && self
                .enclosure_faces
                .iter()
                .flatten()
                .all(|i| (*i as usize) < self.base.enclosure_vertices.len())
            && !self.enclosure_torso_faces.is_empty()
            && self
                .enclosure_torso_faces
                .iter()
                .flatten()
                .all(|i| (*i as usize) < self.base.enclosure_vertices.len())
            && self.enclosure_texcoord_faces.len() == self.enclosure_faces.len()
            && self
                .enclosure_texcoord_faces
                .iter()
                .flatten()
                .all(|i| (*i as usize) < self.enclosure_texcoords.len())
            && self.enclosure_joint_indices.len() == self.base.enclosure_vertices.len()
            && self.enclosure_joint_weights.len() == self.base.enclosure_vertices.len()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsoUpperRigAnchors {
    pub neck_base: [f32; 3],
    pub clavicles: [[f32; 3]; 2],
    pub shoulders: [[f32; 3]; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct TorsoSurface {
    pub domain: String,
    /// Model-space direction from the spine toward the anterior torso.
    pub front: [f32; 3],
    /// Per-morph anterior axes, parallel to `morphs`.  These are evaluated
    /// from each fitted rig rather than inherited from the neutral wearer.
    pub morph_fronts: Vec<[f32; 3]>,
    /// Stable rig-authored anchors for the neck/clavicle/shoulder yoke.
    pub upper_rig_anchors: TorsoUpperRigAnchors,
    /// Morph-corresponding upper anchors, parallel to `morphs`.
    pub morph_upper_rig_anchors: Vec<TorsoUpperRigAnchors>,
    /// Per-morph normalized garment-domain coordinates for `vertices`.
    /// Connectivity and vertex identity remain canonical, while each wearer
    /// receives its own semantic lateral/vertical embedding.
    pub morph_semantic_coordinates: Vec<Vec<[f32; 2]>>,
    pub vertices: Vec<TorsoVertex>,
    pub faces: Vec<[u32; 3]>,
    /// Full-body sagittal midpoints measured before front-surface filtering.
    pub coronal_anchors: Vec<TorsoCoronalAnchor>,
    /// Anchor depths for each morph, parallel to `morphs` and `coronal_anchors`.
    pub morph_coronal_depths: Vec<Vec<f32>>,
    /// Full-body shoulder-top surface anchors measured before front filtering.
    pub shoulder_envelope: Vec<TorsoShoulderSample>,
    /// Ordered shoulder envelopes for each morph, parallel to `morphs`.
    pub morph_shoulder_envelopes: Vec<Vec<TorsoShoulderSample>>,
    /// Full upper-torso/neck/shoulder triangles used only for sparse anchor
    /// placement and collision-clearance validation.
    pub clearance_mesh: TorsoClearanceMesh,
    pub morphs: Vec<SurfaceMorph>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArmorMorph {
    pub name: String,
    /// Independently evaluated endpoint positions on the frozen production
    /// topology. Exporters use these to verify signed-delta encoding exactly;
    /// they are not a second mesh or alternate connectivity path.
    pub direct_positions: Vec<[f32; 3]>,
    pub position_deltas: Vec<[f32; 3]>,
    pub normal_deltas: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedArmor {
    pub design_hash: [u8; 32],
    pub surface_domain: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub joint_indices: Vec<[u32; 8]>,
    pub joint_weights: Vec<[f32; 8]>,
    pub indices: Vec<u32>,
    pub morphs: Vec<ArmorMorph>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DesignError {
    #[error("armor shape parameters are outside their supported ranges")]
    ParametricParameters,
    #[error("armor catalog ID cannot be empty")]
    EmptyCatalogId,
    #[error("bracer coverage must be between 50 and 1000 permille")]
    Coverage,
    #[error("bracer coverage and wrist offset leave the canonical forearm")]
    Placement,
    #[error("bracer wall thickness must be between 1 and 20 millimeters")]
    WallThickness,
    #[error("bracer clearance must be between 1 and 30 millimeters")]
    Clearance,
    #[error("armor recipe encoding failed")]
    Encoding,
    #[error("breastplate edge parameters are outside their supported ranges")]
    BreastplateEdges,
}
