//! Precise manufacturing parameters owned by the canonical weapon kernel.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CrossbowParameters {
    pub length: Metres,
    pub butt_width: Metres,
    pub waist_width: Metres,
    pub nose_width: Metres,
    pub stock_thickness: Metres,
    pub stock_style: CrossbowStockStyle,
    pub butt_drop: Metres,
    pub lock_table_height: Metres,
    pub fore_end_rise: Metres,
    pub facing_style: CrossbowFacingStyle,
    pub facing_thickness: Metres,
    pub prod_construction: CrossbowProdConstruction,
    pub prod_position: Metres,
    pub prod_span: Metres,
    pub prod_depth: Metres,
    pub prod_thickness: Metres,
    pub prod_sweep: Metres,
    pub prod_tip_scale: Ratio,
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
    pub sinew_thickness: Option<Metres>,
    pub string_radius: Metres,
    pub serving_width: Metres,
    pub tip_loop_clearance: Metres,
    pub bridle_spacing: Metres,
    pub bridle_radius: Metres,
    pub nut_position: Metres,
    pub nut_radius: Metres,
    pub nut_width: Metres,
    pub nut_thickness: Metres,
    pub rail_height: Metres,
    pub trigger_length: Metres,
    pub groove_width: Metres,
    pub stirrup_width: Metres,
    pub stirrup_length: Metres,
    pub stirrup_bar: Metres,
    pub spanning_mode: CrossbowSpanningMode,
    pub spanning_bar: Metres,
    pub sight_style: CrossbowSightStyle,
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
    pub binding_material: Option<Material>,
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
pub struct CrossbowBoltParameters {
    pub length: Metres,
    pub shaft_radius: Metres,
    pub head_length: Metres,
    pub head_width: Metres,
    pub head_thickness: Metres,
    pub head_style: CrossbowBoltHeadStyle,
    pub bolt_use: CrossbowBoltBoltUse,
    pub fletching_length: Metres,
    pub fletching_height: Metres,
    pub fletching_count: Count,
    pub butt_length: Metres,
    pub butt_width: Metres,
    pub butt_height: Metres,
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
    pub butt_material: Option<Material>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoltQuiverParameters {
    pub carrier_style: BoltQuiverCarrierStyle,
    pub length: Metres,
    pub bottom_width: Metres,
    pub mouth_width: Metres,
    pub depth: Metres,
    pub wall: Metres,
    pub lining: Metres,
    pub hide_cover: Metres,
    pub strap_width: Metres,
    pub strap_thickness: Metres,
    pub strap_drop: Metres,
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
