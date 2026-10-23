//! Hook closures on the finished inner waist bands.
use bevy_egui::egui;
use fabelgeist_armor::{
    ClothClosures, ClothFastener, Micrometers, Millimeters, Permille, TextileColor,
    WaistBandFasteners,
};
pub(super) fn show(
    ui: &mut egui::Ui,
    design: &mut Option<WaistBandFasteners>,
    supported: bool,
    metal_ring: bool,
) -> bool {
    let mut enabled = design.is_some();
    let mut changed = ui
        .add_enabled(
            supported || enabled,
            egui::Checkbox::new(&mut enabled, "Waist band hooks"),
        )
        .changed();
    if changed {
        *design = enabled.then(|| WaistBandFasteners {
            closures: ClothClosures {
                positions: vec![Permille(250), Permille(650)],
                fastener: hook(6),
            },
            inset: Millimeters(6),
            return_hook: None,
        });
    }
    if let Some(design) = design {
        changed |= super::cloth_closures::positions(ui, &mut design.closures);
        changed |= super::cloth_closures::parameters(ui, &mut design.closures.fastener);
        changed |= super::number(
            ui,
            &mut design.inset.0,
            WaistBandFasteners::INSET_RANGE,
            "Eye origin from closing edge (mm)",
        );
        let mut enabled = design.return_hook.is_some();
        let response = ui.add_enabled(
            metal_ring || enabled,
            egui::Checkbox::new(&mut enabled, "Return hook into final bodice ring"),
        );
        if response.changed() {
            changed = true;
            design.return_hook = enabled.then(|| hook(12));
        }
        if let Some(hook) = &mut design.return_hook {
            changed |= super::cloth_closures::parameters(ui, hook);
        }
    }
    changed
}
fn hook(length: u16) -> ClothFastener {
    ClothFastener::HookEye {
        length: Millimeters(length),
        width: Millimeters(4),
        wire: Micrometers(500),
        color: TextileColor([185, 143, 58]),
        roughness: Permille(350),
    }
}
