//! Longitudinal blade lofts with a ricasso and an authored transverse section.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BladePlan {
    Straight,
    Leaf,
    Cleaver,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BladeCrossSection {
    Diamond,
    Fullered,
    Hexagonal,
    Lenticular,
    Recessed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpearSection {
    Diamond,
    Flat,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FigureEightConstruction {
    ForgedPlate,
    RoundTube,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoftedBladeParameters {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub fuller: Option<FullerParameters>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub point: Option<BladePoint>,
    pub length: Metres,
    pub width: Metres,
    pub thickness: Metres,
    pub curvature: Metres,
    pub plan: BladePlan,
    pub section: BladeCrossSection,
    pub samples: Count,
    pub taper: Ratio,
    pub single_edge: Ratio,
    pub belly: Ratio,
    pub ricasso: Metres,
}

/// A round member along an arbitrary spatial path, with an explicit section.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpatialTubeParameters {
    pub points: Vec<[Metres; 3]>,
    pub radius: Metres,
    pub radial_segments: Count,
}
