//! Stroke material and physical dimensions share the schema's owning bounds.
use bevy_egui::egui;
use fabelgeist_armor::{
    Micrometers, Millimeters, TextileColor, TextileFoldProfile, TextileStrokeStyle,
};
struct StyleChoice {
    label: &'static str,
    style: TextileStrokeStyle,
}
pub(super) fn show(ui: &mut egui::Ui, style: &mut TextileStrokeStyle) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        for choice in choices() {
            let selected = std::mem::discriminant(style) == std::mem::discriminant(&choice.style);
            let response = ui.selectable_label(selected, choice.label);
            if response.clicked() && !selected {
                *style = choice.style;
                changed = true;
            }
        }
    });
    changed |= match style {
        TextileStrokeStyle::Cord { diameter, color } => {
            let mut changed = super::super::number(
                ui,
                &mut diameter.0,
                TextileStrokeStyle::THREAD_RANGE,
                "Thread diameter (micrometres)",
            );
            changed |= ui.color_edit_button_srgb(&mut color.0).changed();
            changed
        }
        TextileStrokeStyle::Stitches {
            width,
            length,
            gap,
            relief,
            color,
        } => stitches(ui, width, length, gap, relief, color),
        TextileStrokeStyle::Braid {
            width,
            pitch,
            relief,
            color,
        } => braid(ui, width, pitch, relief, color),
        TextileStrokeStyle::Pearls {
            diameter,
            spacing,
            color,
        } => pearls(ui, diameter, spacing, color),
        TextileStrokeStyle::Fold {
            width,
            relief,
            profile,
        } => fold(ui, width, relief, profile),
    };
    changed
}
fn choices() -> [StyleChoice; 5] {
    let gold = TextileColor([190, 145, 55]);
    [
        StyleChoice {
            label: "Cord",
            style: TextileStrokeStyle::Cord {
                diameter: Micrometers(500),
                color: gold,
            },
        },
        StyleChoice {
            label: "Stitches",
            style: TextileStrokeStyle::Stitches {
                width: Micrometers(300),
                length: Millimeters(3),
                gap: Millimeters(2),
                relief: Micrometers(150),
                color: gold,
            },
        },
        StyleChoice {
            label: "Braid",
            style: TextileStrokeStyle::Braid {
                width: Millimeters(3),
                pitch: Millimeters(4),
                relief: Micrometers(300),
                color: gold,
            },
        },
        StyleChoice {
            label: "Pearls",
            style: TextileStrokeStyle::Pearls {
                diameter: Millimeters(2),
                spacing: Millimeters(3),
                color: TextileColor([245, 239, 220]),
            },
        },
        StyleChoice {
            label: "Folded seam",
            style: TextileStrokeStyle::Fold {
                width: Millimeters(3),
                relief: Micrometers(500),
                profile: TextileFoldProfile::Rolled,
            },
        },
    ]
}
fn stitches(
    ui: &mut egui::Ui,
    width: &mut Micrometers,
    length: &mut Millimeters,
    gap: &mut Millimeters,
    relief: &mut Micrometers,
    color: &mut TextileColor,
) -> bool {
    let mut changed = super::super::number(
        ui,
        &mut width.0,
        TextileStrokeStyle::THREAD_RANGE,
        "Stitch width (micrometres)",
    );
    changed |= super::super::number(
        ui,
        &mut length.0,
        TextileStrokeStyle::PITCH_RANGE,
        "Stitch length (mm)",
    );
    changed |= super::super::number(
        ui,
        &mut gap.0,
        TextileStrokeStyle::PITCH_RANGE,
        "Stitch gap (mm)",
    );
    changed |= super::super::number(
        ui,
        &mut relief.0,
        TextileStrokeStyle::RELIEF_RANGE,
        "Raised height (micrometres)",
    );
    changed | ui.color_edit_button_srgb(&mut color.0).changed()
}
fn braid(
    ui: &mut egui::Ui,
    width: &mut Millimeters,
    pitch: &mut Millimeters,
    relief: &mut Micrometers,
    color: &mut TextileColor,
) -> bool {
    let mut changed = super::super::number(
        ui,
        &mut width.0,
        TextileStrokeStyle::WIDTH_RANGE,
        "Braid width (mm)",
    );
    changed |= super::super::number(
        ui,
        &mut pitch.0,
        TextileStrokeStyle::PITCH_RANGE,
        "Braid pitch (mm)",
    );
    changed |= super::super::number(
        ui,
        &mut relief.0,
        TextileStrokeStyle::RELIEF_RANGE,
        "Raised height (micrometres)",
    );
    changed | ui.color_edit_button_srgb(&mut color.0).changed()
}
fn pearls(
    ui: &mut egui::Ui,
    diameter: &mut Millimeters,
    spacing: &mut Millimeters,
    color: &mut TextileColor,
) -> bool {
    let mut changed = super::super::number(
        ui,
        &mut diameter.0,
        TextileStrokeStyle::PEARL_RANGE,
        "Pearl diameter (mm)",
    );
    let minimum = spacing.0.max(diameter.0);
    changed |= minimum != spacing.0;
    spacing.0 = minimum;
    changed |= super::super::number(
        ui,
        &mut spacing.0,
        diameter.0..=*TextileStrokeStyle::PITCH_RANGE.end(),
        "Pearl spacing (mm)",
    );
    changed | ui.color_edit_button_srgb(&mut color.0).changed()
}
fn fold(
    ui: &mut egui::Ui,
    width: &mut Millimeters,
    relief: &mut Micrometers,
    profile: &mut TextileFoldProfile,
) -> bool {
    let mut changed = super::super::number(
        ui,
        &mut width.0,
        TextileStrokeStyle::WIDTH_RANGE,
        "Seam width (mm)",
    );
    changed |= super::super::number(
        ui,
        &mut relief.0,
        TextileStrokeStyle::RELIEF_RANGE,
        "Raised height (micrometres)",
    );
    changed |= ui
        .selectable_value(profile, TextileFoldProfile::Rolled, "Rolled edge")
        .changed();
    changed
        | ui.selectable_value(profile, TextileFoldProfile::Felled, "Felled seam")
            .changed()
}
