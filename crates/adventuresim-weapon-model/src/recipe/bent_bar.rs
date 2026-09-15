//! Constant-section bent round bars used for open hilt furniture.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "curve", rename_all = "camelCase", deny_unknown_fields)]
pub enum BarCenterline {
    Opposed {
        span: Metres,
        sweep: Metres,
    },
    Arch {
        width: Metres,
        length: Metres,
        bulge: Metres,
        side: Direction,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BentBarParameters {
    pub centerline: BarCenterline,
    pub radius: Metres,
    pub samples: Count,
    pub radial_segments: Count,
}
