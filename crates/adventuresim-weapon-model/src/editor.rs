//! Numeric controls described at the serialized recipe boundary in metres.
use crate::{
    Material, WeaponDesign,
    recipe::{
        BladeCrossSection, BladePlan, FullerFaces, ProfileInterpolation, Shape, SpearSection,
    },
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
        component_choices(&component.shape, &mut choice);
    }
    fields
}
fn component_choices(shape: &Shape, mut choice: impl FnMut(&str, Vec<serde_json::Value>)) {
    match shape {
        Shape::LoftedBlade(p) => {
            choice("section", section_options(p.section));
            choice(
                "plan",
                [BladePlan::Straight, BladePlan::Leaf, BladePlan::Cleaver]
                    .into_iter()
                    .map(|value| serde_json::to_value(value).expect("blade plan choice"))
                    .collect(),
            );
        }
        Shape::SectionBlade(p) => choice(
            "section",
            section_options(p.section.unwrap_or(BladeCrossSection::Diamond)),
        ),
        Shape::Spear(_) => choice(
            "section",
            [SpearSection::Flat, SpearSection::Diamond]
                .into_iter()
                .map(|value| serde_json::to_value(value).expect("spear section choice"))
                .collect(),
        ),
        _ => {}
    }
    let fuller = match shape {
        Shape::LoftedBlade(p) => p.fuller.as_ref(),
        Shape::SectionBlade(p) => p.fuller.as_ref(),
        _ => None,
    };
    if let Some(fuller) = fuller {
        for groove in 0..fuller.grooves.len() {
            choice(
                &format!("fuller.grooves.{groove}.faces"),
                [FullerFaces::Front, FullerFaces::Back, FullerFaces::Both]
                    .into_iter()
                    .map(|v| serde_json::to_value(v).unwrap())
                    .collect(),
            );
        }
    }
    if let Shape::ProfileGrip(p) | Shape::ProfileBody(p) = shape
        && p.cover.is_some()
    {
        choice(
            "cover.material",
            [
                Material::Leather,
                Material::DarkLeather,
                Material::Cord,
                Material::Brass,
            ]
            .into_iter()
            .map(|v| serde_json::to_value(v).unwrap())
            .collect(),
        );
    }
    if let Shape::Guard(p) = shape
        && p.terminal_profile.is_some()
    {
        choice(
            "terminalProfile.interpolation",
            [ProfileInterpolation::Linear, ProfileInterpolation::Smooth]
                .into_iter()
                .map(|v| serde_json::to_value(v).unwrap())
                .collect(),
        );
    }
}
fn section_options(current: BladeCrossSection) -> Vec<serde_json::Value> {
    let sections = if current == BladeCrossSection::Recessed {
        vec![BladeCrossSection::Recessed]
    } else {
        vec![
            BladeCrossSection::Diamond,
            BladeCrossSection::Fullered,
            BladeCrossSection::Hexagonal,
            BladeCrossSection::Lenticular,
        ]
    };
    sections
        .into_iter()
        .map(|value| serde_json::to_value(value).expect("blade section choice"))
        .collect()
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
            append_numeric_fields(&mut fields, definitions, &value, index);
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
        append_profile_fields(&mut fields, &component.shape, index);
        if let Shape::Guard(guard) = &component.shape
            && let Some(profile) = &guard.terminal_profile
        {
            for station in 0..profile.stations.len() {
                for axis in 0..2 {
                    fields.push(NumericEditorField {
                        path: format!(
                            "recipe.components.{index}.terminalProfile.stations.{station}.{axis}"
                        ),
                        label: format!(
                            "Terminal station {} {}",
                            station + 1,
                            if axis == 0 { "length" } else { "radius" }
                        ),
                        min: 0.0,
                        max: 0.1,
                        step: 0.001,
                    });
                }
            }
        }
    }
    fields
}

fn append_profile_fields(fields: &mut Vec<NumericEditorField>, shape: &Shape, index: usize) {
    if let Shape::ProfileGrip(grip) | Shape::ProfileBody(grip) = shape {
        for station in 0..grip.profile.len() {
            for (name, min, max) in [
                ("at", 0.0, 1.0),
                (
                    "width",
                    0.001,
                    if matches!(shape, Shape::ProfileBody(_)) {
                        0.2
                    } else {
                        0.038
                    },
                ),
                (
                    "depth",
                    0.001,
                    if matches!(shape, Shape::ProfileBody(_)) {
                        0.2
                    } else {
                        0.028
                    },
                ),
            ] {
                if name == "at" && (station == 0 || station + 1 == grip.profile.len()) {
                    continue;
                }
                fields.push(NumericEditorField {
                    path: format!("recipe.components.{index}.profile.{station}.{name}"),
                    label: format!("Profile station {} {name}", station + 1),
                    min,
                    max,
                    step: 0.001,
                });
            }
        }
    }
}

fn append_numeric_fields(
    fields: &mut Vec<NumericEditorField>,
    definitions: &[NumericEditorField],
    value: &serde_json::Value,
    index: usize,
) {
    for definition in definitions {
        let paths = if definition.path.contains('*') {
            (0..value["fuller"]["grooves"].as_array().map_or(0, Vec::len))
                .map(|i| definition.path.replace('*', &i.to_string()))
                .collect()
        } else {
            vec![definition.path.clone()]
        };
        for path in paths {
            let pointer = format!("/{}", path.replace('.', "/"));
            if value
                .pointer(&pointer)
                .is_some_and(serde_json::Value::is_number)
            {
                let mut field = definition.clone();
                field.label = field_label(&path);
                field.path = format!("recipe.components.{index}.{path}");
                if !fields.iter().any(|f| f.path == field.path) {
                    fields.push(field);
                }
            }
        }
    }
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
