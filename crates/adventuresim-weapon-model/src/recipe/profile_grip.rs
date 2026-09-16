//! Visible grip envelopes and a separately authored material cover.
use super::*;

pub(crate) const MAX_GRIP_PROFILE_STATIONS: usize = 12;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GripStation {
    pub at: Ratio,
    pub width: Metres,
    pub depth: Metres,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GripCover {
    pub material: Material,
    /// Inward distance normal to the transverse polygon's sides.
    pub thickness: Metres,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileGripParameters {
    pub length: Metres,
    pub profile: Vec<GripStation>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub cover: Option<GripCover>,
}

impl ProfileGripParameters {
    pub(crate) fn maximum_width(&self) -> f64 {
        self.profile
            .iter()
            .map(|p| p.width.get())
            .fold(0.0, f64::max)
    }
}
