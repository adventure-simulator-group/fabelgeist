//! Author lacing stations, cord routing, and metal or sewn eyelets.
use bevy_egui::egui;
use fabelgeist_armor::{
    BodiceOpening, ClothLacing, GownBodyFit, GownDesign, LaceRouting, Micrometers, Millimeters,
    Permille, TextileAppearance, TextileColor, TextileFinish,
};

pub(super) fn show(ui: &mut egui::Ui, gown: &mut GownDesign) -> bool {
    let supported = !matches!(gown.bodice.opening, BodiceOpening::Closed)
        && matches!(gown.body_fit, GownBodyFit::Fitted);
    let mut enabled = gown.lacing.is_some();
    let response = ui.add_enabled(
        supported || enabled,
        egui::Checkbox::new(&mut enabled, "Opening lacing"),
    );
    let mut changed = response.changed();
    if changed {
        gown.lacing = enabled.then(|| ClothLacing {
            positions: vec![Permille(100), Permille(350), Permille(600), Permille(850)],
            routing: LaceRouting::Crossed,
            ring_radius: Millimeters(3),
            eyelet: fabelgeist_armor::LacingEyelet::Metal {
                wire: Micrometers(500),
                color: TextileColor([80, 70, 50]),
            },
            cord: Micrometers(1500),
            inset: Millimeters(4),
            ring_seat: Millimeters(2),
            cord_lift: Millimeters(3),
            fabric: TextileAppearance::solid(TextileColor([12, 10, 12]), TextileFinish::Linen),
        });
    }
    if !supported {
        ui.label("Lacing requires a fitted body and a front or back opening.");
    }
    if let Some(lacing) = &mut gown.lacing {
        changed |= ui
            .collapsing("Lacing construction", |ui| construction(ui, lacing))
            .body_returned
            .unwrap_or(false);
    }
    changed
}
fn construction(ui: &mut egui::Ui, lacing: &mut ClothLacing) -> bool {
    let mut changed = false;
    let mut spiral = lacing.routing == LaceRouting::Spiral;
    let response = ui.checkbox(&mut spiral, "Spiral route");
    if response.changed() {
        lacing.routing = if spiral {
            LaceRouting::Spiral
        } else {
            LaceRouting::Crossed
        };
        changed = true;
    }
    let mut count = lacing.positions.len();
    let response =
        ui.add(egui::Slider::new(&mut count, ClothLacing::COUNT_RANGE).text("Ring pairs"));
    if response.changed() {
        lacing.positions = (1..=count)
            .map(|i| Permille((i * usize::from(Permille::ONE.0) / (count + 1)) as u16))
            .collect();
        changed = true;
    }
    for i in 0..lacing.positions.len() {
        let low = if i == 0 {
            1
        } else {
            lacing.positions[i - 1].0 + 1
        };
        let high = if i + 1 == lacing.positions.len() {
            999
        } else {
            lacing.positions[i + 1].0 - 1
        };
        changed |= super::number(
            ui,
            &mut lacing.positions[i].0,
            low..=high,
            &format!("Ring pair {} position", i + 1),
        );
    }
    changed |= eyelet(ui, lacing);
    let cord_range = lacing.cord_range();
    let cord = lacing.cord.0.clamp(*cord_range.start(), *cord_range.end());
    changed |= cord != lacing.cord.0;
    lacing.cord.0 = cord;
    changed |= super::number(
        ui,
        &mut lacing.cord.0,
        cord_range,
        "Cord diameter (micrometres)",
    );
    if matches!(lacing.eyelet, fabelgeist_armor::LacingEyelet::Metal { .. }) {
        changed |= super::number(
            ui,
            &mut lacing.ring_seat.0,
            ClothLacing::SEAT_RANGE,
            "Inner ring depth (mm)",
        );
    }
    changed |= super::number(
        ui,
        &mut lacing.cord_lift.0,
        ClothLacing::LIFT_RANGE,
        "Cord face lift (mm)",
    );
    changed |= super::textile::show(ui, &mut lacing.fabric);
    changed
}

fn eyelet(ui: &mut egui::Ui, lacing: &mut ClothLacing) -> bool {
    let mut changed = false;
    let mut sewn = matches!(lacing.eyelet, fabelgeist_armor::LacingEyelet::Sewn { .. });
    let eyelet_response = ui.checkbox(&mut sewn, "Hand-sewn cloth eyelets");
    if eyelet_response.changed() {
        let gauge = lacing.eyelet.gauge();
        if sewn {
            lacing.ring_seat = Millimeters(0);
        }
        lacing.eyelet = if sewn {
            fabelgeist_armor::LacingEyelet::Sewn {
                thread: gauge,
                fabric: lacing.fabric.clone(),
            }
        } else {
            fabelgeist_armor::LacingEyelet::Metal {
                wire: gauge,
                color: TextileColor([80, 70, 50]),
            }
        };
        changed = true;
    }
    changed |= super::number(
        ui,
        &mut lacing.ring_radius.0,
        ClothLacing::RADIUS_RANGE,
        "Ring radius (mm)",
    );
    let minimum = lacing.ring_radius.0.max(*ClothLacing::INSET_RANGE.start());
    let inset = lacing.inset.0.max(minimum);
    changed |= inset != lacing.inset.0;
    lacing.inset.0 = inset;
    changed |= super::number(
        ui,
        &mut lacing.inset.0,
        minimum..=*ClothLacing::INSET_RANGE.end(),
        "Ring inset (mm)",
    );
    changed |= super::number(
        ui,
        &mut lacing.eyelet.gauge_mut().0,
        ClothLacing::WIRE_RANGE,
        "Eyelet thread/wire (micrometres)",
    );
    match &mut lacing.eyelet {
        fabelgeist_armor::LacingEyelet::Metal { color, .. } => {
            changed |= ui.color_edit_button_srgb(&mut color.0).changed();
        }
        fabelgeist_armor::LacingEyelet::Sewn { fabric, .. } => {
            changed |= super::textile::show(ui, fabric);
        }
    }
    changed
}
