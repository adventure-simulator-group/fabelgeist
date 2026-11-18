//! Construction controls shared by fitted upper garments.
use bevy_egui::egui;
use fabelgeist_armor::{
    ArmholeEdge, BodiceDesign, BodiceInterlining, BodiceLining, BodiceOpening, InterliningCoverage,
    Millimeters, NecklineShape, Permille, TextileAppearance, TextileColor, TextileFinish,
};

use super::{number, textile};

pub(super) fn show(
    ui: &mut egui::Ui,
    design: &mut BodiceDesign,
    hem_range: std::ops::RangeInclusive<u16>,
) -> bool {
    let mut changed = false;
    let mut rounded = design.neckline.shape != NecklineShape::Square;
    let rounded_response = ui.checkbox(&mut rounded, "Rounded neckline");
    if rounded_response.changed() {
        design.neckline.shape = if rounded {
            NecklineShape::Rounded {
                squareness: Permille::ONE,
            }
        } else {
            NecklineShape::Square
        };
        changed = true;
    }
    if let NecklineShape::Rounded { squareness } = &mut design.neckline.shape {
        changed |= number(ui, &mut squareness.0, 0..=Permille::ONE.0, "Squareness");
    }
    changed |= number(
        ui,
        &mut design.neckline.width.0,
        BodiceDesign::NECK_WIDTH_RANGE,
        "Neckline width",
    );
    changed |= number(
        ui,
        &mut design.neckline.front_drop.0,
        BodiceDesign::FRONT_DROP_RANGE,
        "Front neckline drop (mm)",
    );
    changed |= number(
        ui,
        &mut design.neckline.back_drop.0,
        BodiceDesign::BACK_DROP_RANGE,
        "Back neckline drop (mm)",
    );
    let mut capped = matches!(design.armhole_edge, ArmholeEdge::Beyond(_));
    let capped_response = ui.checkbox(&mut capped, "Cloth caps the shoulder");
    if capped_response.changed() {
        design.armhole_edge = if capped {
            ArmholeEdge::Beyond(Millimeters(*BodiceDesign::ARMHOLE_BEYOND_RANGE.start()))
        } else {
            ArmholeEdge::Inside(Millimeters(*BodiceDesign::ARMHOLE_INSIDE_RANGE.start()))
        };
        changed = true;
    }
    changed |= match &mut design.armhole_edge {
        ArmholeEdge::Inside(distance) => number(
            ui,
            &mut distance.0,
            BodiceDesign::ARMHOLE_INSIDE_RANGE,
            "Armhole edge inside the shoulder joint (mm)",
        ),
        ArmholeEdge::Beyond(distance) => number(
            ui,
            &mut distance.0,
            BodiceDesign::ARMHOLE_BEYOND_RANGE,
            "Armhole edge beyond the shoulder joint (mm)",
        ),
    };
    changed |= number(
        ui,
        &mut design.armhole_depth.0,
        BodiceDesign::ARMHOLE_DEPTH_RANGE,
        "Armhole depth (mm)",
    );
    changed |= number(ui, &mut design.hem_rise.0, hem_range, "Hem rise (mm)");
    changed |= number(
        ui,
        &mut design.clearance.0,
        BodiceDesign::CLEARANCE_RANGE,
        "Body clearance (mm)",
    );
    changed |= number(
        ui,
        &mut design.thickness.0,
        BodiceDesign::THICKNESS_RANGE,
        "Face cloth thickness (mm)",
    );
    changed |= opening(ui, design);
    changed |= textile::show(ui, &mut design.fabric);
    changed |= lining(ui, design);
    changed
}

