//! Leaf presets and morphology interpolation feed the same editable shape document.
use super::*;
use adventuresim_procedural_textures::leaf::LeafShape;
#[derive(Clone)]
struct MorphSession {
    source: LeafShape,
    last: LeafShape,
    target: String,
    amount: f32,
}
impl MorphSession {
    fn new(shape: LeafShape) -> Self {
        Self {
            source: shape.clone(),
            last: shape,
            target: "beech".into(),
            amount: 0.0,
        }
    }
    fn accept_external(&mut self, shape: &LeafShape) {
        if *shape != self.last {
            self.source = shape.clone();
            self.last = shape.clone();
            self.amount = 0.0;
        }
    }
    fn blend(&mut self) -> LeafShape {
        let target = LeafShape::preset(&self.target).expect("catalogue target");
        self.last = self.source.interpolate(&target, self.amount);
        self.last.clone()
    }
}
pub(super) fn draw(
    ui: &mut Ui,
    recipe: TextureRecipeId,
    values: &mut Value,
    defaults: &Value,
    search: &str,
) -> bool {
    let Some(species) = recipe.leaf_species() else {
        return false;
    };
    let root = format!("/leaves/{}", species.parameter_key());
    let path = format!("{root}/shape");
    let Some(value) = values.pointer_mut(&path) else {
        return false;
    };
    let Ok(current) = serde_json::from_value::<LeafShape>(value.clone()) else {
        return false;
    };
    let id = ui.make_persistent_id(("leaf-morph", recipe.slug()));
    let mut morph = ui
        .ctx()
        .data_mut(|d| d.get_temp::<MorphSession>(id))
        .unwrap_or_else(|| MorphSession::new(current.clone()));
    morph.accept_external(&current);
    let mut changed = false;
    ui.strong("Shared leaf generator");
    ui.small("Presets set the same botanical controls. Morph changes structure; palettes stay independent.");
    let mut load = None;
    egui::ComboBox::from_id_salt("leaf-shape-preset")
        .selected_text("Load shape preset…")
        .show_ui(ui, |ui| {
            for name in LeafShape::preset_names() {
                if ui.selectable_label(false, super::label(name)).clicked() {
                    load = Some(name);
                }
            }
        });
    if let Some(name) = load {
        let shape = LeafShape::preset(name).unwrap();
        *value = serde_json::to_value(&shape).unwrap();
        morph = MorphSession::new(shape);
        changed = true;
    }
    let target_before = morph.target.clone();
    egui::ComboBox::from_id_salt("leaf-morph-target")
        .selected_text(format!("Morph to {}", super::label(&morph.target)))
        .show_ui(ui, |ui| {
            for name in LeafShape::preset_names() {
                ui.selectable_value(&mut morph.target, name.to_owned(), super::label(name));
            }
        });
    if target_before != morph.target {
        morph.source = morph.last.clone();
        morph.amount = 0.0;
    }
    if ui
        .add(egui::Slider::new(&mut morph.amount, 0.0..=1.0).text("Morph"))
        .changed()
    {
        *value = serde_json::to_value(morph.blend()).unwrap();
        changed = true;
    }
    ui.small("Structural constraints keep combinations valid; incompatible architectures can limit a morph.");
    ui.ctx().data_mut(|d| d.insert_temp(id, morph));
    changed |= controls(ui, &path, value, defaults, search);
    let relief = format!("{root}/relief");
    if let (Some(v), Some(d)) = (values.pointer_mut(&relief), defaults.pointer(&relief)) {
        egui::CollapsingHeader::new("Leaf relief").show(ui, |ui| {
            changed |= super::node(ui, &relief, v, d, search, recipe.control_paths());
        });
    }
    changed
}
fn controls(ui: &mut Ui, path: &str, value: &mut Value, defaults: &Value, search: &str) -> bool {
    let mut changed = false;
    for (name, fields) in GROUPS {
        let fields: Vec<_> = fields
            .iter()
            .filter(|field| search.is_empty() || field.replace('_', " ").contains(search))
            .collect();
        if fields.is_empty() {
            continue;
        }
        egui::CollapsingHeader::new(super::label(name))
            .default_open(*name == "profile")
            .open(if search.is_empty() { None } else { Some(true) })
            .show(ui, |ui| {
                for field in fields {
                    let child = format!("{path}/{field}");
                    if let (Some(v), Some(d)) = (value.get_mut(*field), defaults.pointer(&child)) {
                        changed |= super::scalar(ui, &child, v, d);
                    }
                }
            });
    }
    changed
}
const GROUPS: &[(&str, &[&str])] = &[
    (
        "profile",
        &["half_length", "half_width", "widest_at", "base_exponent"],
    ),
    (
        "shape",
        &["tip_exponent", "base_width", "tip_width", "asymmetry"],
    ),
    (
        "axis",
        &["bend", "petiole_length", "petiole_width", "midrib_width"],
    ),
    (
        "notches",
        &[
            "base_notch_depth",
            "base_notch_width",
            "tip_notch_depth",
            "tip_notch_width",
        ],
    ),
    (
        "lobes",
        &[
            "lobe_frequency",
            "lobe_depth",
            "lobe_roundness",
            "lobe_stagger",
        ],
    ),
    (
        "teeth",
        &[
            "tooth_frequency",
            "tooth_depth",
            "tooth_sharpness",
            "tooth_lean",
        ],
    ),
    (
        "veins",
        &[
            "secondary_count",
            "secondary_first",
            "secondary_last",
            "secondary_reach",
        ],
    ),
    (
        "vein_style",
        &[
            "secondary_sweep",
            "secondary_curve",
            "secondary_width",
            "secondary_tip_width",
        ],
    ),
    (
        "lobe_grade",
        &[
            "lobe_basal_scale",
            "lobe_middle_scale",
            "lobe_apical_scale",
            "lobe_progressive_sweep",
        ],
    ),
    (
        "margin_style",
        &[
            "secondary_tooth_ratio",
            "margin_hierarchy",
            "margin_asymmetry",
            "lobe_bristle",
        ],
    ),
    (
        "venation",
        &[
            "vein_alternation",
            "vein_gradation",
            "vein_branching",
            "vein_loops",
        ],
    ),
    (
        "topology",
        &["palmate", "compound_separation", "fan", "parallel_venation"],
    ),
    (
        "organs",
        &["organ_count", "organ_spread", "organ_length", "organ_width"],
    ),
    (
        "organ_style",
        &[
            "organ_attachment",
            "terminal_scale",
            "organ_alternation",
            "dichotomy",
        ],
    ),
    (
        "hierarchy",
        &["bipinnate", "pinna_count", "leaflet_count", "pinna_length"],
    ),
    (
        "landmarks",
        &[
            "basal_obliquity",
            "landmark_lobes",
            "landmark_side_bias",
            "apex_truncation",
        ],
    ),
    (
        "planar",
        &[
            "frond",
            "fan_segmentation",
            "peltate_depth",
            "longitudinal_veins",
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_edit_reanchors_morph_including_undo() {
        let oak = LeafShape::default();
        let mut session = MorphSession::new(oak.clone());
        session.amount = 0.5;
        let blend = session.blend();
        session.accept_external(&blend);
        assert_eq!(session.amount, 0.5);
        session.accept_external(&oak);
        assert_eq!(session.amount, 0.0);
        assert_eq!(session.source, oak);
    }
}
