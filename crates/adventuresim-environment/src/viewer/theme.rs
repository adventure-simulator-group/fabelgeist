//! The viewer's look: dark glass panels floating over the scene, large type
//! and one leaf-green accent, as a game's HUD.

use bevy_egui::egui::{
    self, Color32, Context, CornerRadius, FontFamily, FontId, Frame, Margin, Shadow, Stroke,
    TextStyle,
};

/// The accent: what is chosen, on, or growing.
pub const ACCENT: Color32 = Color32::from_rgb(132, 204, 96);
/// A panel's fill: dark, a little green, and see-through.
pub const GLASS: Color32 = Color32::from_rgba_premultiplied(10, 14, 11, 236);
/// A raised element inside a panel: a slot, a card, a button.
pub const RAISED: Color32 = Color32::from_rgba_premultiplied(30, 38, 32, 230);
/// The same, under the pointer.
pub const HOVERED: Color32 = Color32::from_rgba_premultiplied(46, 58, 48, 240);
/// The scene behind the menu, dimmed.
pub const BACKDROP: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 120);
/// A panel's faint edge.
pub const EDGE: Color32 = Color32::from_rgba_premultiplied(40, 46, 40, 46);
pub const TEXT: Color32 = Color32::from_rgb(234, 240, 230);
pub const MUTED: Color32 = Color32::from_rgb(156, 170, 152);
/// What a click would destroy.
pub const DANGER: Color32 = Color32::from_rgb(236, 112, 84);
/// Space between a panel and the window's edge, logical pixels.
pub const GUTTER: f32 = 16.0;
/// A panel's corner radius, logical pixels.
pub const ROUNDING: u8 = 14;
/// A raised element's corner radius, logical pixels.
pub const ROUNDING_SMALL: u8 = 10;

/// Style every egui context the viewer draws with.
pub fn apply(ctx: &Context) {
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(15.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(15.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Heading,
                FontId::new(24.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            ),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 7.0);
        style.spacing.interact_size.y = 26.0;
        style.spacing.slider_width = 300.0;
        let visuals = &mut style.visuals;
        visuals.dark_mode = true;
        visuals.override_text_color = Some(TEXT);
        visuals.window_fill = GLASS;
        visuals.panel_fill = GLASS;
        visuals.window_stroke = Stroke::new(1.0_f32, EDGE);
        visuals.window_corner_radius = CornerRadius::same(ROUNDING);
        visuals.selection.bg_fill = ACCENT.gamma_multiply(0.55);
        visuals.selection.stroke = Stroke::new(1.5_f32, ACCENT);
        visuals.slider_trailing_fill = true;
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.corner_radius = CornerRadius::same(ROUNDING_SMALL);
            widget.bg_stroke = Stroke::NONE;
        }
        visuals.widgets.inactive.weak_bg_fill = RAISED;
        visuals.widgets.inactive.bg_fill = RAISED;
        visuals.widgets.hovered.weak_bg_fill = HOVERED;
        visuals.widgets.hovered.bg_fill = HOVERED;
        visuals.widgets.active.weak_bg_fill = ACCENT.gamma_multiply(0.6);
        visuals.widgets.active.bg_fill = ACCENT;
    });
}

/// A floating panel.
pub fn panel() -> Frame {
    Frame::new()
        .fill(GLASS)
        .stroke(Stroke::new(1.0_f32, EDGE))
        .corner_radius(ROUNDING)
        .inner_margin(Margin::symmetric(14, 10))
        .shadow(Shadow {
            offset: [0, 6],
            blur: 18,
            spread: 0,
            color: Color32::from_black_alpha(80),
        })
}
