//! Helmet controls distinguish bowls, integrated faces and attached plates.
use super::number;
use adventuresim_armor_model::{
    CloseHelmetDesign, HelmetCrown, HelmetDesign as H, HelmetFit, SalletDesign, VentSides,
    VisorBreaths,
};
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, design: &mut H) -> bool {
    let mut changed = false;
    let rows = match design {
        H::Morion(d) => vec![
            (&mut d.brim_width.0, 20..=65, "Brim breadth (mm)"),
            (&mut d.brim_sweep.0, 15..=60, "Brim sweep (mm)"),
            (&mut d.front_reach.0, 800..=2000, "Front reach"),
            (&mut d.back_reach.0, 800..=2000, "Rear reach"),
            (&mut d.back_sweep.0, 500..=1500, "Rear sweep"),
            (&mut d.comb_height.0, 15..=85, "Comb height (mm)"),
        ],
        H::KettleHat(d) => vec![
            (&mut d.brim_width.0, 30..=85, "Brim breadth (mm)"),
            (&mut d.brim_drop.0, 5..=45, "Brim drop (mm)"),
        ],
        H::Barbute(d) => vec![
            (&mut d.rear_edge_lift.0, 0..=60, "Rear edge lift (mm)"),
            (&mut d.eye_height.0, 14..=45, "Eye opening height (mm)"),
            (&mut d.opening_roundness.0, 0..=1000, "Opening roundness"),
            (&mut d.eye_opening.0, 500..=850, "Eye opening (mrad)"),
            (&mut d.mouth_opening.0, 120..=400, "Mouth opening (mrad)"),
            (&mut d.cheek_depth.0, 750..=1400, "Cheek depth"),
            (&mut d.chin_taper.0, 750..=1000, "Chin taper"),
            (&mut d.nape_flare.0, 0..=12, "Nape flare (mm)"),
        ],
        H::Burgonet(d) => {
            ui.label("Cheek fluting");
            changed |= crate::fluting_controls::show(ui, &mut d.cheek_fluting);
            vec![
                (&mut d.nape_depth.0, 650..=1400, "Nape depth"),
                (&mut d.nape_taper.0, 550..=1000, "Nape taper"),
                (&mut d.chin_tab.0, 0..=35, "Chin tab (mm)"),
                (&mut d.nape_recession.0, 0..=30, "Nape recession (mm)"),
                (&mut d.peak_length.0, 20..=65, "Peak reach (mm)"),
                (&mut d.peak_drop.0, 0..=20, "Peak drop (mm)"),
                (&mut d.comb_height.0, 0..=60, "Comb height (mm)"),
                (&mut d.cheek_depth.0, 750..=1150, "Cheek depth"),
                (
                    &mut d.neck_guard_fraction.0,
                    200..=550,
                    "Separate neck guard length",
                ),
                (&mut d.cheek_width.0, 750..=1250, "Cheek coverage"),
                (&mut d.cheek_taper.0, 800..=1050, "Cheek taper"),
                (&mut d.neck_flare.0, 5..=40, "Nape flare (mm)"),
            ]
        }
        H::Sallet(d) => {
            changed |= sallet(ui, d);
            Vec::new()
        }
        H::VisoredSallet(d) => {
            changed |= sallet(ui, &mut d.skull);
            vec![
                (&mut d.side_panel.0, 300..=1000, "Visor side coverage"),
                (&mut d.visor_height.0, 35..=75, "Visor height (mm)"),
                (&mut d.pivot_rise.0, 25..=60, "Pivot rise (mm)"),
                (&mut d.visor_projection.0, 12..=50, "Visor projection (mm)"),
                (&mut d.sight_gap.0, 5..=15, "Sight opening (mm)"),
            ]
        }
        H::CloseHelmet(d) => {
            changed |= close(ui, d);
            Vec::new()
        }
        H::ArmingCap(_) => Vec::new(),
        H::MailCoif(d) => vec![
            (&mut d.neck_coverage.0, 800..=1100, "Neck coverage"),
            (&mut d.front_flap_length.0, 60..=150, "Front length (mm)"),
            (&mut d.back_flap_length.0, 70..=170, "Rear length (mm)"),
            (&mut d.flap_width.0, 750..=1200, "Flap width"),
        ],
    };
    for (value, range, label) in rows {
        changed |= number(ui, value, range, label);
    }
    let maximum_gauge = if matches!(design, H::CloseHelmet(_)) {
        4
    } else {
        8
    };
    let (fit, crown) = match design {
        H::Morion(d) => (&mut d.fit, Some(&mut d.crown)),
        H::KettleHat(d) => (&mut d.fit, Some(&mut d.crown)),
        H::Barbute(d) => (&mut d.fit, Some(&mut d.crown)),
        H::Burgonet(d) => (&mut d.fit, Some(&mut d.crown)),
        H::Sallet(d) => (&mut d.fit, Some(&mut d.crown)),
        H::VisoredSallet(d) => (&mut d.skull.fit, Some(&mut d.skull.crown)),
        H::CloseHelmet(d) => (&mut d.fit, Some(&mut d.crown)),
        H::ArmingCap(d) => (d, None),
        H::MailCoif(d) => (&mut d.fit, None),
    };
    changed |= fitting(ui, fit, maximum_gauge);
    if let Some(crown) = crown {
        changed |= crown_controls(ui, crown);
    }
    changed
}

