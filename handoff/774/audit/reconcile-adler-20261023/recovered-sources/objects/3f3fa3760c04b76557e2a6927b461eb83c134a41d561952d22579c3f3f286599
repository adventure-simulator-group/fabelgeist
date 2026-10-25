//! Controls for how a plate-steel piece is built: one solid plate, or
//! lamellar lames or scales laced over its surface.
use super::*;
use fabelgeist_armor::{Construction, Lacing, Plate, Tiling};

/// Metres shown in millimetres.
const MILLIMETRES_PER_METRE: f32 = 1000.0;

/// Returns whether the construction changed.
pub(super) fn edit(ui: &mut egui::Ui, construction: &mut Construction) -> bool {
    let before = construction.clone();
    ui.horizontal(|ui| {
        ui.label("Construction");
        for choice in [
            Construction::Solid,
            Construction::Lamellar(Tiling::lamellar()),
            Construction::Scale(Tiling::scale()),
        ] {
            let chosen = std::mem::discriminant(construction) == std::mem::discriminant(&choice);
            // Switching starts the small plates from the construction's usual shape.
            if ui.selectable_label(chosen, choice.name()).clicked() && !chosen {
                *construction = choice;
            }
        }
    })
    .response
    .on_hover_text("Small plates are laid over the fitted piece in overlapping rows.");
    if let Some(tiling) = construction.tiling_mut() {
        ui.collapsing("Small plates", |ui| plate(ui, &mut tiling.plate));
        lacing(ui, tiling);
    }
    if let Err(error) = construction.validate() {
        ui.colored_label(egui::Color32::LIGHT_RED, error.to_string());
    }
    *construction != before
}

fn plate(ui: &mut egui::Ui, plate: &mut Plate) {
    millimetres(ui, "Width (mm)", &mut plate.width, Plate::WIDTH);
    millimetres(ui, "Height (mm)", &mut plate.height, Plate::HEIGHT);
    millimetres(ui, "Thickness (mm)", &mut plate.thickness, Plate::THICKNESS);
    millimetres(ui, "Gap (mm)", &mut plate.gap, Plate::GAP);
    let bevel = 0.0..=plate.thickness * Plate::MAX_BEVEL;
    plate.bevel = plate.bevel.min(*bevel.end());
    millimetres(ui, "Edge bevel (mm)", &mut plate.bevel, bevel);
    metal_controls::slider(ui, "Rounded foot", &mut plate.roundness, 0.0..=1.0);
    metal_controls::slider(ui, "Row overlap", &mut plate.overlap, Plate::OVERLAP);
    metal_controls::slider(ui, "Row stagger", &mut plate.stagger, 0.0..=1.0);
    ui.add(egui::Slider::new(&mut plate.hole_pairs, 0..=Plate::MAX_HOLE_PAIRS).text("Hole pairs"));
    let holes = 0.0..=plate.max_hole_radius();
    plate.hole_radius = plate.hole_radius.min(*holes.end());
    millimetres(ui, "Hole radius (mm)", &mut plate.hole_radius, holes);
}

fn lacing(ui: &mut egui::Ui, tiling: &mut Tiling) {
    let mut laced = tiling.lacing.is_some();
    let holes =
        tiling.plate.hole_pairs > 0 && Lacing::max_radius(&tiling.plate) >= *Lacing::RADIUS.start();
    ui.add_enabled(holes, egui::Checkbox::new(&mut laced, "Laced"))
        .on_disabled_hover_text("Lacing runs through the plates' holes; widen them.");
    tiling.lacing = match (laced && holes, tiling.lacing.take()) {
        (false, _) => None,
        (true, lacing) => Some(lacing.unwrap_or_default()),
    };
    let Some(lacing) = tiling.lacing.as_mut() else {
        return;
    };
    let radius = *Lacing::RADIUS.start()..=Lacing::max_radius(&tiling.plate);
    lacing.radius = lacing.radius.clamp(*radius.start(), *radius.end());
    millimetres(ui, "Cord radius (mm)", &mut lacing.radius, radius);
    ui.horizontal(|ui| {
        ui.label("Cord colour");
        ui.color_edit_button_rgb(&mut lacing.color);
    });
    metal_controls::slider(ui, "Cord roughness", &mut lacing.roughness, 0.0..=1.0);
}

/// A slider over a length in metres, shown in millimetres.
fn millimetres(
    ui: &mut egui::Ui,
    label: &str,
    metres: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) {
    let mut shown = *metres * MILLIMETRES_PER_METRE;
    let range = range.start() * MILLIMETRES_PER_METRE..=range.end() * MILLIMETRES_PER_METRE;
    if ui
        .add(egui::Slider::new(&mut shown, range).text(label))
        .changed()
    {
        *metres = shown / MILLIMETRES_PER_METRE;
    }
}
