//! A typed parameter document supplies the values; the common schema supplies legal controls.
mod leaf;
use adventuresim_procedural_textures::{ControlBounds, ControlPath, TextureRecipeId};
use bevy_egui::egui::{self, Ui};
use serde_json::Value;

pub(crate) fn draw(
    ui: &mut Ui,
    recipe: TextureRecipeId,
    values: &mut Value,
    defaults: &Value,
    search: &str,
) -> bool {
    let allowed = recipe.control_paths();
    let mut changed = false;
    if let Some(seed) = values
        .get_mut("seed")
        .filter(|_| allowed.iter().any(|path| path.as_str() == "/seed"))
    {
        ui.horizontal(|ui| {
            ui.label("Seed");
            let mut number = seed.as_u64().unwrap_or_default();
            if ui.add(egui::DragValue::new(&mut number)).changed() {
                *seed = number.into();
                changed = true;
            }
            if ui
                .small_button("↻")
                .on_hover_text("Try the next deterministic seed")
                .clicked()
            {
                *seed = number.wrapping_add(1).into();
                changed = true;
            }
        });
    }
    let palette = match recipe {
        TextureRecipeId::HewnOak => Some("/hewn_oak/colors"),
        TextureRecipeId::HandmadeBrick => Some("/handmade_brick/colors"),
        TextureRecipeId::DressedStone => Some("/dressed_stone/colors"),
        _ => None,
    };
    if let Some(path) = palette
        && let (Some(value), Some(default)) = (values.pointer_mut(path), defaults.pointer(path))
    {
        egui::CollapsingHeader::new("Palette")
            .default_open(true)
            .show(ui, |ui| {
                changed |= node(ui, path, value, default, search, allowed);
            });
    }
    changed |= leaf::draw(ui, recipe, values, defaults, search);
    for group in groups(recipe) {
        let path = format!("/{group}");
        let Some(value) = values.pointer_mut(&path) else {
            continue;
        };
        let Some(default) = defaults.pointer(&path) else {
            continue;
        };
        ui.push_id(group, |ui| {
            egui::CollapsingHeader::new(label(group))
                .default_open(group != "hewn_oak")
                .open(if search.is_empty() { None } else { Some(true) })
                .show(ui, |ui| {
                    changed |= node(ui, &path, value, default, search, allowed);
                });
        });
    }
    if let Some(species) = leaf_palette(recipe) {
        let path = format!("/leaf_colors/{species}");
        if let (Some(value), Some(default)) = (values.pointer_mut(&path), defaults.pointer(&path)) {
            egui::CollapsingHeader::new("Leaf palette")
                .default_open(true)
                .show(ui, |ui| {
                    changed |= node(ui, &path, value, default, search, allowed);
                });
        }
    }
    changed
}

fn node(
    ui: &mut Ui,
    path: &str,
    value: &mut Value,
    default: &Value,
    search: &str,
    allowed: &[ControlPath],
) -> bool {
    let mut changed = false;
    match (value, default) {
        (Value::Object(values), Value::Object(defaults)) => {
            changed |= object(ui, path, values, defaults, search, allowed);
        }
        (Value::Array(values), Value::Array(defaults)) => {
            let knots = path.ends_with("/knots");
            let variable =
                knots || path == "/window_glass/bubbles" || path == "/window_glass/patches";
            let mut remove = None;
            for (index, value) in values.iter_mut().enumerate() {
                let default = defaults
                    .get(index)
                    .or_else(|| defaults.first())
                    .unwrap_or(&Value::Null);
                let child = format!("{path}/{index}");
                ui.push_id(index, |ui| {
                    if value.is_object() || value.is_array() {
                        ui.horizontal(|ui| {
                            ui.strong(format!(
                                "{} {}",
                                if knots { "Knot" } else { "Item" },
                                index + 1
                            ));
                            if variable && ui.small_button("Remove").clicked() {
                                remove = Some(index);
                            }
                        });
                        if color_value(path, value).is_some() {
                            changed |= color(ui, &child, value, default);
                        } else {
                            changed |= node(ui, &child, value, default, search, allowed);
                        }
                    } else {
                        changed |= scalar(ui, &child, value, default);
                    }
                });
            }
            if let Some(index) = remove {
                values.remove(index);
                changed = true;
            }
            if variable
                && values.len() < 16
                && ui
                    .button(if knots { "+ Add knot" } else { "+ Add feature" })
                    .clicked()
            {
                let mut knot = defaults[0].clone();
                if knots {
                    knot["center"] = serde_json::json!([0.5, 0.5]);
                }
                values.push(knot);
                changed = true;
            }
        }
        _ => {}
    }
    changed
}

