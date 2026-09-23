//! The studio's look: a dark, warm palette with gold accents, the project's
//! typeface, Game Icons, and the few widgets the character screens share.
use bevy::prelude::Local;
use bevy_egui::{EguiContexts, egui};
use std::sync::{Arc, LazyLock};

/// The deepest background, behind text fields and the viewport's edges.
pub(crate) const INK: egui::Color32 = egui::Color32::from_rgb(10, 11, 14);
/// The side panel, faintly translucent over the scene.
pub(crate) const PANEL: egui::Color32 = egui::Color32::from_rgba_premultiplied(14, 15, 19, 244);
/// Cards and other grouped content.
pub(crate) const SURFACE: egui::Color32 = egui::Color32::from_rgb(24, 25, 30);
/// Buttons and slider rails, a step above a card.
pub(crate) const RAISED: egui::Color32 = egui::Color32::from_rgb(38, 38, 44);
/// Hairlines between regions.
pub(crate) const LINE: egui::Color32 = egui::Color32::from_rgb(52, 48, 42);
/// Resting borders on interactive widgets.
pub(crate) const BRONZE: egui::Color32 = egui::Color32::from_rgb(98, 80, 52);
/// The accent: selected tabs, headings and values.
pub(crate) const GOLD: egui::Color32 = egui::Color32::from_rgb(214, 178, 106);
/// The accent under the pointer.
pub(crate) const GOLD_BRIGHT: egui::Color32 = egui::Color32::from_rgb(244, 214, 150);
/// Filled selections.
const GOLD_DEEP: egui::Color32 = egui::Color32::from_rgb(118, 90, 46);
/// Body text.
pub(crate) const PARCHMENT: egui::Color32 = egui::Color32::from_rgb(230, 222, 206);
/// Secondary text.
pub(crate) const MUTED: egui::Color32 = egui::Color32::from_rgb(146, 138, 124);
/// Problems and failures.
pub(crate) const DANGER: egui::Color32 = egui::Color32::from_rgb(228, 116, 100);

/// Width of borders and rules, in points.
pub(crate) const HAIRLINE: f32 = 1.0;
const CORNER: u8 = 6;
const CARD_CORNER: u8 = 8;

/// The large display style used for the character's name.
pub(crate) fn display_font() -> egui::FontId {
    egui::FontId::proportional(30.0)
}

/// Install the font, style and image loaders once, before the first panel.
pub(crate) fn install(mut contexts: EguiContexts, mut installed: Local<bool>) {
    if *installed {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    ctx.set_fonts(fonts());
    ctx.all_styles_mut(style);
    egui_extras::install_image_loaders(ctx);
    *installed = true;
}

fn fonts() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "sn-pro".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/SNPro-VariableFont_wght.ttf"
        ))),
    );
    // egui's own fonts stay behind SN Pro for the symbols it lacks.
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "sn-pro".into());
    fonts
}

fn style(style: &mut egui::Style) {
    use egui::{FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Small, FontId::proportional(12.5)),
        (TextStyle::Body, FontId::proportional(15.0)),
        (TextStyle::Button, FontId::proportional(15.0)),
        (TextStyle::Heading, FontId::proportional(21.0)),
        (TextStyle::Monospace, FontId::monospace(13.0)),
    ]
    .into();
    let spacing = &mut style.spacing;
    spacing.item_spacing = egui::vec2(8.0, 7.0);
    spacing.button_padding = egui::vec2(12.0, 5.0);
    spacing.interact_size.y = 24.0;
    spacing.slider_rail_height = 6.0;
    spacing.indent = 16.0;
    spacing.scroll = egui::style::ScrollStyle::floating();
    style.visuals = visuals();
}

fn visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PANEL;
    visuals.window_fill = SURFACE;
    visuals.window_stroke = egui::Stroke::new(HAIRLINE, BRONZE);
    visuals.window_corner_radius = CARD_CORNER.into();
    visuals.menu_corner_radius = CORNER.into();
    visuals.extreme_bg_color = INK;
    visuals.faint_bg_color = SURFACE;
    visuals.code_bg_color = INK;
    visuals.hyperlink_color = GOLD;
    visuals.error_fg_color = DANGER;
    visuals.warn_fg_color = GOLD_BRIGHT;
    visuals.selection.bg_fill = GOLD_DEEP;
    visuals.selection.stroke = egui::Stroke::new(HAIRLINE, PARCHMENT);
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = egui::style::HandleShape::Circle;
    visuals.indent_has_left_vline = false;
    visuals.collapsing_header_frame = false;

    let widget = |fill, border, text| egui::style::WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: egui::Stroke::new(HAIRLINE, border),
        corner_radius: CORNER.into(),
        fg_stroke: egui::Stroke::new(HAIRLINE, text),
        expansion: 0.0,
    };
    let widgets = &mut visuals.widgets;
    widgets.noninteractive = widget(SURFACE, LINE, PARCHMENT);
    widgets.inactive = widget(RAISED, LINE, PARCHMENT);
    widgets.hovered = egui::style::WidgetVisuals {
        expansion: 1.0,
        ..widget(egui::Color32::from_rgb(54, 49, 42), GOLD, GOLD_BRIGHT)
    };
    widgets.active = widget(
        egui::Color32::from_rgb(76, 62, 40),
        GOLD_BRIGHT,
        GOLD_BRIGHT,
    );
    widgets.open = widgets.active;
    visuals
}

