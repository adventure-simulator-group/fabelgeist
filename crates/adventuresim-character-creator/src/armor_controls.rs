//! Editable construction controls for selected catalog armor.
use super::{EquipmentCatalog, Studio};
use adventuresim_armor_model::{GarmentArmorDesign, GarmentArmorKind, GarmentPlateShape};
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
                .design(&item.id)
                .map(|design| (item.id.clone(), item.display_name.clone(), design))
        })
        .collect::<Vec<_>>();
    for (id, label, mut design) in selected {
        let changed = ui
            .collapsing(format!("{label} shape"), |ui| match &mut design {
                ParametricDesign::Limb(d) => limb::show(ui, d),
                ParametricDesign::Helmet(d) => helmet::show(ui, d),
                ParametricDesign::Garment(d) => garment(ui, d),
                ParametricDesign::Underlayer(d) => underlayer(ui, d),
            })
            .body_returned
            .unwrap_or(false);
        if changed {
            catalog.1.insert(id, design);
            studio.dirty = true;
        }
    }
    for (label, path) in [
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
            Ok(()) => "Saved catalog, vambrace and breastplate designs".into(),
            Err(error) => format!("Could not save armor designs: {error}"),
        };
    }
}

fn save(catalog: &EquipmentCatalog, studio: &Studio) -> anyhow::Result<()> {
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

fn garment(ui: &mut egui::Ui, d: &mut GarmentArmorDesign) -> bool {
    let mut changed = false;
    for (value, range, label) in [
        (&mut d.length.0, 500..=1300, "Length"),
        (&mut d.clearance.0, 1..=40, "Padding clearance (mm)"),
        (&mut d.wall_thickness.0, 1..=16, "Thickness (mm)"),
    ] {
        changed |= number(ui, value, range, label);
    }
    if matches!(
        d.kind,
        GarmentArmorKind::Brigandine
            | GarmentArmorKind::JackOfPlates
            | GarmentArmorKind::Fauld
            | GarmentArmorKind::Tassets
            | GarmentArmorKind::Gorget
            | GarmentArmorKind::MailSkirt
            | GarmentArmorKind::PaddedSkirt
    ) {
        changed |= number(ui, &mut d.flare.0, 0..=500, "Hem flare");
    }
    if matches!(
        d.kind,
        GarmentArmorKind::ArmingDoublet
            | GarmentArmorKind::Brigandine
            | GarmentArmorKind::JackOfPlates
            | GarmentArmorKind::MailShirt
    ) {
        changed |= number(ui, &mut d.waist.0, 800..=1100, "Waist width");
    }
    match &mut d.plate_shape {
        GarmentPlateShape::None => return changed,
        GarmentPlateShape::Fauld {
            front_arch,
            waist_rise,
        } => {
            changed |= number(ui, &mut waist_rise.0, 0..=80, "Waist rise (mm)");
            changed |= number(ui, &mut front_arch.0, 0..=350, "Front arch")
        }
        GarmentPlateShape::Tassets {
            inner_cutaway,
            hem_point,
            width,
            gap,
            hem_roundness,
        } => {
            changed |= number(ui, &mut inner_cutaway.0, 0..=400, "Inner cutaway");
            changed |= number(ui, &mut hem_point.0, 0..=200, "Hem point");
            changed |= number(ui, &mut width.0, 700..=1150, "Plate width");
            changed |= number(ui, &mut gap.0, 100..=400, "Separation");
            changed |= number(ui, &mut hem_roundness.0, 0..=300, "Hem roundness");
        }
        GarmentPlateShape::Gorget {
            neck_clearance,
            collar_slope,
            collar_height,
            hem_flatness,
            rear_hem_flatness,
            rear_sweep,
            front_depth,
            back_depth,
            front_width,
            back_width,
        } => {
            changed |= number(ui, &mut neck_clearance.0, 2..=15, "Neck clearance (mm)");
            changed |= number(ui, &mut collar_slope.0, 0..=1000, "Collar slope");
            changed |= number(ui, &mut collar_height.0, 500..=2000, "Collar height");
            changed |= number(ui, &mut hem_flatness.0, 0..=1000, "Front hem flatness");
            changed |= number(ui, &mut rear_hem_flatness.0, 0..=1000, "Rear hem flatness");
            changed |= number(ui, &mut rear_sweep.0, 0..=1000, "Rearward shoulder sweep");
            changed |= number(ui, &mut front_depth.0, 600..=1500, "Front depth");
            changed |= number(ui, &mut back_depth.0, 600..=1500, "Rear depth");
            changed |= number(ui, &mut front_width.0, 500..=1300, "Front bib width");
            changed |= number(ui, &mut back_width.0, 500..=1300, "Rear bib width");
        }
    }
    changed |= ui
        .add(egui::Slider::new(&mut d.lame_count, 1..=8).text("Lames"))
        .changed();
    changed | crate::fluting_controls::show(ui, &mut d.fluting)
}
