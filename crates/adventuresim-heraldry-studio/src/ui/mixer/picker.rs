//! Reachable colors are curves, not a filled RGB triangle or convex hull.
use adventuresim_heraldry::paint::mixing::*;
use bevy_egui::egui;
const STRIP_SAMPLES: u16 = 80;

pub(super) fn swatch(ui: &mut egui::Ui, label: &str, rgb: [u8; 3]) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(42.0, 24.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, 3.0, egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        ui.label(label);
        ui.monospace(format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]));
    });
}
pub(super) fn ingredients(ui: &mut egui::Ui, mix: &mut StockMix) -> bool {
    let before = *mix;
    let mut stock = mix.stock();
    egui::ComboBox::from_id_salt("stock")
        .selected_text(stock.label())
        .show_ui(ui, |ui| {
            for choice in PaintStock::ALL {
                ui.selectable_value(&mut stock, choice, choice.label());
            }
        });
    let mut white = if stock.supports_tints() {
        mix.white_permille()
    } else {
        0
    };
    ui.add_enabled(
        stock.supports_tints(),
        egui::Slider::new(&mut white, 0..=WHITE_SCALE).text("Added white stock · ‰ by volume"),
    );
    *mix = StockMix::new(stock, white).expect("bounded stock controls");
    before != *mix
}
pub(super) fn gamut(ui: &mut egui::Ui, mix: &mut StockMix, workshop: &Workshop) {
    egui::CollapsingHeader::new("Reachable colors · click a strip").default_open(true).show(ui, |ui| {
        ui.weak("Tint families run from pure stock to pure white. Dimmed recipes exceed this budget or use unavailable stock.");
        for stock in PaintStock::ALL {
            ui.label(stock.label());
            let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click_and_drag());
            for i in 0..STRIP_SAMPLES {
                let white = if stock.supports_tints() { (u32::from(i) * u32::from(WHITE_SCALE) / u32::from(STRIP_SAMPLES - 1)) as u16 } else { 0 };
                let candidate = StockMix::new(stock, white).expect("bounded strip");
                let [r,g,b] = candidate.color();
                let color = egui::Color32::from_rgb(r,g,b);
                let enabled = workshop.quote(candidate).is_ok_and(|q| q.is_some());
                let tile = egui::Rect::from_x_y_ranges(rect.left()+rect.width()*f32::from(i)/f32::from(STRIP_SAMPLES)..=rect.left()+rect.width()*f32::from(i+1)/f32::from(STRIP_SAMPLES), rect.y_range());
                ui.painter().rect_filled(tile, 0.0, if enabled {color} else {color.gamma_multiply(0.25)});
            }
            if (response.clicked() || response.dragged()) && let Some(point) = response.interact_pointer_pos() {
                let white = if stock.supports_tints() { (((point.x - rect.left()) / rect.width()).clamp(0.0,1.0)*f32::from(WHITE_SCALE)).round() as u16 } else { 0 };
                let candidate = StockMix::new(stock, white).expect("bounded pointer");
                if workshop.quote(candidate).is_ok_and(|q| q.is_some()) { *mix = candidate; }
            }
        }
    });
}
pub(super) fn workshop(ui: &mut egui::Ui, workshop: &mut Workshop) {
    ui.weak("Illustrative units; all stocks start at equal prices. Enter your own price scenario. Gum binder is already included.");
    let mut ml = f64::from(workshop.batch_microliters.0) / 1000.0;
    ui.horizontal(|ui| {
        ui.label("Batch mL (for each tone)");
        ui.add(
            egui::DragValue::new(&mut ml)
                .range(0.001..=1000.0)
                .speed(0.1),
        );
    });
    workshop.batch_microliters = BatchVolume((ml * 1000.0).round() as u32);
    ui.horizontal(|ui| {
        ui.label("Preparation cost per batch");
        ui.add(egui::DragValue::new(&mut workshop.setup_cost.0).range(0..=1_000_000));
    });
    let mut budget = workshop.budget_units.is_some();
    if ui.checkbox(&mut budget, "Limit batch cost").changed() {
        workshop.budget_units = budget.then_some(100);
    }
    if let Some(limit) = &mut workshop.budget_units {
        ui.add(
            egui::DragValue::new(limit)
                .range(0..=1_000_000)
                .suffix(" units"),
        );
    }
    for stock in PaintStock::ALL {
        ui.horizontal(|ui| {
            ui.checkbox(&mut workshop.available[stock.index()], stock.label());
            ui.add(
                egui::DragValue::new(&mut workshop.prices_per_ml[stock.index()].0)
                    .range(0..=1_000_000)
                    .suffix(" units/mL"),
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_complete_reachable_palette_renders_without_overflow() {
        let ctx = egui::Context::default();
        let mut mix = StockMix::pure(PaintStock::Azurite);
        let before = mix;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            gamut(ui, &mut mix, &Workshop::default());
        });
        assert!(
            output.shapes.len() > 100,
            "all six open color strips must render"
        );
        assert_eq!(mix, before, "drawing the picker does not select a paint");
    }
}
