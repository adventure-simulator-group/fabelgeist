//! Open-sided shoulder covering and its independently pleated standing collar.
use bevy_egui::egui;
use fabelgeist_armor::{
    ClothBinding, ClothNeckCurve, ClothPleatProfile, ClothPleats, CollarBand, CollarSection,
    Micrometers, Millimeters, PartletDesign, Permille, StandingClothCollar, TextileAppearance,
    TextileColor, TextileFinish, TextileRepeatCount,
};

pub(super) fn show(ui: &mut egui::Ui, design: &mut PartletDesign) -> bool {
    let mut changed = super::number(
        ui,
        &mut design.hem_rise.0,
        PartletDesign::HEM_RISE_RANGE,
        "Hem rise (mm)",
    );
    changed |= super::number(
        ui,
        &mut design.neck_rise.0,
        PartletDesign::NECK_RISE_RANGE,
        "Neckline rise (mm)",
    );
    changed |= super::number(
        ui,
        &mut design.neckline_slope.0,
        PartletDesign::NECKLINE_SLOPE_RANGE,
        "Neckline slope",
    );
    let mut curved = design.front_curve.is_some();
    if ui.checkbox(&mut curved, "Curved front neckline").changed() {
        changed = true;
        design.front_curve = curved.then_some(ClothNeckCurve {
            reach: Millimeters(80),
            drop: Millimeters(25),
        });
    }
    if let Some(curve) = &mut design.front_curve {
        changed |= super::number(
            ui,
            &mut curve.reach.0,
            ClothNeckCurve::REACH_RANGE,
            "Front curve reach (mm)",
        );
        changed |= super::number(
            ui,
            &mut curve.drop.0,
            ClothNeckCurve::DROP_RANGE,
            "Front neck drop (mm)",
        );
    }
    changed |= super::number(
        ui,
        &mut design.shoulder_bridge.0,
        PartletDesign::SHOULDER_BRIDGE_RANGE,
        "Shoulder bridge depth (mm)",
    );
    changed |= super::number(
        ui,
        &mut design.side_opening.0,
        PartletDesign::SIDE_OPENING_RANGE,
        "Side opening angle",
    );
    changed |= super::number(
        ui,
        &mut design.back_slit.0,
        PartletDesign::BACK_SLIT_RANGE,
        "Back slit length (mm)",
    );
    changed |= super::number(
        ui,
        &mut design.back_gap.0,
        PartletDesign::BACK_GAP_RANGE,
        "Back slit width (mm)",
    );
    changed |= super::number(
        ui,
        &mut design.clearance.0,
        PartletDesign::CLEARANCE_RANGE,
        "Body clearance (mm)",
    );
    changed |= super::number(
        ui,
        &mut design.thickness.0,
        PartletDesign::THICKNESS_RANGE,
        "Cloth gauge (micrometres)",
    );
    changed |= super::cloth_ties::show(ui, &mut design.ties);
    changed |= collar_controls(ui, &mut design.collar);
    changed |= binding_controls(ui, &mut design.binding);
    ui.collapsing("Panel fabric", |ui| {
        changed |= super::textile::show(ui, &mut design.fabric)
    });
    changed
}

fn collar_controls(ui: &mut egui::Ui, design: &mut Option<StandingClothCollar>) -> bool {
    let mut changed = false;
    let mut enabled = design.is_some();
    if ui.checkbox(&mut enabled, "Standing collar").changed() {
        changed = true;
        *design = enabled.then_some(StandingClothCollar {
            height: Millimeters(44),
            fabric: TextileAppearance::solid(TextileColor([242, 240, 230]), TextileFinish::Linen),
            top_scale: Permille::ONE,
            taper_height: Millimeters(44),
            section: CollarSection::Fitted,
            flare: Millimeters(0),
            neckband: None,
            trim: None,
            pleats: Some(ClothPleats {
                count: TextileRepeatCount(40),
                depth: Millimeters(2),
                profile: ClothPleatProfile::Fluted,
            }),
        });
    }
    if let Some(collar) = design {
        changed |= section_controls(ui, &mut collar.section);
        ui.collapsing("Collar fabric", |ui| {
            changed |= super::textile::show(ui, &mut collar.fabric)
        });
        changed |= band_controls(
            ui,
            &mut collar.neckband,
            "Inner neckband",
            collar.height,
            TextileColor([242, 240, 230]),
        );
        changed |= band_controls(
            ui,
            &mut collar.trim,
            "Outer collar trim",
            collar.height,
            TextileColor([190, 145, 40]),
        );
        changed |= super::number(
            ui,
            &mut collar.top_scale.0,
            StandingClothCollar::TOP_SCALE_RANGE,
            "Collar top circumference",
        );
        changed |= super::number(
            ui,
            &mut collar.flare.0,
            StandingClothCollar::FLARE_RANGE,
            "Free-edge flare (mm)",
        );
        changed |= super::number(
            ui,
            &mut collar.taper_height.0,
            1..=collar.height.0,
            "Collar base transition (mm)",
        );
        changed |= super::number(
            ui,
            &mut collar.height.0,
            StandingClothCollar::HEIGHT_RANGE,
            "Collar height (mm)",
        );
        let mut pleated = collar.pleats.is_some();
        if ui.checkbox(&mut pleated, "Even collar pleats").changed() {
            changed = true;
            collar.pleats = pleated.then_some(ClothPleats {
                count: TextileRepeatCount(40),
                depth: Millimeters(2),
                profile: ClothPleatProfile::Fluted,
            });
        }
        if let Some(pleats) = &mut collar.pleats {
            changed |= ui
                .add(
                    egui::Slider::new(&mut pleats.count.0, StandingClothCollar::PLEAT_COUNT_RANGE)
                        .text("Pleat count"),
                )
                .changed();
            changed |= ui
                .selectable_value(&mut pleats.profile, ClothPleatProfile::Fluted, "Flutes")
                .changed();
            changed |= ui
                .selectable_value(
                    &mut pleats.profile,
                    ClothPleatProfile::RoundedGathers,
                    "Rounded gathers",
                )
                .changed();
            changed |= super::number(
                ui,
                &mut pleats.depth.0,
                StandingClothCollar::PLEAT_DEPTH_RANGE,
                "Pleat depth (mm)",
            );
        }
    }
    changed
}

