//! Fabric choice and chainmail appearance controls for one draped garment.
use super::*;
use adventuresim_character_creator::garment_material::MailWeave;
use std::ops::RangeInclusive;

const MM_PER_M: f32 = 1000.0;
const SRGB_BYTE_MAX: f32 = 255.0;

pub(super) fn show(
    ui: &mut egui::Ui,
    id: adventuresim_character_creator::inventory::InventoryItemId,
    selection: &mut GarmentSelection,
) {
    let before = selection.fabric;
    egui::ComboBox::from_id_salt(("fabric_preset", id))
        .selected_text(selection.fabric.label())
        .show_ui(ui, |ui| {
            for fabric in FabricPreset::ALL {
                ui.selectable_value(&mut selection.fabric, fabric, fabric.label());
            }
        });
    if selection.fabric != before {
        // Settling drag belongs to the fabric; other stage settings are kept.
        selection.drape.settling.damping = selection.fabric.fabric().damping;
    }
    if selection.fabric == FabricPreset::Chainmail {
        mail_weave(ui, &mut selection.mail);
    }
}

fn mail_weave(ui: &mut egui::Ui, weave: &mut MailWeave) {
    ui.label("Chainmail rings");
    ui.small("Appearance only: changes apply without re-draping.");
    millimetres(
        ui,
        "Ring outer diameter (mm)",
        &mut weave.ring_outer_diameter_m,
        MailWeave::RING_OUTER_DIAMETER_M,
    );
    // Dependent limits follow the ring, so every edit stays a valid weave.
    let wire = weave.wire_diameter_range_m();
    weave.wire_diameter_m = weave.wire_diameter_m.clamp(*wire.start(), *wire.end());
    millimetres(ui, "Wire diameter (mm)", &mut weave.wire_diameter_m, wire);
    let rows = weave.row_pitch_range_m();
    weave.row_pitch_m = weave.row_pitch_m.clamp(*rows.start(), *rows.end());
    millimetres(ui, "Row spacing (mm)", &mut weave.row_pitch_m, rows);
    ui.add(
        egui::Slider::new(&mut weave.ring_tilt_degrees, MailWeave::RING_TILT_DEGREES)
            .text("Ring tilt (°)"),
    );
    ui.add(egui::Slider::new(&mut weave.roughness, MailWeave::ROUGHNESS).text("Steel roughness"));
    ui.horizontal(|ui| {
        let mut srgb = weave
            .steel_color_srgb
            .map(|channel| (channel * SRGB_BYTE_MAX).round() as u8);
        if ui.color_edit_button_srgb(&mut srgb).changed() {
            weave.steel_color_srgb = srgb.map(|channel| channel as f32 / SRGB_BYTE_MAX);
        }
        ui.label("Steel color");
    });
    ui.small(format!("{:.0} rings/m²", weave.rings_per_m2()));
}

fn millimetres(ui: &mut egui::Ui, label: &str, metres: &mut f32, range: RangeInclusive<f32>) {
    let mut value = *metres * MM_PER_M;
    let (min, max) = (*range.start(), *range.end());
    if ui
        .add(egui::Slider::new(&mut value, min * MM_PER_M..=max * MM_PER_M).text(label))
        .changed()
    {
        // The mm round trip can round past a limit, e.g. 0.5 mm -> 0.00049999 m.
        *metres = (value / MM_PER_M).clamp(min, max);
    }
}