fn object(
    ui: &mut Ui,
    path: &str,
    values: &mut serde_json::Map<String, Value>,
    defaults: &serde_json::Map<String, Value>,
    search: &str,
    allowed: &[ControlPath],
) -> bool {
    let root = path.split('/').count() == 2;
    let mut changed = false;
    let mut keys = values
        .keys()
        .filter(|key| {
            (!root
                || key.as_str() != "colors"
                    && allowed
                        .iter()
                        .any(|p| p.as_str() == format!("{path}/{key}")))
                && (search.is_empty()
                    || format!("{path}/{key}").replace('_', " ").contains(search)
                    || values[*key].to_string().replace('_', " ").contains(search))
        })
        .cloned()
        .collect::<Vec<_>>();
    keys.sort_by_key(|key| {
        (
            match key.as_str() {
                "colors" => 0,
                "knots" => 1,
                "tile_metres" => 2,
                "height_range_metres" => 3,
                _ => 4,
            },
            key.clone(),
        )
    });
    let advanced = |key: &str| {
        root && (key.starts_with("sample_")
            || key.starts_with("generate_")
            || key.split('_').count() > 4
            || key.ends_with("_1")
            || key.ends_with("_2")
            || key.ends_with("_3"))
    };
    for key in keys.iter().filter(|key| !advanced(key)) {
        changed |= field(ui, path, key, values, defaults, search, allowed);
    }
    let keys = keys.iter().filter(|key| advanced(key)).collect::<Vec<_>>();
    if !keys.is_empty() {
        egui::CollapsingHeader::new(format!("Advanced coefficients · {}",keys.len())).default_open(!search.is_empty()).show(ui,|ui| {
            ui.small("Fine control over individual features. Hover a value for its default and range.");
            for key in keys { changed |= field(ui,path,key,values,defaults,search,allowed); }
        });
    }
    changed
}
fn field(
    ui: &mut Ui,
    path: &str,
    key: &str,
    values: &mut serde_json::Map<String, Value>,
    defaults: &serde_json::Map<String, Value>,
    search: &str,
    allowed: &[ControlPath],
) -> bool {
    let Some(value) = values.get_mut(key) else {
        return false;
    };
    let Some(default) = defaults.get(key) else {
        return false;
    };
    let child = format!("{path}/{key}");
    let mut changed = false;
    ui.push_id(key, |ui| {
        if color_value(&child, value).is_some() {
            changed |= color(ui, &child, value, default);
        } else if value.is_array() || value.is_object() {
            egui::CollapsingHeader::new(label(key))
                .default_open(key == "colors" || key == "knots" || !search.is_empty())
                .show(ui, |ui| {
                    changed |= node(ui, &child, value, default, search, allowed);
                });
        } else {
            changed |= scalar(ui, &child, value, default);
        }
    });
    changed
}

