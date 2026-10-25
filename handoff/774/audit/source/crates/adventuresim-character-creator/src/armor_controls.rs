//! Editable construction controls for catalog armor designs.
use adventuresim_character_creator::{armor_recipes::ParametricDesign, item_design::ItemDesign};
use bevy_egui::egui;
use std::ops::RangeInclusive;
#[path = "helmet_controls.rs"]
mod helmet;
#[path = "limb_controls.rs"]
mod limb;

/// The mail coif's shape controls, for the draped chainmail coif.
pub(super) fn coif(ui: &mut egui::Ui, design: &mut fabelgeist_armor::CoifDesign) -> bool {
    let mut helmet = fabelgeist_armor::HelmetDesign::MailCoif(*design);
    let changed = helmet::show(ui, &mut helmet);
    if let fabelgeist_armor::HelmetDesign::MailCoif(edited) = helmet {
        *design = edited;
    }
    changed
}

pub(super) fn number(
    ui: &mut egui::Ui,
    value: &mut u16,
    range: RangeInclusive<u16>,
    label: &str,
) -> bool {
    ui.add(egui::Slider::new(value, range).text(label))
        .changed()
}

/// Shape controls for one catalog item's design. Returns whether it changed.
pub(super) fn design(ui: &mut egui::Ui, design: &mut ItemDesign) -> bool {
    match design {
        ItemDesign::Recipe(ParametricDesign::Limb(d)) => limb::show(ui, d),
        ItemDesign::Recipe(ParametricDesign::Helmet(d)) => helmet::show(ui, d),
        ItemDesign::Recipe(ParametricDesign::PuffAndSlash(d)) => puff_and_slash(ui, d),
        ItemDesign::Recipe(ParametricDesign::TrunkHose(d)) => trunk_hose(ui, d),
        ItemDesign::Recipe(ParametricDesign::Garment(d)) => {
            super::garment_armor_controls::show(ui, d)
        }
        ItemDesign::Recipe(ParametricDesign::WaistAssembly(d)) => {
            let fauld = ui
                .collapsing("Fauld", |ui| {
                    super::garment_armor_controls::show(ui, &mut d.fauld)
                })
                .body_returned
                .unwrap_or(false);
            let tassets = ui
                .collapsing("Tassets", |ui| {
                    super::garment_armor_controls::show(ui, &mut d.tassets)
                })
                .body_returned
                .unwrap_or(false);
            fauld || tassets
        }
        ItemDesign::Recipe(ParametricDesign::Underlayer(d)) => underlayer(ui, d),
        ItemDesign::Vambrace(d) => super::equipment_controls::bracer(ui, d),
        ItemDesign::Breastplate(d) => super::equipment_controls::breastplate(ui, d),
    }
}

fn trunk_hose(ui: &mut egui::Ui, design: &mut fabelgeist_armor::TrunkHoseDesign) -> bool {
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

fn puff_and_slash(ui: &mut egui::Ui, design: &mut fabelgeist_armor::PuffAndSlashDesign) -> bool {
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
