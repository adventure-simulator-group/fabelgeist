//! A continuous bowed crossguard with a constant beveled blade mortise.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BladeMortise {
    pub width: Metres,
    pub thickness: Metres,
    pub bevel_width_ratio: Ratio,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MortisedGuardParameters {
    pub width: Metres,
    pub height: Metres,
    pub thickness: Metres,
    pub sweep: Metres,
    pub shoulder_height: Metres,
    pub edge_bevel: Metres,
    pub terminal_scale: Ratio,
    pub mortise: BladeMortise,
}