fn scalar(ui: &mut Ui, path: &str, value: &mut Value, default: &Value) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        match (&mut *value, default) {
            (Value::Number(number), Value::Number(_)) if path.ends_with("/salt") => {
                let mut seed = number.as_u64().unwrap_or_default();
                if ui.add(egui::DragValue::new(&mut seed)).changed() {
                    *number = seed.into();
                    changed = true;
                }
            }
            (Value::Number(number), Value::Number(default)) => {
                let bounds = ControlBounds::for_default(path, default);
                let mut edited = number.as_f64().unwrap_or_default();
                let speed = (default.as_f64().unwrap_or(1.0).abs() * 0.01).max(0.000_01);
                let mut control = egui::DragValue::new(&mut edited)
                    .range(bounds.min..=bounds.max)
                    .speed(speed);
                if bounds.integer {
                    control = control.fixed_decimals(0);
                }
                let response = ui.add(control);
                if response.changed() {
                    if bounds.integer {
                        edited = edited.round();
                    }
                    *number = if number.is_u64() {
                        (edited as u64).into()
                    } else if number.is_i64() {
                        (edited as i64).into()
                    } else {
                        serde_json::Number::from_f64(edited).unwrap()
                    };
                    changed = true;
                }
                response.on_hover_text(format!(
                    "Default: {default}\nRange: {}–{}\n{path}",
                    bounds.min, bounds.max
                ));
            }
            (Value::Bool(value), Value::Bool(_)) => {
                changed |= ui.checkbox(value, "").changed();
            }
            _ => {
                ui.label(value.to_string());
            }
        }
        ui.label(control_label(path));
        if *value != *default
            && ui
                .small_button("↶")
                .on_hover_text("Reset this control")
                .clicked()
        {
            *value = default.clone();
            changed = true;
        }
    });
    changed
}

fn color_value(path: &str, value: &Value) -> Option<[u8; 3]> {
    let values = value.as_array()?;
    let color = path.contains("/colors/")
        || path.contains("/leaf_colors/") && !path.ends_with("roughness")
        || path.contains("srgb")
        || path.ends_with("transmitted_color")
        || path.contains("palette") && !path.contains("roughness");
    if !color || values.len() != 3 || !values.iter().all(Value::is_number) {
        return None;
    }
    let scale = if values.iter().all(|v| v.as_f64().unwrap() <= 1.0)
        && values.iter().any(|v| v.as_u64().is_none())
    {
        255.0
    } else {
        1.0
    };
    Some(std::array::from_fn(|i| {
        (values[i].as_f64().unwrap() * scale).round() as u8
    }))
}
fn color(ui: &mut Ui, path: &str, value: &mut Value, default: &Value) -> bool {
    let Some(mut rgb) = color_value(path, value) else {
        return false;
    };
    let mut changed = false;
    ui.horizontal(|ui| {
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            let normalized = default
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_u64().is_none());
            *value = if normalized {
                serde_json::json!(rgb.map(|v| f64::from(v) / 255.0))
            } else {
                serde_json::json!(rgb)
            };
            changed = true;
        }
        ui.label(label(path.rsplit('/').next().unwrap()));
        if *value != *default && ui.small_button("↶").clicked() {
            *value = default.clone();
            changed = true;
        }
    });
    changed
}

pub(crate) fn label(value: &str) -> String {
    if value == "hewn_oak_grain" {
        return "Growth rings and knots".into();
    }
    let words = value.replace(['_', '-'], " ");
    let mut chars = words.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}
fn leaf_palette(recipe: TextureRecipeId) -> Option<&'static str> {
    use TextureRecipeId::*;
    match recipe {
        WhiteOakLeaf => Some("white_oak"),
        DryWhiteOakLeaf => Some("dry_white_oak"),
        HazelLeaf => Some("hazel"),
        BlackthornLeaf => Some("blackthorn"),
        HawthornLeaf => Some("hawthorn"),
        BeechLeaf => Some("beech"),
        _ => None,
    }
}
fn groups(recipe: TextureRecipeId) -> Vec<&'static str> {
    let mut groups = Vec::new();
    for path in recipe.control_paths() {
        let group = path.group();
        if group != "leaves"
            && group != "leaf_colors"
            && group != "seed"
            && !groups.contains(&group)
        {
            groups.push(group);
        }
    }
    if recipe == TextureRecipeId::HewnOak {
        groups.sort_by_key(|group| usize::from(*group != "hewn_oak_grain"));
    }
    groups
}

fn control_label(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    if let Ok(index) = name.parse::<usize>() {
        if path.contains("/knots/") && path.contains("/center/") {
            return ["U position", "V position"][index.min(1)].into();
        }
        if path.contains("/knots/") && path.contains("/radii/") {
            return ["Across radius", "Along radius"][index.min(1)].into();
        }
        return format!("Value {}", index + 1);
    }
    label(name).replace(" metres", " (m)")
}
