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
    pub thickness: Vec<PlateThicknessStation>,
}
