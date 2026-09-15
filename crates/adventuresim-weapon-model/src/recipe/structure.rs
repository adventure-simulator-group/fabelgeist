//! Precise manufacturing parameters owned by the canonical weapon kernel.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoxParameters {
    pub size: [Metres; 3],
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub fit_shaft_side: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SocketParameters {
    pub profile: Vec<[Metres; 2]>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub segments: Option<Count>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub fit_shaft: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub wall: Option<Metres>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollarParameters {
    pub width: Metres,
    pub radius: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub segments: Option<Count>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SleeveParameters {
    pub length: Metres,
    pub radius: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub top_radius: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub segments: Option<Count>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub fit_shaft: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub wall: Option<Metres>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GripParameters {
    pub length: Metres,
    pub radius: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub bottom_scale: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub top_scale: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub wraps: Option<Count>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub segments: Option<Count>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OvalGripParameters {
    pub length: Metres,
    pub width: Metres,
    pub thickness: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub bottom_scale: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub top_scale: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub segments: Option<Count>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SlabGripParameters {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub construction: Option<SlabGripConstruction>,
    pub length: Metres,
    pub width: Metres,
    pub thickness: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub scale_thickness: Option<Metres>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SlabGripConstruction {
    Homogeneous,
    TangAndScales,
}
