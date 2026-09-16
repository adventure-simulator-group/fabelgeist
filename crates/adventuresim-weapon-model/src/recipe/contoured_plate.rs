//! A forged planar blank with dimensioned boundary curves and a central ridge.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum PlateBoundarySpan {
    Line {
        to: [Ratio; 2],
    },
    Cubic {
        controls: [[Ratio; 2]; 2],
        to: [Ratio; 2],
    },
}

impl PlateBoundarySpan {
    pub(crate) fn end(&self) -> [Ratio; 2] {
        match self {
            Self::Line { to } | Self::Cubic { to, .. } => *to,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlateThicknessStation {
    pub at: Ratio,
    pub edge: Metres,
    pub ridge: Metres,
    pub ridge_half_width: Metres,
    pub flat_half_width: Metres,
    /// Per-face recession at the midpoint of each ridge slope.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub hollow_depth: Option<Metres>,
}

/// A measured full thickness at a transverse fraction of the blank width.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlateSectionLandmark {
    pub across: Ratio,
    pub thickness: Metres,
}

/// Landmark identities follow the same ordered tracks at every axial station.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlateSectionStation {
    pub at: Ratio,
    pub profile: Vec<PlateSectionLandmark>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum PlateSurface {
    Ridge {
        stations: Vec<PlateThicknessStation>,
    },
    Profile {
        stations: Vec<PlateSectionStation>,
    },
}

impl PlateThicknessStation {
    pub(crate) fn hollow_depth(&self) -> f64 {
        self.hollow_depth.map_or(0.0, Metres::get)
    }
}

/// Coordinates are fractions of width and length. The final boundary span must
/// return to `start`. Thickness stations cover the complete axial interval.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContouredPlateParameters {
    pub width: Metres,
    pub length: Metres,
    pub start: [Ratio; 2],
    pub boundary: Vec<PlateBoundarySpan>,
    pub surface: PlateSurface,
}
