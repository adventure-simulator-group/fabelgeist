//! Native picker and slider boundaries for authored encoded material values.

use fabelgeist_color::{Srgb, TransferFunction};
use fabelgeist_math::units::Ratio;
use std::ops::RangeInclusive;

pub(super) fn color(ui: &mut egui::Ui, color: &mut Srgb) -> egui::Response {
    let mut linear = color.linear_channels();
    let response = ui.color_edit_button_rgb(&mut linear);
    if response.changed() {
        *color = Srgb::from_encoded_channels(
            TransferFunction::Srgb.from_linear_rgb(linear),
        );
    }
    response
}

pub(super) fn roughness(
    ui: &mut egui::Ui,
    label: &str,
    roughness: &mut Ratio,
    range: RangeInclusive<f32>,
) -> egui::Response {
    let mut native = roughness.ratio();
    let response = ui.add(egui::Slider::new(&mut native, range).text(label));
    if response.changed() {
        *roughness = Ratio::from_ratio(native);
    }
    response
}
