//! Shared controls for ordered closures on a physical cloth opening.
use bevy_egui::egui;
use fabelgeist_armor::{
    ClothClosures, ClothFastener, Micrometers, Millimeters, Permille, TextileAppearance,
    TextileColor, TextileFinish,
};

pub(super) fn show(
    ui: &mut egui::Ui,
    closures: &mut Option<ClothClosures>,
    initial: ClothClosures,
) -> bool {
    let mut enabled = closures.is_some();
    let response = ui.checkbox(&mut enabled, "Opening fasteners");
    let mut changed = response.changed();
    if changed {
        *closures = enabled.then_some(initial);
    }
    let Some(closures) = closures else {
        return changed;
    };
    let mut count = closures.positions.len();
    let response =
        ui.add(egui::Slider::new(&mut count, ClothClosures::COUNT_RANGE).text("Fastener count"));
    if response.changed() {
        changed = true;
        closures.positions = (1..=count)
            .map(|i| Permille((i * usize::from(Permille::ONE.0) / (count + 1)) as u16))
            .collect();
    }
    for i in 0..closures.positions.len() {
        let minimum = if i == 0 {
            *ClothClosures::POSITION_RANGE.start()
        } else {
            closures.positions[i - 1].0 + 1
        };
        let maximum = if i + 1 == closures.positions.len() {
            *ClothClosures::POSITION_RANGE.end()
        } else {
            closures.positions[i + 1].0 - 1
        };
        changed |= super::number(
            ui,
            &mut closures.positions[i].0,
            minimum..=maximum,
            &format!("Fastener {} position", i + 1),
        );
    }
    changed |= fastener_controls(ui, &mut closures.fastener);
    changed
}

fn fastener_controls(ui: &mut egui::Ui, fastener: &mut ClothFastener) -> bool {
    let mut changed = kind_controls(ui, fastener);
    match fastener {
        ClothFastener::ButtonLoop {
            diameter,
            cord,
            fabric,
        } => {
            changed |= super::number(
                ui,
                &mut diameter.0,
                ClothFastener::BUTTON_DIAMETER_RANGE,
                "Button diameter (mm)",
            );
            let range = ClothFastener::cord_range(*diameter);
            let limited = cord.0.clamp(*range.start(), *range.end());
            changed |= limited != cord.0;
            cord.0 = limited;
            changed |= super::number(ui, &mut cord.0, range, "Loop cord diameter (micrometres)");
            changed |= super::textile::show(ui, fabric);
        }
        ClothFastener::HookEye {
            length,
            width,
            wire,
            color,
            roughness,
        } => {
            changed |= super::number(
                ui,
                &mut length.0,
                ClothFastener::HOOK_LENGTH_RANGE,
                "Hook reach (mm)",
            );
            let limited = width.0.min(length.0);
            changed |= limited != width.0;
            width.0 = limited;
            changed |= super::number(
                ui,
                &mut width.0,
                *ClothFastener::HOOK_WIDTH_RANGE.start()
                    ..=length.0.min(*ClothFastener::HOOK_WIDTH_RANGE.end()),
                "Eye width (mm)",
            );
            let range = ClothFastener::wire_range(*width);
            let limited = wire.0.clamp(*range.start(), *range.end());
            changed |= limited != wire.0;
            wire.0 = limited;
            changed |= super::number(ui, &mut wire.0, range, "Wire diameter (micrometres)");
            changed |= super::number(
                ui,
                &mut roughness.0,
                ClothFastener::ROUGHNESS_RANGE,
                "Metal roughness",
            );
            changed |= ui.color_edit_button_srgb(&mut color.0).changed();
        }
    }
    changed
}

fn kind_controls(ui: &mut egui::Ui, fastener: &mut ClothFastener) -> bool {
    let button = matches!(fastener, ClothFastener::ButtonLoop { .. });
    let mut changed = false;
    ui.horizontal(|ui| {
        let button_response = ui.selectable_label(button, "Button and finger loop");
        if button_response.clicked() && !button {
            *fastener = ClothFastener::ButtonLoop {
                diameter: Millimeters(5),
                cord: Micrometers(500),
                fabric: TextileAppearance::solid(
                    TextileColor([242, 240, 230]),
                    TextileFinish::Silk,
                ),
            };
            changed = true;
        }
        let hook_response = ui.selectable_label(!button, "Hook and eye");
        if hook_response.clicked() && button {
            *fastener = ClothFastener::HookEye {
                length: Millimeters(8),
                width: Millimeters(4),
                wire: Micrometers(500),
                color: TextileColor([185, 143, 58]),
                roughness: Permille(300),
            };
            changed = true;
        }
    });
    changed
}
