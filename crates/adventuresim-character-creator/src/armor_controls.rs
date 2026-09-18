//! Editable construction controls for selected catalog armor.
use super::{EquipmentCatalog, Studio};
use adventuresim_character_creator::armor_recipes::ParametricDesign;
use bevy_egui::egui;
use std::ops::RangeInclusive;
#[path = "helmet_controls.rs"]
mod helmet;
#[path = "limb_controls.rs"]
mod limb;

pub(super) fn number(
    ui: &mut egui::Ui,
    value: &mut u16,
    range: RangeInclusive<u16>,
    label: &str,
) -> bool {
    ui.add(egui::Slider::new(value, range).text(label))
        .changed()
}

pub(super) fn show(ui: &mut egui::Ui, catalog: &mut EquipmentCatalog, studio: &mut Studio) {
    let selected = catalog
        .0
        .iter()
        .filter(|item| studio.recipe.clothing.iter().any(|c| c.item_id == item.id))
        .filter_map(|item| {
            catalog
                .default_design(&item.id)
                .map(|design| (item.id.clone(), item.display_name.clone(), design))
        })
        .collect::<Vec<_>>();
    for (id, label, mut design) in selected {
        let changed = ui
            .collapsing(format!("{label} default shape"), |ui| {
                controls(ui, &mut design)
            })
            .body_returned
            .unwrap_or(false);
        if changed {
            catalog.1.defaults.insert(id.clone(), design.clone());
            studio.dirty = true;
        }
        placement_controls(ui, catalog, studio, &id, &label, &design);
    }
    super::fastener_controls::show(ui, catalog, studio);
    for (label, path) in [
        ("Fastenings", &mut studio.fastener_designs_path),
        ("Catalog armor", &mut studio.armor_designs_path),
        ("Vambrace", &mut studio.bracer_design_path),
        ("Breastplate", &mut studio.breastplate_design_path),
    ] {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.text_edit_singleline(path);
        });
    }
    if ui.button("Save all armor designs").clicked() {
        studio.status = match save(catalog, studio) {
            Ok(()) => "Saved armor shapes and fastenings".into(),
            Err(error) => format!("Could not save armor designs: {error}"),
        };
    }
}

fn controls(ui: &mut egui::Ui, design: &mut ParametricDesign) -> bool {
    match design {
        ParametricDesign::Limb(d) => limb::show(ui, d),
        ParametricDesign::PuffAndSlash(d) => puff_and_slash(ui, d),
        ParametricDesign::TrunkHose(d) => trunk_hose(ui, d),
        ParametricDesign::Helmet(d) => helmet::show(ui, d),
        ParametricDesign::Garment(d) => super::garment_controls::show(ui, d),
        ParametricDesign::Underlayer(d) => underlayer(ui, d),
        ParametricDesign::WaistAssembly(d) => {
            let fauld = ui
                .collapsing("Fauld", |ui| {
                    super::garment_controls::show(ui, &mut d.fauld)
                })
                .body_returned
                .unwrap_or(false);
            let tassets = ui
                .collapsing("Tassets", |ui| {
                    super::garment_controls::show(ui, &mut d.tassets)
                })
                .body_returned
                .unwrap_or(false);
            fauld || tassets
        }
    }
}

fn trunk_hose(ui: &mut egui::Ui, design: &mut adventuresim_armor_model::TrunkHoseDesign) -> bool {
    let mut changed = number(ui, &mut design.clearance.0, 1..=10, "Body clearance (mm)");
    changed |= number(ui, &mut design.thickness.0, 1..=8, "Cloth thickness (mm)");
    changed |= number(ui, &mut design.length.0, 900..=1200, "Upper-leg reach");
    changed |= ui
        .add(
            egui::Slider::new(&mut design.panel_count, 4..=16)
                .step_by(2.0)
                .text("Vertical panes"),
        )
        .changed();
    ui.horizontal(|ui| {
        ui.label("Primary fabric");
        changed |= ui
            .color_edit_button_srgb(&mut design.primary_color.0)
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label("Secondary fabric");
        changed |= ui
            .color_edit_button_srgb(&mut design.secondary_color.0)
            .changed();
    });
    changed
}

