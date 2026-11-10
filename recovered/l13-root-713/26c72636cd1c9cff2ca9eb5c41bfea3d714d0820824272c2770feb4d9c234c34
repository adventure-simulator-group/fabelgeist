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
    pub const ONE: Self = Self(1_000);

    pub fn unit(self) -> f32 {
        f32::from(self.0) / 1_000.0
    }
}

/// A nonnegative angle stored in thousandths of a radian.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Milliradians(pub u16);
impl Milliradians {
    pub fn radians(self) -> f32 {
        f32::from(self.0) / 1_000.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BracerDesign {
    pub fluting: Option<crate::PlateFluting>,
    /// Extra opening radius; the middle of the forearm remains close fitting.
    pub elbow_flare: Millimeters,
    pub wrist_flare: Millimeters,
    pub center_ridge: Millimeters,
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
            fluting: None,
            elbow_flare: Millimeters(0),
            wrist_flare: Millimeters(0),
            center_ridge: Millimeters(0),
            catalog_id: "vambrace".into(),
            coverage: Permille(650),
            wrist_offset: Permille(20),
            clearance: Millimeters(12),
            wall_thickness: Millimeters(3),
        }
    }
}

impl BracerDesign {
    pub(crate) fn columns(&self) -> Vec<f32> {
        const BASE_SEGMENTS: usize = 32;
        let mut columns = self.fluting.as_ref().map_or_else(
            || {
                (0..=BASE_SEGMENTS)
                    .map(|i| i as f32 / BASE_SEGMENTS as f32)
                    .collect()
            },
            |fluting| fluting.columns(BASE_SEGMENTS),
        );
        columns.pop();
        columns
    }

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

#[derive(Clone, Debug, PartialEq)]
pub struct ArmorMorph {
    pub name: String,
    /// Absolute endpoint positions on the frozen production
    /// topology. Exporters use these to verify signed-delta encoding exactly;
    /// they are not a second mesh or alternate connectivity path.
    pub direct_positions: Vec<[f32; 3]>,
    pub position_deltas: Vec<[f32; 3]>,
    pub normal_deltas: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedArmor {
    pub components: Vec<crate::ArmorComponent>,
    pub design_hash: [u8; 32],
    pub surface_domain: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub joint_indices: Vec<[u32; 8]>,
    pub joint_weights: Vec<[f32; 8]>,
    pub indices: Vec<u32>,
    /// The plate face of each triangle; empty for a piece that is not a
    /// thickened plate.
    pub faces: Vec<crate::PlateFace>,
    /// The band along the plates' edges, once [`GeneratedArmor::trimmed`].
    pub trim: Option<crate::ArmorTrim>,
    /// The outer face of every plate laid out on a grid, which a
    /// construction such as scales is laid over. Trimming renumbers the
    /// vertices, so a trimmed piece has none.
    pub grids: Vec<crate::SurfaceGrid>,
    pub morphs: Vec<ArmorMorph>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DesignError {
    #[error("breastplate profile parameters are outside their supported ranges")]
    BreastplateShape,
    #[error("breastplate flute parameters or fade spacing are outside their supported ranges")]
    PlateFluting,
    #[error(
        "visor slots do not fit their pattern span or row spacing with a 3 mm metal web; reduce count/size or increase spacing"
    )]
    VisorOpeningSpacing,
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