fn opening(ui: &mut egui::Ui, design: &mut BodiceDesign) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let close = ui
            .selectable_label(matches!(design.opening, BodiceOpening::Closed), "Closed")
            .clicked();
        if close {
            changed |= !matches!(design.opening, BodiceOpening::Closed);
            design.opening = BodiceOpening::Closed;
            if let Some(interlining) = &mut design.interlining {
                interlining.coverage = InterliningCoverage::Full;
            }
        }
        for choice in [
            BodiceOpening::Front {
                waist: Millimeters(35),
                chest: Millimeters(60),
            },
            BodiceOpening::Back {
                waist: Millimeters(15),
                chest: Millimeters(15),
            },
        ] {
            let label = match choice {
                BodiceOpening::Front { .. } => "Front opening",
                _ => "Back opening",
            };
            if ui
                .selectable_label(
                    std::mem::discriminant(&design.opening) == std::mem::discriminant(&choice),
                    label,
                )
                .clicked()
                && std::mem::discriminant(&design.opening) != std::mem::discriminant(&choice)
            {
                design.opening = choice;
                changed = true;
            }
        }
    });
    if let BodiceOpening::Front { waist, chest } | BodiceOpening::Back { waist, chest } =
        &mut design.opening
    {
        changed |= number(
            ui,
            &mut waist.0,
            BodiceDesign::WAIST_OPENING_RANGE,
            "Opening at waist (mm)",
        );
        changed |= number(
            ui,
            &mut chest.0,
            BodiceDesign::CHEST_OPENING_RANGE,
            "Opening at neck (mm)",
        );
    }
    changed
}

fn lining(ui: &mut egui::Ui, design: &mut BodiceDesign) -> bool {
    let mut enabled = design.lining.is_some();
    let mut changed = ui.checkbox(&mut enabled, "Separate lining").changed();
    if changed {
        design.lining = enabled.then(|| BodiceLining {
            fabric: TextileAppearance::solid(TextileColor([32, 29, 28]), TextileFinish::Linen),
            thickness: Millimeters(1),
        });
        if !enabled {
            design.interlining = None;
        }
    }
    if let Some(lining) = &mut design.lining {
        changed |= ui
            .collapsing("Lining", |ui| {
                let mut changed = number(
                    ui,
                    &mut lining.thickness.0,
                    BodiceDesign::LINING_THICKNESS_RANGE,
                    "Lining thickness (mm)",
                );
                changed |= textile::show(ui, &mut lining.fabric);
                changed
            })
            .body_returned
            .unwrap_or(false);
        let mut enabled = design.interlining.is_some();
        let interlining_changed = ui.checkbox(&mut enabled, "Separate interlining").changed();
        if interlining_changed {
            changed = true;
            design.interlining = enabled.then(|| BodiceInterlining {
                fabric: lining.fabric.clone(),
                thickness: Millimeters(1),
                coverage: InterliningCoverage::Full,
            });
        }
    }
    if let Some(interlining) = &mut design.interlining {
        changed |= ui
            .collapsing("Stiffening", |ui| {
                let mut changed = ui
                    .selectable_value(
                        &mut interlining.coverage,
                        InterliningCoverage::Full,
                        "Full supporting sheet",
                    )
                    .changed();
                if !matches!(design.opening, BodiceOpening::Closed) {
                    let facings = ui
                        .selectable_label(
                            matches!(interlining.coverage, InterliningCoverage::Facings { .. }),
                            "Opening and neck facings",
                        )
                        .clicked();
                    if facings
                        && !matches!(interlining.coverage, InterliningCoverage::Facings { .. })
                    {
                        interlining.coverage = InterliningCoverage::Facings {
                            width: Millimeters(35),
                        };
                        changed = true;
                    }
                }
                if let InterliningCoverage::Facings { width } = &mut interlining.coverage {
                    changed |= number(
                        ui,
                        &mut width.0,
                        BodiceDesign::FACING_WIDTH_RANGE,
                        "Facing width (mm)",
                    );
                }
                changed |= number(
                    ui,
                    &mut interlining.thickness.0,
                    BodiceDesign::LINING_THICKNESS_RANGE,
                    "Facing thickness (mm)",
                );
                changed |= textile::show(ui, &mut interlining.fabric);
                changed
            })
            .body_returned
            .unwrap_or(false);
    }
    changed
}
