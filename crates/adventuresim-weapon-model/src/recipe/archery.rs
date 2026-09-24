//! Precise manufacturing parameters owned by the canonical weapon kernel.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArcheryBowParameters {
    pub construction: ArcheryBowConstruction,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub limb_section: Option<ArcheryBowLimbSection>,
    pub length: Metres,
    pub grip_length: Metres,
    pub limb_width: Metres,
    pub limb_depth: Metres,
    pub grip_width: Metres,
    pub grip_depth: Metres,
    pub tip_scale: Ratio,
    pub reflex: Metres,
    pub recurve: Metres,
    pub upper_ratio: Ratio,
    pub brace_height: Metres,
    pub string_radius: Metres,
    pub loop_radius: Metres,
    pub loop_gap: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub backing_thickness: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub horn_thickness: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub samples: Option<Count>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub radial_segments: Option<Count>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub string_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub nock_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub core_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub horn_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub back_material: Option<Material>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArrowParameters {
    pub length: Metres,
    pub shaft_radius: Metres,
    pub head_length: Metres,
    pub head_width: Metres,
    pub head_thickness: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub head_style: Option<ArrowHeadStyle>,
    pub fletching_length: Metres,
    pub fletching_height: Metres,
    pub fletching_count: Count,
    pub nock_length: Metres,
    pub nock_slot_width: Metres,
    pub maximum_string_radius: Metres,
    pub nock_clearance: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub nock_style: Option<ArrowNockStyle>,
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
    pub head_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub fletching_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub nock_material: Option<Material>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArrowQuiverParameters {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub carrier_style: Option<ArrowQuiverCarrierStyle>,
    pub length: Metres,
    pub mouth_radius: Metres,
    pub bottom_scale: Ratio,
    pub wall: Metres,
    pub rim_radius: Metres,
    pub strap_width: Metres,
    pub strap_thickness: Metres,
    pub strap_drop: Metres,
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
    pub rim_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub strap_material: Option<Material>,
}
