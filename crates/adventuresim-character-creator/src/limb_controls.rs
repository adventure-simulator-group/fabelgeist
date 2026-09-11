//! Controls follow the construction of each limb defense.
use super::number;
use adventuresim_armor_model::{LimbArmorDesign as L, PlateGauge};
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, design: &mut L) -> bool {
    let mut changed = false;
    let rows = match design {
        L::Greave(d) => vec![
            (&mut d.ankle_extension.0, 0..=30, "Ankle extension (mm)"),
            (&mut d.length.0, 650..=1050, "Length"),
            (&mut d.ankle_taper.0, 450..=850, "Ankle taper"),
            (&mut d.calf_height.0, 450..=800, "Calf height"),
            (&mut d.knee_taper.0, 700..=1050, "Knee taper"),
            (&mut d.shin_ridge.0, 0..=15, "Shin ridge (mm)"),
        ],
        L::Cuisse(d) => vec![
            (&mut d.length.0, 600..=1050, "Length"),
            (&mut d.knee_taper.0, 550..=1000, "Knee taper"),
            (&mut d.wrap.0, 500..=900, "Side coverage"),
            (&mut d.center_ridge.0, 0..=15, "Central ridge (mm)"),
            (&mut d.upper_edge_slope.0, 0..=40, "Upper edge slope (mm)"),
        ],
        L::Rerebrace(d) => vec![
            (&mut d.length.0, 600..=1050, "Length"),
            (&mut d.distal_taper.0, 600..=1100, "Elbow taper"),
            (&mut d.wrap.0, 550..=950, "Side coverage"),
            (&mut d.center_ridge.0, 0..=12, "Central ridge (mm)"),
            (&mut d.section_depth.0, 850..=1200, "Section depth"),
        ],
        L::Poleyn(d) | L::Couter(d) => vec![
            (&mut d.proximal_flare.0, 0..=15, "Proximal rim flare (mm)"),
            (&mut d.length.0, 650..=1250, "Length"),
            (&mut d.dome.0, 100..=1100, "Cup projection"),
            (&mut d.wing.0, 0..=650, "Wing spread"),
            (&mut d.wing_height.0, 600..=1400, "Wing height"),
            (&mut d.wing_roundness.0, 0..=1000, "Wing roundness"),
            (&mut d.distal_wing_scale.0, 500..=2000, "Distal fan lobe"),
            (&mut d.wing_notch.0, 0..=600, "Wing notch"),
            (&mut d.center_ridge.0, 0..=12, "Central ridge (mm)"),
        ],
        L::Spaulder(d) => {
            changed |= ui
                .add(egui::Slider::new(&mut d.lame_count, 2..=7).text("Lames"))
                .changed();
            vec![
                (&mut d.length.0, 650..=1250, "Length"),
                (&mut d.crown.0, 850..=1350, "Shoulder crown"),
                (&mut d.crown_reach.0, 500..=900, "Crown reach"),
                (&mut d.lower_flare.0, 0..=15, "Lower rim flare (mm)"),
                (&mut d.wrap.0, 450..=700, "Side coverage"),
                (&mut d.rear_extension.0, 800..=1400, "Rear coverage"),
            ]
        }
        L::MittenGauntlet(d) => {
            changed |= ui
                .add(egui::Slider::new(&mut d.finger_lames, 2..=6).text("Hand lames"))
                .changed();
            vec![
                (&mut d.cuff_length.0, 150..=650, "Cuff length"),
                (&mut d.cuff_clearance.0, 0..=15, "Cuff clearance (mm)"),
                (&mut d.cuff_flare.0, 1050..=1600, "Cuff flare"),
                (&mut d.knuckle_width.0, 850..=1200, "Knuckle breadth"),
                (&mut d.knuckle_ridge.0, 0..=8, "Knuckle ridge (mm)"),
            ]
        }
        L::Sabaton(d) => {
            changed |= ui
                .add(egui::Slider::new(&mut d.lame_count, 3..=8).text("Instep lames"))
                .changed();
            vec![
                (&mut d.ankle_cutaway.0, 0..=30, "Ankle opening trim (mm)"),
                (&mut d.toe_width.0, 800..=1450, "Toe breadth"),
                (&mut d.toe_extension.0, 0..=100, "Toe extension (mm)"),
                (&mut d.instep_height.0, 850..=1250, "Instep fullness"),
                (&mut d.toe_roundness.0, 500..=1800, "Toe roundness"),
            ]
        }
        L::LeatherBoot(d) => vec![
            (&mut d.shaft_height.0, 40..=350, "Shaft height (mm)"),
            (&mut d.shaft_flare.0, 1000..=1500, "Shaft flare"),
            (&mut d.toe_width.0, 800..=1300, "Toe breadth"),
        ],
    };
    for (value, range, label) in rows {
        changed |= number(ui, value, range, label);
    }
    let gauge = match design {
        L::Greave(d) => &mut d.gauge,
        L::Cuisse(d) => &mut d.gauge,
        L::Rerebrace(d) => &mut d.gauge,
        L::Poleyn(d) | L::Couter(d) => &mut d.gauge,
        L::Spaulder(d) => &mut d.gauge,
        L::MittenGauntlet(d) => &mut d.gauge,
        L::Sabaton(d) => &mut d.gauge,
        L::LeatherBoot(d) => &mut d.gauge,
    };
    changed |= gauge_controls(ui, gauge);
    if let Some(fluting) = design.fluting_mut() {
        changed |= crate::fluting_controls::show(ui, fluting);
    }
    changed
}

fn gauge_controls(ui: &mut egui::Ui, d: &mut PlateGauge) -> bool {
    number(ui, &mut d.clearance.0, 2..=25, "Padding clearance (mm)")
        | number(ui, &mut d.thickness.0, 1..=6, "Wall thickness (mm)")
}
