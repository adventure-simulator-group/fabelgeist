//! Physical waist-down dressing slit; front-opening fitted cloth only.
use bevy_egui::egui;
use fabelgeist_armor::{
    BodiceOpening, ClothSlit, DressingOpeningSide, GownBodyFit, GownDesign, GownDressingSlit,
    Millimeters,
};
pub(super) fn show(ui: &mut egui::Ui, gown: &mut GownDesign) -> bool {
    let supported = matches!(gown.bodice.opening, BodiceOpening::Front { .. })
        && matches!(gown.body_fit, GownBodyFit::Fitted);
    let mut enabled = gown.dressing_slit.is_some();
    let mut changed = ui
        .add_enabled(
            supported || enabled,
            egui::Checkbox::new(&mut enabled, "Skirt dressing slit"),
        )
        .changed();
    if changed {
        gown.dressing_slit = enabled.then_some(GownDressingSlit {
            length: Millimeters(305),
            gap: Millimeters(2),
            side: DressingOpeningSide::Right,
            overlap: Millimeters(13),
        });
    }
    if let Some(slit) = &mut gown.dressing_slit {
        changed |= super::number(
            ui,
            &mut slit.length.0,
            GownDressingSlit::LENGTH_RANGE,
            "Slit below waist (mm)",
        );
        changed |= super::number(
            ui,
            &mut slit.gap.0,
            ClothSlit::GAP_RANGE,
            "Rest separation (mm)",
        );
        changed |= super::number(
            ui,
            &mut slit.overlap.0,
            GownDressingSlit::OVERLAP_RANGE,
            "Finished overlap (mm)",
        );
        changed |= ui
            .selectable_value(&mut slit.side, DressingOpeningSide::Right, "Wearer's right")
            .changed();
        changed |= ui
            .selectable_value(&mut slit.side, DressingOpeningSide::Left, "Wearer's left")
            .changed();
        ui.label("The slit must reach the hanging cloth below the waist transition.");
    }
    changed
}
