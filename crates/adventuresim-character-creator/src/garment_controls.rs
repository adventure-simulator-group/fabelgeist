//! Waist and neck garment controls, including alternate tasset constructions.
use super::armor_controls::number;
use adventuresim_armor_model::{
    GarmentArmorDesign, GarmentArmorKind, GarmentPlateShape, WrappedTassetDesign,
};
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, design: &mut GarmentArmorDesign) -> bool {
    let original = design.clone();
    tasset_construction(ui, design);
    let minimum_length = if design.kind == GarmentArmorKind::Fauld {
        100
    } else {
        500
    };
    for (value, range, label) in [
        (&mut design.length.0, minimum_length..=1300, "Length"),
        (&mut design.clearance.0, 1..=40, "Padding clearance (mm)"),
        (&mut design.wall_thickness.0, 1..=16, "Thickness (mm)"),
    ] {
        number(ui, value, range, label);
    }
    if matches!(
        design.kind,
        GarmentArmorKind::Brigandine
            | GarmentArmorKind::JackOfPlates
            | GarmentArmorKind::Fauld
            | GarmentArmorKind::Tassets
            | GarmentArmorKind::Gorget
            | GarmentArmorKind::MailSkirt
            | GarmentArmorKind::PaddedSkirt
    ) {
        number(ui, &mut design.flare.0, 0..=500, "Hem flare");
    }
    if matches!(
        design.kind,
        GarmentArmorKind::ArmingDoublet
            | GarmentArmorKind::Brigandine
            | GarmentArmorKind::JackOfPlates
            | GarmentArmorKind::MailShirt
    ) {
        number(ui, &mut design.waist.0, 800..=1100, "Waist width");
    }
    if design.plate_shape != GarmentPlateShape::None {
        let maximum_lames = if design.kind == GarmentArmorKind::Tassets {
            12
        } else {
            8
        };
        ui.add(egui::Slider::new(&mut design.lame_count, 1..=maximum_lames).text("Lames"));
        plate_shape(ui, design);
        crate::fluting_controls::show(ui, &mut design.fluting);
    }
    *design != original
}

fn tasset_construction(ui: &mut egui::Ui, design: &mut GarmentArmorDesign) {
    if design.kind != GarmentArmorKind::Tassets {
        return;
    }
    let original = matches!(design.plate_shape, GarmentPlateShape::WrappedTassets(_));
    let mut wrapped = original;
    ui.horizontal(|ui| {
        ui.label("Tasset construction");
        ui.selectable_value(&mut wrapped, false, "Front panels");
        ui.selectable_value(&mut wrapped, true, "Wrapping thigh plates");
    });
    if wrapped != original {
        design.plate_shape = if wrapped {
            GarmentPlateShape::WrappedTassets(WrappedTassetDesign::default())
        } else {
            GarmentPlateShape::for_kind(GarmentArmorKind::Tassets)
        };
    }
}

fn plate_shape(ui: &mut egui::Ui, design: &mut GarmentArmorDesign) {
    match &mut design.plate_shape {
        GarmentPlateShape::None => {}
        GarmentPlateShape::WrappedTassets(shape) => {
            super::wrapped_tasset_controls::show(ui, shape, design.lame_count);
        }
        GarmentPlateShape::Fauld {
            front_arch,
            front_arch_width,
            waist_rise,
            chevron_slope,
        } => {
            number(ui, &mut waist_rise.0, 0..=120, "Waist rise (mm)");
            number(ui, &mut chevron_slope.0, 0..=500, "Chevron slope");
            let arch_limit = 1200.min(design.length.0.saturating_sub(1));
            front_arch.0 = front_arch.0.min(arch_limit);
            number(ui, &mut front_arch.0, 0..=arch_limit, "Front arch");
            number(ui, &mut front_arch_width.0, 250..=800, "Front arch width");
        }
        GarmentPlateShape::Tassets {
            inner_cutaway,
            hem_point,
            width,
            gap,
            hem_roundness,
        } => {
            number(ui, &mut inner_cutaway.0, 0..=400, "Inner cutaway");
            number(ui, &mut hem_point.0, 0..=200, "Hem point");
            number(ui, &mut width.0, 700..=1150, "Plate width");
            number(ui, &mut gap.0, 100..=400, "Separation");
            number(ui, &mut hem_roundness.0, 0..=300, "Hem roundness");
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
            number(ui, &mut neck_clearance.0, 2..=15, "Neck clearance (mm)");
            number(ui, &mut collar_slope.0, 0..=1000, "Collar slope");
            number(ui, &mut collar_height.0, 500..=2000, "Collar height");
            number(ui, &mut hem_flatness.0, 0..=1000, "Front hem flatness");
            number(ui, &mut rear_hem_flatness.0, 0..=1000, "Rear hem flatness");
            number(ui, &mut rear_sweep.0, 0..=1000, "Rearward shoulder sweep");
            number(ui, &mut front_depth.0, 600..=1500, "Front depth");
            number(ui, &mut back_depth.0, 600..=1500, "Rear depth");
            number(ui, &mut front_width.0, 500..=1300, "Front bib width");
            number(ui, &mut back_width.0, 500..=1300, "Rear bib width");
        }
    }
}
