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

#[derive(Clone, Debug, PartialEq)]
pub struct ArmorMorph {
    pub name: String,
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
}