fn sallet(ui: &mut egui::Ui, d: &mut SalletDesign) -> bool {
    let mut changed = false;
    for (value, range, label) in [
        (&mut d.rear_edge_lift.0, 0..=60, "Rear skirt lift (mm)"),
        (
            &mut d.opening_width.0,
            550..=1250,
            "Face opening half-angle (mrad)",
        ),
        (&mut d.opening_sweep.0, 0..=2000, "Opening sweep"),
        (&mut d.cheek_depth.0, 350..=1400, "Cheek depth"),
        (&mut d.tail_length.0, 35..=110, "Tail reach (mm)"),
        (&mut d.tail_drop.0, 10..=45, "Tail drop (mm)"),
        (&mut d.tail_width.0, 650..=1400, "Tail breadth"),
        (&mut d.brow_projection.0, 0..=20, "Brow projection (mm)"),
    ] {
        changed |= number(ui, value, range, label);
    }
    changed
}

fn fitting(ui: &mut egui::Ui, fit: &mut HelmetFit, maximum_gauge: u16) -> bool {
    number(ui, &mut fit.clearance.0, 3..=30, "Lining clearance (mm)")
        | number(ui, &mut fit.crown_height.0, 900..=1300, "Crown height")
        | number(
            ui,
            &mut fit.wall_thickness.0,
            1..=maximum_gauge,
            "Wall thickness (mm)",
        )
}

fn crown_controls(ui: &mut egui::Ui, crown: &mut HelmetCrown) -> bool {
    ui.label("Skull bowl");
    number(ui, &mut crown.fullness.0, 700..=1150, "Crown fullness")
        | number(ui, &mut crown.ridge_height.0, 0..=18, "Formed ridge (mm)")
        | crate::fluting_controls::show(ui, &mut crown.fluting)
}

fn close(ui: &mut egui::Ui, d: &mut CloseHelmetDesign) -> bool {
    let original = *d;
    let mut changed = false;
    for (value, range, label) in [
        (&mut d.neck_length.0, 0..=45, "Neck extension (mm)"),
        (&mut d.throat_flare.0, 0..=15, "Throat flange (mm)"),
        (&mut d.back_flare.0, 0..=20, "Rear neck flange (mm)"),
        (&mut d.back_edge_lift.0, 10..=45, "Rear neck edge lift (mm)"),
        (&mut d.sight_ledge.0, 0..=12, "Eye ledge (mm)"),
        (&mut d.face_clearance.0, 3..=20, "Face clearance (mm)"),
        (&mut d.temple_clearance.0, 3..=15, "Temple clearance (mm)"),
        (&mut d.jaw_width.0, 700..=1000, "Jaw breadth"),
        (&mut d.neck_width.0, 650..=1000, "Neck breadth"),
        (&mut d.visor_projection.0, 12..=50, "Visor projection (mm)"),
        (&mut d.chin_projection.0, 0..=25, "Chin projection (mm)"),
        (&mut d.sight_span.0, 70..=190, "Sight span (mm)"),
        (&mut d.sight_gap.0, 3..=12, "Sight height (mm)"),
        (&mut d.sight_bridge.0, 0..=12, "Sight bridge (mm)"),
        (&mut d.ridge_height.0, 300..=550, "Visor ridge height"),
        (&mut d.ridge_sharpness.0, 0..=1000, "Visor ridge sharpness"),
        (&mut d.comb_height.0, 0..=40, "Comb height (mm)"),
        (&mut d.nape_length.0, 0..=80, "Nape length (mm)"),
        (&mut d.nape_flare.0, 0..=90, "Nape flare (mm)"),
    ] {
        changed |= number(ui, value, range, label);
    }
    ui.label("Visor fluting");
    changed |= crate::fluting_controls::show(ui, &mut d.visor_fluting);
    changed |= breaths(ui, &mut d.breaths);
    if changed && let Err(error) = H::CloseHelmet(*d).validate() {
        *d = original;
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("Edit rejected: {error}"),
        );
        return false;
    }
    changed
}

fn breaths(ui: &mut egui::Ui, d: &mut VisorBreaths) -> bool {
    let mut changed = false;
    ui.label("Visor breaths");
    changed |= ui
        .add(egui::Slider::new(&mut d.count_per_row, 0..=8).text("Slits per row"))
        .changed();
    changed |= ui
        .add(egui::Slider::new(&mut d.rows, 1..=2).text("Rows"))
        .changed();
    ui.horizontal(|ui| {
        ui.label("Breath sides");
        for (side, label) in [
            (VentSides::Both, "Both"),
            (VentSides::Left, "Left"),
            (VentSides::Right, "Right"),
        ] {
            changed |= ui.selectable_value(&mut d.sides, side, label).changed();
        }
    });
    for (value, range, label) in [
        (&mut d.width.0, 2..=6, "Slit width (mm)"),
        (&mut d.length.0, 8..=20, "Slit length (mm)"),
        (&mut d.span.0, 20..=60, "Pattern span (mm)"),
        (&mut d.row_spacing.0, 14..=25, "Row spacing (mm)"),
        (&mut d.center_offset.0, 10..=100, "Pattern offset (mm)"),
        (&mut d.height.0, 450..=850, "Pattern height"),
        (&mut d.rounding.0, 0..=1000, "Slit roundness"),
    ] {
        changed |= number(ui, value, range, label);
    }
    changed |= ui
        .add(
            egui::Slider::new(&mut d.inclination.0, -90..=90)
                .text("Slit angle")
                .suffix("°"),
        )
        .changed();
    changed
}
