//! Burgonet plates and an optional separate face defense.
use adventuresim_armor_model::{BuffeCourses, BuffeDesign, BurgonetDesign};
use bevy_egui::egui;

pub(super) fn burgonet(ui: &mut egui::Ui, design: &mut BurgonetDesign) -> bool {
    ui.label("Cheek fluting");
    let mut changed = crate::fluting_controls::show(ui, &mut design.cheek_fluting);
    for (value, range, label) in [
        (&mut design.nape_depth.0, 650..=1400, "Nape depth"),
        (&mut design.nape_taper.0, 550..=1000, "Nape taper"),
        (&mut design.chin_tab.0, 0..=35, "Chin tab (mm)"),
        (&mut design.nape_recession.0, 0..=30, "Nape recession (mm)"),
        (&mut design.peak_length.0, 20..=65, "Peak reach (mm)"),
        (&mut design.peak_drop.0, 0..=20, "Peak drop (mm)"),
        (&mut design.peak_rise.0, 0..=20, "Peak rise (mm)"),
        (&mut design.comb_height.0, 0..=60, "Comb height (mm)"),
        (&mut design.cheek_depth.0, 750..=1150, "Cheek depth"),
        (
            &mut design.neck_guard_fraction.0,
            200..=550,
            "Separate neck guard length",
        ),
        (&mut design.cheek_width.0, 750..=1250, "Cheek coverage"),
        (&mut design.cheek_taper.0, 800..=1050, "Cheek taper"),
        (&mut design.neck_flare.0, 5..=40, "Nape flare (mm)"),
    ] {
        changed |= super::armor_controls::number(ui, value, range, label);
    }
    changed | show(ui, &mut design.buffe)
}

fn show(ui: &mut egui::Ui, buffe: &mut Option<BuffeDesign>) -> bool {
    let original = *buffe;
    let mut enabled = buffe.is_some();
    let toggle = ui.checkbox(&mut enabled, "Separate buffe face defense");
    if toggle.changed() {
        *buffe = enabled.then(BuffeDesign::default);
    }
    if let Some(shape) = buffe {
        course_controls(ui, &mut shape.courses);
        let mut pierced = shape.breaths.is_some();
        let toggle = ui.checkbox(&mut pierced, "Pierced buffe breaths");
        if toggle.changed() {
            shape.breaths = pierced.then(adventuresim_armor_model::VisorBreaths::buffe);
        }
        if let Some(breaths) = &mut shape.breaths {
            super::visor_breath_controls::show(ui, breaths, 50..=950);
        }
        for (value, range, label) in [
            (&mut shape.sight_gap.0, 5..=20, "Sight gap (mm)"),
            (&mut shape.face_projection.0, 0..=40, "Face projection (mm)"),
            (&mut shape.chin_width.0, 500..=1100, "Chin width"),
            (&mut shape.throat_depth.0, 550..=1000, "Throat depth"),
            (&mut shape.neck_drop.0, 0..=35, "Neck drop (mm)"),
            (&mut shape.side_wrap.0, 1500..=1950, "Side return (mrad)"),
            (&mut shape.medial_ridge.0, 0..=25, "Medial ridge (mm)"),
            (&mut shape.ridge_sharpness.0, 0..=1000, "Ridge sharpness"),
            (&mut shape.chin_point.0, 0..=60, "Chin point (mm)"),
        ] {
            super::armor_controls::number(ui, value, range, label);
        }
    }
    *buffe != original
}

fn course_controls(ui: &mut egui::Ui, courses: &mut Option<BuffeCourses>) {
    let mut enabled = courses.is_some();
    let toggle = ui.checkbox(&mut enabled, "Overlapping buffe faceplates");
    if toggle.changed() {
        *courses = enabled.then(BuffeCourses::default);
    }
    if let Some(design) = courses {
        ui.add(egui::Slider::new(&mut design.plate_count, 2..=3).text("Faceplate count"));
        for (value, range, label) in [
            (&mut design.lower_boundary.0, 200..=700, "Lower seam height"),
            (
                &mut design.upper_boundary.0,
                450..=850,
                "Upper seam height (three plates)",
            ),
            (&mut design.overlap.0, 2..=12, "Faceplate overlap (mm)"),
            (&mut design.lap_clearance.0, 0..=4, "Lap clearance (mm)"),
            (&mut design.boundary_drop.0, 0..=25, "Seam median drop (mm)"),
        ] {
            super::armor_controls::number(ui, value, range, label);
        }
    }
}
