//! Author the independent folded main and front waist support bands.
use bevy_egui::egui;
use fabelgeist_armor::{
    BodiceOpening, ClothTape, GownBodyFit, GownDesign, GownWaistSupport, Micrometers, Millimeters,
    TextileAppearance, TextileColor, TextileFinish,
};
pub(super) fn show(ui: &mut egui::Ui, gown: &mut GownDesign) -> bool {
    let supported = matches!(gown.bodice.opening, BodiceOpening::Front { .. })
        && matches!(gown.body_fit, GownBodyFit::Fitted);
    let mut enabled = gown.waist_support.is_some();
    let response = ui.add_enabled(
        supported || enabled,
        egui::Checkbox::new(&mut enabled, "Folded waist support bands"),
    );
    let mut changed = response.changed();
    if changed {
        gown.waist_support = enabled.then(|| GownWaistSupport {
            main: ClothTape {
                width: Millimeters(32),
                thickness: Micrometers(500),
                fabric: TextileAppearance::solid(
                    TextileColor([194, 184, 155]),
                    TextileFinish::Linen,
                ),
            },
            front_width: Millimeters(190),
            front_height: Millimeters(25),
            fasteners: None,
        });
    }
    if !supported {
        ui.label("Separate front support requires a fitted, front-opening bodice.");
    }
    let fastening_supported = gown.dressing_slit.as_ref().is_some_and(|s| s.overlap.0 > 0);
    let metal_ring = gown
        .lacing
        .as_ref()
        .is_some_and(|l| matches!(l.eyelet, fabelgeist_armor::LacingEyelet::Metal { .. }));
    if let Some(support) = &mut gown.waist_support {
        changed |= super::number(
            ui,
            &mut support.main.width.0,
            ClothTape::WIDTH_RANGE,
            "Main band height (mm)",
        );
        changed |= super::number(
            ui,
            &mut support.front_width.0,
            GownWaistSupport::FRONT_WIDTH_RANGE,
            "Front band span (mm)",
        );
        changed |= super::number(
            ui,
            &mut support.front_height.0,
            ClothTape::WIDTH_RANGE,
            "Front band height (mm)",
        );
        changed |= super::number(
            ui,
            &mut support.main.thickness.0,
            ClothTape::THICKNESS_RANGE,
            "Linen gauge before folding (micrometres)",
        );
        changed |= super::textile::show(ui, &mut support.main.fabric);
        changed |= super::waist_fasteners::show(
            ui,
            &mut support.fasteners,
            fastening_supported,
            metal_ring,
        );
    }
    changed
}
