//! Canonical precise weapon recipes, including museum authoring constructions.
//!
//! JSON is a boundary format only. Geometry consumes typed dimensions, materials,
//! sections and assembly choices; labels never select a construction algorithm.

/// Optional fields may be absent; a present field must contain its declared type.
fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}
mod archery;
mod choices;
mod crossbow;
pub(crate) mod facing;
mod firearm;
mod hilt;
mod material;
mod stock;
mod working_sections;
pub use stock::ForgeStock;
mod sectioned_blade;
pub use sectioned_blade::*;
mod blade_form;
pub use blade_form::*;
mod blade_clearance;
mod blade_profile;
pub(crate) use blade_profile::BladeProfile;
mod profile_grip;
pub use profile_grip::*;
mod terminal_profile;
pub use terminal_profile::*;
mod assembly;
mod bent_bar;
pub use bent_bar::*;
mod melee;
mod spear;
pub use spear::*;
mod shaft;
pub use shaft::*;
mod polls;
pub use polls::*;
mod quantities;
mod shape;
mod shield;
mod structure;
mod validation;
pub use validation::RecipeError;

pub use archery::*;
pub use assembly::*;
pub use choices::*;
pub use crossbow::*;
pub use firearm::*;
pub use hilt::*;
pub use material::*;
pub use melee::*;
pub use quantities::*;
use serde::{Deserialize, Serialize};
pub use shape::*;
pub use shield::*;
use std::collections::BTreeMap;
pub use structure::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Recipe {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub shaft: Option<Shaft>,
    pub components: Vec<Component>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub grip_clearance: Option<Metres>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Shaft {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub wrappings: Option<Vec<ShaftWrapping>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub tenon: Option<ShaftTenon>,
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
    pub segments: Option<Count>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub material: Option<Material>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    /// Keep this working end opposite another named working end on the same head.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub opposed_to: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub role: Option<crate::ComponentRole>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub label: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub offset: Option<[Metres; 3]>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub rotation: Option<[Degrees; 3]>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub mount: Option<Mount>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub attach: Option<Attachment>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub stretch_between: Option<[String; 2]>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub anchor: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub insertion: Option<Metres>,
    #[serde(flatten)]
    pub shape: Shape,
}

impl Component {
    pub(crate) fn resolved_id(&self, index: usize) -> String {
        self.id
            .clone()
            .or_else(|| self.label.clone())
            .unwrap_or_else(|| format!("component-{index}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_catalog_round_trips_without_losing_parameters() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../catalog/authoring.json")).unwrap();
        for preset in catalog["presets"].as_array().unwrap() {
            let json = &preset["definition"];
            let recipe: Recipe = serde_json::from_value(json.clone())
                .unwrap_or_else(|error| panic!("{}: {error}", preset["id"]));
            assert!(
                same_json(json, &serde_json::to_value(recipe).unwrap()),
                "{}",
                preset["id"]
            );
        }
    }

    fn same_json(a: &serde_json::Value, b: &serde_json::Value) -> bool {
        use serde_json::Value;
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => a.as_f64() == b.as_f64(),
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same_json(a, b))
            }
            (Value::Object(a), Value::Object(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .all(|(key, value)| b.get(key).is_some_and(|other| same_json(value, other)))
            }
            _ => a == b,
        }
    }

    #[test]
    fn recipe_rejects_unknown_nested_fields() {
        let recipe = serde_json::json!({"components":[{"kind":"grip","length":0.15,"radius":0.018,"unrecognized":1}]});
        assert!(serde_json::from_value::<Recipe>(recipe).is_err());
    }
}
