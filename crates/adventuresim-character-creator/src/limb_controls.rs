//! Controls follow the construction of each limb defense.
use super::number;
use adventuresim_armor_model::LimbArmorDesign as L;
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
        L::Poleyn(d) | L::Couter(d) => {
            changed |= joint_construction(ui, d);
            if d.construction == adventuresim_armor_model::JointCupConstruction::Wrapped {
                changed |= crate::joint_extension_controls::show(ui, &mut d.distal_extension);
            }
            vec![
                (&mut d.proximal_flare.0, 0..=15, "Proximal rim flare (mm)"),
                (&mut d.length.0, 650..=1250, "Length"),
                (&mut d.dome.0, 100..=1100, "Cup projection"),
                (&mut d.wing.0, d.construction.wing_range(), "Wing spread"),
                (&mut d.wing_height.0, 600..=1400, "Wing height"),
                (&mut d.wing_roundness.0, 0..=1000, "Wing roundness"),
                (&mut d.distal_wing_scale.0, 500..=2000, "Distal fan lobe"),
                (&mut d.wing_notch.0, 0..=600, "Wing notch"),
                (&mut d.center_ridge.0, 0..=12, "Central ridge (mm)"),
            ]
        }
        L::Spaulder(d) => {
            changed |= crate::besagew_controls::show(ui, &mut d.besagew);
            changed |= ui
                .add(egui::Slider::new(&mut d.lame_count, 2..=7).text("Lames"))
                .changed();
            vec![
                (&mut d.length.0, 650..=1250, "Length"),
                (&mut d.crown.0, 850..=1350, "Shoulder crown"),
                (&mut d.crown_reach.0, 500..=900, "Crown reach"),
                (&mut d.crown_coverage.0, 200..=1000, "Crown coverage"),
                (&mut d.lower_flare.0, 0..=15, "Lower rim flare (mm)"),
                (&mut d.wrap.0, 350..=700, "Side coverage"),
                (&mut d.rear_extension.0, 800..=1400, "Rear coverage"),
            ]
        }
        L::Pauldron(d) => {
            changed |= pauldron_controls(ui, d);
            vec![]
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
    changed | material_controls(ui, design)
}

fn joint_construction(ui: &mut egui::Ui, d: &mut adventuresim_armor_model::JointCupDesign) -> bool {
    use adventuresim_armor_model::JointCupConstruction;
    let mut changed = false;
    ui.horizontal(|ui| {
        changed |= ui
            .selectable_value(
                &mut d.construction,
                JointCupConstruction::Wrapped,
                "Wrapped joint plate",
            )
            .changed();
        changed |= ui
            .selectable_value(
                &mut d.construction,
                JointCupConstruction::RaisedCop,
                "Raised cop",
            )
            .changed();
    });
    if changed {
        if d.construction == JointCupConstruction::RaisedCop {
            d.distal_extension = None;
        }
        d.wing.0 = d.wing.0.min(*d.construction.wing_range().end());
    }
    if d.construction == JointCupConstruction::RaisedCop {
        changed |= number(
            ui,
            &mut d.medial_wrap.0,
            adventuresim_armor_model::JointCupDesign::MEDIAL_WRAP_RANGE,
            "Medial coverage",
        );
        changed |= number(
            ui,
            &mut d.lateral_wrap.0,
            adventuresim_armor_model::JointCupDesign::LATERAL_WRAP_RANGE,
            "Lateral coverage",
        );
    }
    use adventuresim_armor_model::JointFluteOrientation;
    ui.horizontal(|ui| {
        ui.label("Flute direction");
        changed |= ui
            .selectable_value(
                &mut d.flute_orientation,
                JointFluteOrientation::Longitudinal,
                "Along limb",
            )
            .changed();
        changed |= ui
            .selectable_value(
                &mut d.flute_orientation,
                JointFluteOrientation::Transverse,
                "Across fan",
            )
            .changed();
    });
    changed
}

