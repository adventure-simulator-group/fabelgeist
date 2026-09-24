//! Precise manufacturing parameters owned by the canonical weapon kernel.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FirearmParameters {
    pub length: Metres,
    pub firearm_family: FirearmFirearmFamily,
    pub stock_style: FirearmStockStyle,
    pub lock_type: FirearmLockType,
    pub barrel_count: Count,
    pub barrel_length: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub secondary_barrel_length: Option<Metres>,
    pub bore: Metres,
    pub barrel_wall: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub barrel_separation: Option<Metres>,
    pub octagonal_ratio: Ratio,
    pub muzzle_style: FirearmMuzzleStyle,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub muzzle_flare: Option<Metres>,
    pub butt_width: Metres,
    pub waist_width: Metres,
    pub fore_width: Metres,
    pub stock_depth: Metres,
    pub butt_drop: Metres,
    pub lock_position: Metres,
    pub lock_wheel_radius: Metres,
    pub pan_width: Metres,
    pub trigger_length: Metres,
    pub guard_width: Metres,
    pub ramrod_radius: Metres,
    pub band_count: Count,
    pub sight_style: FirearmSightStyle,
    pub facing_style: FirearmFacingStyle,
    pub facing_thickness: Metres,
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
    pub barrel_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub lock_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub facing_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub furniture_material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub inlay_material: Option<Material>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LeadBallParameters {
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
pub struct BallPouchParameters {
    pub width: Metres,
    pub height: Metres,
    pub depth: Metres,
    pub wall: Metres,
    pub flap_length: Metres,
    pub flap_overlap: Metres,
    pub flap_angle: Degrees,
    pub belt_loop_width: Metres,
    pub belt_loop_gap: Metres,
    pub closure_style: BallPouchClosureStyle,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub hardware_material: Option<Material>,
}