/// Uppercase, letter-spaced gold text for section labels.
pub(crate) fn caps(text: &str, size: f32, color: egui::Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        &text.to_uppercase(),
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(size),
            color,
            extra_letter_spacing: size * 0.14,
            ..Default::default()
        },
    );
    job
}

/// A framed group of controls under a small gold title.
pub(crate) fn card<R>(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(SURFACE)
        .stroke(egui::Stroke::new(HAIRLINE, LINE))
        .corner_radius(CARD_CORNER)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(caps(title, 12.0, GOLD));
            ui.add_space(2.0);
            add_contents(ui)
        })
        .inner
}

/// Height of a stat slider's track and handle; egui sizes the handle from it.
const STAT_SLIDER_HEIGHT: f32 = 16.0;
/// egui's slider handle radius as a fraction of the slider's height.
const SLIDER_HANDLE_FRACTION: f32 = 1.0 / 2.5;

/// A labelled slider across the full width, its value beside the label and
/// its fill growing from `neutral` toward the value. Double-clicking the
/// slider returns it to `neutral`.
pub(crate) fn stat_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    neutral: f32,
) -> egui::Response {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        let off_neutral = (*value - neutral).abs() > f32::EPSILON;
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(label).color(PARCHMENT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let text = egui::RichText::new(format!("{value:+.2}"));
                ui.label(text.color(if off_neutral { GOLD } else { MUTED }));
            });
        });
        ui.spacing_mut().slider_width = ui.available_width();
        ui.spacing_mut().interact_size.y = STAT_SLIDER_HEIGHT;
        let (low, high) = (*range.start(), *range.end());
        let mut response = ui
            .add(
                egui::Slider::new(value, range)
                    .show_value(false)
                    .trailing_fill(false),
            )
            .on_hover_text("Double-click to reset");
        if response.double_clicked() {
            *value = neutral;
            response.mark_changed();
        }
        let rect = response.rect;
        let handle = rect.height() * SLIDER_HANDLE_FRACTION;
        let track = rect.x_range().shrink(handle);
        let x = |v: f32| egui::lerp(track, (v - low) / (high - low));
        let (from, to) = (x(neutral), x(*value));
        let rail = ui.spacing().slider_rail_height;
        let bar = egui::Rect::from_x_y_ranges(
            from.min(to)..=from.max(to),
            egui::Rangef::point(rect.center().y).expand(rail * 0.5),
        );
        // The fill covers the rail, so the handle is drawn again above it.
        let painter = ui.painter();
        painter.rect_filled(bar, rail * 0.5, GOLD.gamma_multiply(0.7));
        let visuals = ui.style().interact(&response);
        painter.circle(
            egui::pos2(to, rect.center().y),
            handle + visuals.expansion,
            visuals.bg_fill,
            visuals.fg_stroke,
        );
        ui.add_space(4.0);
        response
    })
    .inner
}

/// A button filled with the accent, for the one action a screen leads to.
pub(crate) fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(INK).strong())
        .fill(GOLD)
        .stroke(egui::Stroke::new(HAIRLINE, GOLD_BRIGHT))
}

/// A Game Icons glyph from the vendored collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Icon {
    Person,
    Knapsack,
    Anvil,
    OpenBook,
}

/// The glyph's SVG with its `currentColor` fill made white, so an image tint
/// colors it.
macro_rules! white_icon {
    ($file:literal) => {{
        static SVG: LazyLock<Vec<u8>> = LazyLock::new(|| {
            include_str!(concat!("../../strategic-web/static/icons/game/", $file))
                .replace("currentColor", "#ffffff")
                .into_bytes()
        });
        (concat!("bytes://game-icons/", $file), SVG.as_slice())
    }};
}

impl Icon {
    pub(crate) fn image(self) -> egui::Image<'static> {
        let (uri, svg): (&'static str, &'static [u8]) = match self {
            Self::Person => white_icon!("person.svg"),
            Self::Knapsack => white_icon!("knapsack.svg"),
            Self::Anvil => white_icon!("anvil.svg"),
            Self::OpenBook => white_icon!("open-book.svg"),
        };
        egui::Image::new(egui::ImageSource::Bytes {
            uri: uri.into(),
            bytes: egui::load::Bytes::Static(svg),
        })
    }
}