fn material_controls(ui: &mut egui::Ui, design: &mut L) -> bool {
    let thickness_range = if matches!(design, L::Pauldron(_)) {
        adventuresim_armor_model::PauldronDesign::THICKNESS_RANGE
    } else {
        1..=6
    };
    let gauge = match design {
        L::Greave(d) => &mut d.gauge,
        L::Cuisse(d) => &mut d.gauge,
        L::Rerebrace(d) => &mut d.gauge,
        L::Poleyn(d) | L::Couter(d) => &mut d.gauge,
        L::Spaulder(d) => &mut d.gauge,
        L::Pauldron(d) => &mut d.gauge,
        L::MittenGauntlet(d) => &mut d.gauge,
        L::Sabaton(d) => &mut d.gauge,
        L::LeatherBoot(d) => &mut d.gauge,
    };
    let mut changed = number(ui, &mut gauge.clearance.0, 2..=25, "Padding clearance (mm)")
        | number(
            ui,
            &mut gauge.thickness.0,
            thickness_range,
            "Wall thickness (mm)",
        );
    if let Some(fluting) = design.fluting_mut() {
        changed |= crate::fluting_controls::show(ui, fluting);
    }
    changed
}

fn pauldron_controls(ui: &mut egui::Ui, d: &mut adventuresim_armor_model::PauldronDesign) -> bool {
    let mut changed = ui
        .add(egui::Slider::new(&mut d.upper_lames, 1..=3).text("Neck lames"))
        .changed();
    changed |= ui
        .add(egui::Slider::new(&mut d.lower_lames, 3..=7).text("Arm lames"))
        .changed();
    for (value, range, label) in [
        (
            &mut d.front_reach.0,
            adventuresim_armor_model::PauldronDesign::FRONT_REACH_RANGE,
            "Front wing reach (mm)",
        ),
        (
            &mut d.outline.front_return.0,
            1800..=2800,
            "Front wing return (mrad)",
        ),
        (
            &mut d.outline.rear_return.0,
            1800..=2800,
            "Rear wing return (mrad)",
        ),
        (&mut d.rear_reach.0, 60..=145, "Rear wing reach (mm)"),
        (&mut d.front_drop.0, 0..=60, "Front wing drop (mm)"),
        (&mut d.rear_drop.0, 0..=75, "Rear wing drop (mm)"),
        (&mut d.neck_reach.0, 20..=60, "Neck reach (mm)"),
        (
            &mut d.plate_clearance.0,
            adventuresim_armor_model::PauldronDesign::PLATE_CLEARANCE_RANGE,
            "Supporting plate separation (mm)",
        ),
        (&mut d.arm_allowance.0, 0..=20, "Rerebrace allowance (mm)"),
        (&mut d.crown_height.0, 1000..=1450, "Shoulder crown"),
        (&mut d.arm_length.0, 65..=125, "Arm lames length (mm)"),
        (
            &mut d.outline.front_extension.0,
            0..=90,
            "Main front wing extension (mm)",
        ),
        (
            &mut d.outline.rear_extension.0,
            0..=90,
            "Main rear wing extension (mm)",
        ),
        (
            &mut d.outline.wing_start.0,
            800..=1500,
            "Wing descent start (mrad)",
        ),
        (
            &mut d.outline.corner_rounding.0,
            adventuresim_armor_model::PauldronOutline::CORNER_ROUNDING_RANGE,
            "Angular corner rounding",
        ),
        (
            &mut d.outline.upper_span.0,
            150..=400,
            "Upper lames share of shoulder",
        ),
        (
            &mut d.outline.arm_wrap.0,
            1400..=1800,
            "Arm lame half-wrap (mrad)",
        ),
    ] {
        changed |= number(ui, value, range, label);
    }
    changed | hanging_wing_controls(ui, &mut d.outline)
}

fn hanging_wing_controls(
    ui: &mut egui::Ui,
    d: &mut adventuresim_armor_model::PauldronOutline,
) -> bool {
    use adventuresim_armor_model::PauldronOutline as Outline;
    let mut changed = false;
    for (value, range, label) in [
        (
            &mut d.front_wing_position.0,
            Outline::WING_POSITION_RANGE,
            "Front wing low-region position",
        ),
        (
            &mut d.rear_wing_position.0,
            Outline::WING_POSITION_RANGE,
            "Rear wing low-region position",
        ),
        (
            &mut d.front_wing_rounding.0,
            Outline::WING_ROUNDING_RANGE,
            "Front hanging wing rounding",
        ),
        (
            &mut d.rear_wing_rounding.0,
            Outline::WING_ROUNDING_RANGE,
            "Rear hanging wing rounding",
        ),
    ] {
        changed |= number(ui, value, range, label);
    }
    changed
}