fn section_controls(ui: &mut egui::Ui, section: &mut CollarSection) -> bool {
    let mut changed = false;
    let mut oval = matches!(section, CollarSection::Oval { .. });
    if ui.checkbox(&mut oval, "Oval tailor-rest stand").changed() {
        changed = true;
        *section = if oval {
            CollarSection::Oval {
                depth_to_width: Permille(750),
            }
        } else {
            CollarSection::Fitted
        };
    }
    if let CollarSection::Oval { depth_to_width } = section {
        changed |= super::number(
            ui,
            &mut depth_to_width.0,
            CollarSection::DEPTH_TO_WIDTH_RANGE,
            "Stand depth relative to width",
        );
    }
    changed
}

fn binding_controls(ui: &mut egui::Ui, design: &mut Option<ClothBinding>) -> bool {
    let mut changed = false;
    let mut bound = design.is_some();
    if ui.checkbox(&mut bound, "Bound cloth edges").changed() {
        changed = true;
        *design = bound.then_some(ClothBinding {
            width: Millimeters(5),
            hem_width: Millimeters(13),
            thickness: Micrometers(200),
            fabric: TextileAppearance::solid(TextileColor([242, 240, 230]), TextileFinish::Linen),
        });
    }
    if let Some(binding) = design {
        changed |= super::number(
            ui,
            &mut binding.width.0,
            ClothBinding::WIDTH_RANGE,
            "Edge binding width (mm)",
        );
        changed |= super::number(
            ui,
            &mut binding.hem_width.0,
            ClothBinding::WIDTH_RANGE,
            "Hem binding width (mm)",
        );
        changed |= super::number(
            ui,
            &mut binding.thickness.0,
            ClothBinding::THICKNESS_RANGE,
            "Binding gauge (micrometres)",
        );
        ui.collapsing("Binding fabric", |ui| {
            changed |= super::textile::show(ui, &mut binding.fabric)
        });
    }
    changed
}

fn band_controls(
    ui: &mut egui::Ui,
    band: &mut Option<CollarBand>,
    label: &str,
    height: Millimeters,
    color: TextileColor,
) -> bool {
    let mut enabled = band.is_some();
    let mut changed = ui.checkbox(&mut enabled, label).changed();
    if changed {
        *band = enabled.then_some(CollarBand {
            bottom: Millimeters(0),
            height: Millimeters(height.0.min(15)),
            thickness: Micrometers(200),
            fabric: TextileAppearance::solid(color, TextileFinish::Linen),
        });
    }
    if let Some(band) = band {
        changed |= super::number(
            ui,
            &mut band.bottom.0,
            0..=height.0.saturating_sub(1),
            "Band bottom (mm)",
        );
        changed |= super::number(
            ui,
            &mut band.height.0,
            1..=height.0.saturating_sub(band.bottom.0).max(1),
            "Band height (mm)",
        );
        changed |= super::number(
            ui,
            &mut band.thickness.0,
            ClothBinding::THICKNESS_RANGE,
            "Band gauge (micrometres)",
        );
        ui.collapsing(label, |ui| {
            changed |= super::textile::show(ui, &mut band.fabric)
        });
    }
    changed
}
