//! Turned terminal profiles measured from the receiving member's end plane.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProfileInterpolation {
    Linear,
    Smooth,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalProfile {
    /// Axial offset and radius in metres. Equal offsets form annular shoulders.
    pub stations: Vec<[Metres; 2]>,
    pub interpolation: ProfileInterpolation,
}
