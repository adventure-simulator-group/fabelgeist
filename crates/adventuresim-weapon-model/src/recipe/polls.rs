//! Impact heads, curved beaks and flanged mace construction dimensions.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FacetedBeakParameters {
    pub length: Metres,
    pub root: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub tip: Option<Metres>,
    pub thickness: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub direction: Option<Direction>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub set: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub bend_position: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub tip_thickness: Option<Metres>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BillParameters {
    pub length: Metres,
    pub width: Metres,
    pub hook: Metres,
    pub thickness: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub root: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub root_length: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub belly_position: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub hook_depth: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub hook_curvature: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub point_length: Option<Ratio>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PickParameters {
    pub length: Metres,
    pub radius: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub direction: Option<Direction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BeakParameters {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub bend_profile: Option<BeakBendProfile>,
    pub length: Metres,
    pub radius: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub direction: Option<Direction>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub curvature: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub thickness: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub root_section: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub tip_section: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub bend_position: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub droop: Option<Metres>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HammerParameters {
    pub length: Metres,
    pub face: Metres,
    pub neck: Metres,
    pub thickness: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub direction: Option<Direction>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub crown: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub neck_ratio: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub face_flare: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub crown_length: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub face_thickness: Option<Metres>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaceParameters {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub core_profile: Option<Vec<[Metres; 2]>>,
    pub length: Metres,
    pub root_radius: Metres,
    pub shoulder_radius: Metres,
    pub cusp_radius: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub cusp_height: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub concavity: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub crown_length: Option<Metres>,
    pub flanges: Count,
    pub flange_thickness: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub profile_samples: Option<Count>,
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
    pub waist: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub flange_depth: Option<Metres>,
}
