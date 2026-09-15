//! Numeric controls described at the serialized recipe boundary in metres.
use crate::{
    Material, WeaponDesign,
    recipe::{BladeCrossSection, BladePlan, Shape, SpearSection},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NumericEditorField {
    pub path: String,
    #[serde(default)]
    pub label: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

/// Controls at the serialized forge boundary. Choice values come from the
/// component's actual domain enum, so equal field names can have different sets.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum EditorField {
    Numeric(NumericEditorField),
    Choice {
        path: String,
        options: Vec<serde_json::Value>,
    },
}

pub fn editor_fields(design: &WeaponDesign) -> Vec<EditorField> {
    let mut fields: Vec<_> = numeric_editor_fields(design)
        .into_iter()
        .map(EditorField::Numeric)
        .collect();
    for (index, component) in design.recipe.components.iter().enumerate() {
        let mut choice = |name: &str, options| {
            fields.push(EditorField::Choice {
                path: format!("recipe.components.{index}.{name}"),
                options,
            })
        };
        choice(
            "material",
            [
                Material::Wood,
                Material::Leather,
                Material::DarkLeather,
                Material::Brass,
                Material::Steel,
                Material::DarkSteel,
            ]
            .into_iter()
            .map(|value| serde_json::to_value(value).expect("material choice"))
            .collect(),
        );
        match &component.shape {
            Shape::LoftedBlade(_) => {
                choice(
                    "section",
                    [BladeCrossSection::Diamond, BladeCrossSection::Fullered]
                        .into_iter()
                        .map(|value| serde_json::to_value(value).expect("blade section choice"))
                        .collect(),
                );
                choice(
                    "plan",
                    [BladePlan::Straight, BladePlan::Leaf, BladePlan::Cleaver]
                        .into_iter()
                        .map(|value| serde_json::to_value(value).expect("blade plan choice"))
                        .collect(),
                );
            }
            Shape::Spear(_) => choice(
                "section",
                [SpearSection::Flat, SpearSection::Diamond]
                    .into_iter()
                    .map(|value| serde_json::to_value(value).expect("spear section choice"))
                    .collect(),
            ),
            _ => {}
        }
    }
    fields
}
pub fn numeric_editor_fields(design: &WeaponDesign) -> Vec<NumericEditorField> {
    static FIELDS: OnceLock<BTreeMap<String, Vec<NumericEditorField>>> = OnceLock::new();
    let catalog = FIELDS.get_or_init(|| {
        serde_json::from_str(include_str!("../catalog/editor-fields.json"))
            .expect("authored editor fields")
    });
    let mut fields = Vec::new();
    for (index, component) in design.recipe.components.iter().enumerate() {
        let value = serde_json::to_value(component).expect("serializable component");
        let Some(kind) = value["kind"].as_str() else {
            continue;
        };
        if let Some(definitions) = catalog.get(kind) {
            for field in definitions {
                let pointer = format!("/{}", field.path.replace('.', "/"));
                if value
                    .pointer(&pointer)
                    .is_some_and(serde_json::Value::is_number)
                {
                    let mut field = field.clone();
                    field.label = field_label(&field.path);
                    field.path = format!("recipe.components.{index}.{}", field.path);
                    if !fields
                        .iter()
                        .any(|f: &NumericEditorField| f.path == field.path)
                    {
                        fields.push(field);
                    }
                }
            }
        }
        if let Shape::Socket(socket) = &component.shape {
            for station in 0..socket.profile.len() {
                for (axis, min, max) in [(0, 0.0, 1.0), (1, 0.001, 0.08)] {
                    fields.push(NumericEditorField {
                        path: format!("recipe.components.{index}.profile.{station}.{axis}"),
                        label: format!(
                            "Profile station {} {}",
                            station + 1,
                            if axis == 0 { "height" } else { "radius" }
                        ),
                        min,
                        max,
                        step: 0.001,
                    });
                }
            }
        }
    }
    fields
}

fn field_label(path: &str) -> String {
    match path {
        "size.0" => "Width (X)".into(),
        "size.1" => "Length (Y)".into(),
        "size.2" => "Depth (Z)".into(),
        _ => {
            let mut label = String::new();
            for c in path.chars() {
                if c == '.' {
                    label.push(' ');
                } else if c.is_uppercase() {
                    label.push(' ');
                    label.extend(c.to_lowercase());
                } else if label.is_empty() {
                    label.extend(c.to_uppercase());
                } else {
                    label.push(c);
                }
            }
            label
        }
    }
}