fn puff_and_slash(
    ui: &mut egui::Ui,
    design: &mut adventuresim_armor_model::PuffAndSlashDesign,
) -> bool {
    let mut changed = ui
        .add(egui::Slider::new(&mut design.puff_count, 1..=8).text("Puff courses"))
        .changed();
    changed |= number(
        ui,
        &mut design.puff_fullness.0,
        5..=90,
        "Puff fullness (mm)",
    );
    changed |= number(
        ui,
        &mut design.puff_roundness.0,
        400..=2500,
        "Puff roundness",
    );
    changed |= ui
        .add(egui::Slider::new(&mut design.slash_count, 3..=16).text("Slashes per course"))
        .changed();
    for (value, range, label) in [
        (&mut design.distal_fullness.0, 250..=1500, "Distal fullness"),
        (&mut design.slash_width.0, 50..=650, "Slash width"),
        (&mut design.slash_length.0, 300..=900, "Slash length"),
        (
            &mut design.constriction_width.0,
            40..=350,
            "Constricted band width",
        ),
        (&mut design.length.0, 350..=1000, "Limb coverage"),
        (
            &mut design.proximal_position.0,
            0..=1000,
            "Proximal placement",
        ),
        (&mut design.rotation.0, 0..=1000, "Pattern rotation"),
        (&mut design.clearance.0, 1..=15, "Body clearance (mm)"),
        (&mut design.thickness.0, 1..=6, "Cloth thickness (mm)"),
    ] {
        changed |= number(ui, value, range, label);
    }
    ui.horizontal(|ui| {
        ui.label("Outer fabric");
        changed |= ui
            .color_edit_button_srgb(&mut design.outer_color.0)
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label("Undercloth");
        changed |= ui
            .color_edit_button_srgb(&mut design.undercloth_color.0)
            .changed();
    });
    changed
}

fn placement_controls(
    ui: &mut egui::Ui,
    catalog: &mut EquipmentCatalog,
    studio: &mut Studio,
    id: &str,
    label: &str,
    default: &ParametricDesign,
) {
    use adventuresim_character_creator::armor_design_input::ArmorPlacement;
    let placements = studio
        .recipe
        .clothing
        .iter()
        .filter(|c| c.item_id == id)
        .filter_map(|c| ArmorPlacement::parse(&c.placement_id))
        .collect::<std::collections::BTreeSet<_>>();
    for placement in placements {
        let mut specific = catalog
            .1
            .placements
            .get(id)
            .and_then(|p| p.get(&placement))
            .is_some();
        let toggle = ui.checkbox(
            &mut specific,
            format!("Customize {label} {}", placement.as_str()),
        );
        if toggle.changed() {
            if specific {
                catalog
                    .1
                    .placements
                    .entry(id.into())
                    .or_default()
                    .insert(placement, default.clone());
            } else if let Some(choices) = catalog.1.placements.get_mut(id) {
                choices.remove(&placement);
                if choices.is_empty() {
                    catalog.1.placements.remove(id);
                }
            }
            studio.dirty = true;
        }
        if let Some(design) = catalog
            .1
            .placements
            .get_mut(id)
            .and_then(|p| p.get_mut(&placement))
        {
            studio.dirty |= ui
                .collapsing(format!("{label} {} shape", placement.as_str()), |ui| {
                    controls(ui, design)
                })
                .body_returned
                .unwrap_or(false);
        }
    }
}

fn save(catalog: &EquipmentCatalog, studio: &Studio) -> anyhow::Result<()> {
    for recipe in catalog.2.values() {
        recipe.validate()?;
    }
    std::fs::write(
        &studio.fastener_designs_path,
        serde_json::to_vec_pretty(&catalog.2)?,
    )?;
    adventuresim_character_creator::armor_design_output::DesignPaths {
        catalog: std::path::Path::new(&studio.armor_designs_path),
        bracer: std::path::Path::new(&studio.bracer_design_path),
        breastplate: std::path::Path::new(&studio.breastplate_design_path),
    }
    .save(
        &catalog.1,
        &studio.bracer_design,
        &studio.breastplate_design,
    )
}

fn underlayer(
    ui: &mut egui::Ui,
    d: &mut adventuresim_character_creator::underlayer::UnderlayerDesign,
) -> bool {
    use adventuresim_character_creator::underlayer::{self, UnderlayerKind};
    let mut changed = number(
        ui,
        &mut d.clearance.0,
        underlayer::CLEARANCE_MM,
        "Body clearance (mm)",
    );
    changed |= number(
        ui,
        &mut d.thickness.0,
        underlayer::THICKNESS_MM,
        "Material thickness (mm)",
    );
    if !d.kind.is_mail() {
        ui.horizontal(|ui| {
            ui.label("Fabric");
            changed |= ui.color_edit_button_srgb(&mut d.color.0).changed();
        });
    }
    if d.kind != UnderlayerKind::MailVoiders {
        let range = d.length_range();
        changed |= number(ui, &mut d.length.0, range, "Garment length");
        if d.kind == UnderlayerKind::ArmingDoublet {
            changed |= number(
                ui,
                &mut d.sleeve_length.0,
                underlayer::SLEEVE_LENGTH,
                "Sleeve length",
            );
        }
    }
    if matches!(
        d.kind,
        UnderlayerKind::MailVoiders | UnderlayerKind::MailKneeVoider | UnderlayerKind::MailStandard
    ) {
        changed |= number(
            ui,
            &mut d.patch_width.0,
            underlayer::PATCH_WIDTH_MM,
            "Mail patch width (mm)",
        );
    }
    changed
}
