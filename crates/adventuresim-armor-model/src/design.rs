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
    /// Width of the neck opening across the upper front panel.
    pub neck_width: Permille,
    /// Distance the neckline descends below the shoulder peaks.
    pub neck_depth: Permille,
    /// Distance the armscyes descend below the shoulder peaks.
    pub arm_opening_depth: Permille,
    /// Width of the lower edge relative to the chest.
    pub waist_width: Permille,
    /// Height of the lower opening over the stomach.
    pub stomach_height: Permille,
    /// Strength of the plate fairing that suppresses anatomical detail.
    pub rigidity: Permille,
    /// Strength of the elliptical return from the sternum toward the flanks.
    pub wrap: Permille,
    /// Additional smooth forward crown at the center of the plate.
    pub crown: Millimeters,
    /// Length of the skirt below the waist rail, relative to torso height.
    pub skirt_length: Permille,
    /// Outward flare of the skirt's lower edge.
    pub skirt_flare: Millimeters,
    pub clearance: Millimeters,
    pub wall_thickness: Millimeters,
}

impl Default for BreastplateDesign {
    fn default() -> Self {
        Self {
            catalog_id: "breastplate".into(),
            neck_width: Permille(260),
            neck_depth: Permille(120),
            arm_opening_depth: Permille(420),
            waist_width: Permille(740),
            stomach_height: Permille(350),
            rigidity: Permille(1_000),
            wrap: Permille(1_000),
            crown: Millimeters(40),
            skirt_length: Permille(130),
            skirt_flare: Millimeters(90),
            clearance: Millimeters(10),
            wall_thickness: Millimeters(3),
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
pub struct TorsoClearanceMesh {
    pub vertices: Vec<TorsoShoulderSample>,
    pub faces: Vec<[u32; 3]>,
    /// Morph-corresponding vertices, parallel to `TorsoSurface::morphs`.
    pub morph_vertices: Vec<Vec<TorsoShoulderSample>>,
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
