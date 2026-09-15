//! Two authoring interfaces over the same saved, measured stock recipes.
mod picker;
use adventuresim_heraldry::{
    artwork::PaintTone,
    document::Tincture,
    paint::{Paint, PaintPalette, mixing::*},
};
use bevy_egui::egui;

pub(crate) struct Mixer {
    visible: bool,
    source: Paint,
    tincture: Tincture,
    tone: PaintTone,
    paint: MixedPaint,
    request: SearchRequest,
    result: Option<Match>,
    status: String,
}
impl Default for Mixer {
    fn default() -> Self {
        Self {
            visible: false,
            source: Paint::Mixed {
                paint: MixedPaint::uniform(StockMix::pure(PaintStock::Azurite)),
            },
            tincture: Tincture::Azure,
            tone: PaintTone::Base,
            paint: MixedPaint::uniform(StockMix::pure(PaintStock::Azurite)),
            request: SearchRequest::default(),
            result: None,
            status: String::new(),
        }
    }
}
impl Mixer {
    pub fn open(&mut self, tincture: Tincture, paint: Paint) {
        self.source = paint;
        self.tincture = tincture;
        self.tone = PaintTone::Base;
        self.visible = true;
        self.result = None;
        self.paint = match paint {
            Paint::Mixed { paint } => paint,
            Paint::Recipe { .. } => {
                let nearest = |tone| {
                    SearchRequest {
                        target_srgb: paint.color(tone),
                        tolerance: ColorTolerance(0.0),
                        workshop: Workshop::default(),
                    }
                    .solve()
                    .expect("valid nearest-color request")
                    .expect("all stocks available")
                    .mix
                };
                MixedPaint {
                    base: nearest(PaintTone::Base),
                    shadow: nearest(PaintTone::Shadow),
                    highlight: nearest(PaintTone::Highlight),
                }
            }
        };
        self.request.target_srgb = paint.color(self.tone);
        self.status =
            "Choose recipes for all three tones; Apply saves the measured paint to this tincture."
                .into();
    }
    pub fn show(&mut self, ctx: &egui::Context, palette: &mut PaintPalette) {
        if self.visible && palette[self.tincture] != self.source {
            self.visible = false;
        }
        if !self.visible {
            return;
        }
        let mut visible = true;
        egui::Window::new(format!("Measured paint mixer · {:?}", self.tincture)).open(&mut visible).default_width(640.0).default_height(850.0).vscroll(true).show(ctx, |ui| {
            ui.label("Gum Arabic on parchment · D65 daylight / 2° observer");
            ui.weak("Reference-paint study. Shield-ground transfer and roughness are estimates. This limited palette is not the full historical gamut.");
            ui.horizontal(|ui| {
                for tone in PaintTone::ALL {
                    if ui.selectable_value(&mut self.tone, tone, format!("{tone:?}")).changed() {
                        self.request.target_srgb = self.paint.tone_mut(tone).color();
                        self.result = None;
                    }
                }
            });
            ui.separator();
            let mix = self.paint.tone_mut(self.tone);
            if picker::ingredients(ui, mix) { self.result = None; }
            picker::gamut(ui, mix, &self.request.workshop);
            picker::swatch(ui, "Selected paint", mix.color());
            ui.small(if mix.measured_knot() { "Measured specimen knot" } else { "Interpolated between measured specimen knots" });
            let old_workshop = self.request.workshop.clone();
            ui.collapsing("Batch, stock availability and prices", |ui| picker::workshop(ui, &mut self.request.workshop));
            if old_workshop != self.request.workshop { self.result = None; }
            match self.request.workshop.quote(*mix) {
                Ok(Some(quote)) => {
                    ui.label(format!("This tone's batch: {:.3} cost units", quote.cost_units()));
                    for (stock, ml) in PaintStock::ALL.into_iter().zip(quote.stock_milliliters) {
                        if ml > 0.0 { ui.small(format!("{}: {ml:.4} mL prepared paint", stock.label())); }
                    }
                }
                _ => { ui.colored_label(egui::Color32::YELLOW, "Selected batch is unavailable or over budget."); }
            }
            ui.separator();
            self.search(ui);
            ui.separator();
            ui.label(&self.status);
            let can_apply = [self.paint.base, self.paint.shadow, self.paint.highlight].into_iter().all(|m| self.request.workshop.quote(m).is_ok_and(|q| q.is_some()));
            if ui.add_enabled(can_apply, egui::Button::new("Apply all three paint recipes")).clicked() {
                palette[self.tincture] = Paint::Mixed { paint: self.paint };
                self.source = palette[self.tincture];
                self.status = "Saved ingredient recipes. Batch prices remain a workshop scenario, separate from the object.".into();
            }
            if !can_apply { ui.weak("Each tone's batch must be available and within budget before applying."); }
            ui.collapsing("Sources and limits", |ui| {
                ui.hyperlink_to("Reichert et al. · measured paint specimens, CC BY 4.0", "https://doi.org/10.6084/m9.figshare.28639103.v3");
                ui.hyperlink_to("CIE colorimetry tables · CC BY-SA 4.0", "https://cie.co.at/data-tables");
                ui.label("White shares are volumes of prepared paint, not pigment mass. Only pure, half-white and white endpoints were measured. Other shares are empirical interpolation; thickness and opacity were not calibrated.");
                ui.label("Grape-seed black is gray in this preparation. Lead-tin yellow's midpoint is slightly darker than its pure sample. The source's separate Lab spreadsheet differs from spectral integration; predictive accuracy is not established.");
                ui.label("Quotes cover a chosen wet-paint batch, not this entire shield. Prices are editable illustrative units; binder is already in the stock. Cloth dyeing needs a separate model.");
            });
        });
        self.visible = visible;
    }
    fn search(&mut self, ui: &mut egui::Ui) {
        let old_request = self.request.clone();
        ui.strong("Find a recipe for a requested color");
        ui.horizontal(|ui| {
            ui.color_edit_button_srgb(&mut self.request.target_srgb);
            ui.label("Requested sRGB (not saved as paint)");
        });
        ui.add(
            egui::Slider::new(&mut self.request.tolerance.0, 0.0..=30.0).text("Tolerance · ΔE76"),
        );
        if self.request != old_request {
            self.result = None;
        }
        if ui.button("Find least-cost recipe").clicked() {
            match self.request.solve() {
                Ok(result) => {
                    self.result = result;
                    self.status = if self.result.is_some() {
                        "Search complete over every supported 0.1% mixture."
                    } else {
                        "No recipe is available within this batch budget."
                    }
                    .into();
                }
                Err(error) => {
                    self.result = None;
                    self.status = error.to_string();
                }
            }
        }
        if let Some(result) = &self.result {
            picker::swatch(ui, "Achievable", result.preview_srgb);
            ui.label(format!(
                "{} · {:.1}% white · ΔE76 {:.3} · {:.3} cost units",
                result.mix.stock().label(),
                f64::from(result.mix.shares()[PaintStock::LeadWhite.index()]) / 10.0,
                result.delta_e76,
                result.quote.cost_units()
            ));
            let label = match result.status {
                MatchStatus::WithinTolerance => "Use cheapest matching recipe",
                MatchStatus::NearestOutsideTolerance => {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        "No match within tolerance; this is the nearest feasible color.",
                    );
                    "Use nearest despite mismatch"
                }
            };
            if ui.button(label).clicked() {
                *self.paint.tone_mut(self.tone) = result.mix;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opening_does_not_edit_and_external_replacement_discards_the_draft() {
        let mut palette = PaintPalette::default();
        let original = palette;
        let mut mixer = Mixer::default();
        mixer.open(Tincture::Azure, palette[Tincture::Azure]);
        assert_eq!(palette, original);
        palette[Tincture::Azure] = Paint::Recipe {
            recipe: adventuresim_heraldry::paint::PaintRecipe::IndigoWhiteSize,
        };
        let changed = palette;
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            mixer.show(ui.ctx(), &mut palette)
        });
        assert!(!mixer.visible);
        assert_eq!(palette, changed);
    }
}
