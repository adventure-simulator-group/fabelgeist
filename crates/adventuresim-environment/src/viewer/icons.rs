//! Icons painted from shapes, so they need no icon font: the plants to sow,
//! the tools, and the time controls.

use bevy_egui::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};
use fabelgeist_botany::species::Foliage;
use fabelgeist_botany::viewer::sim::SpeciesInfo;

/// A plant of `species`, in its leaf and bark colours.
pub fn plant(painter: &Painter, rect: Rect, species: &SpeciesInfo) {
    let leaf = species_color(species.leaf);
    let bark = species_color(species.bark);
    let size = rect.width().min(rect.height());
    let center = rect.center();
    let base = pos2(center.x, center.y + size * 0.46);
    match species.foliage {
        Foliage::Needles => conifer(painter, base, size, leaf, bark),
        Foliage::Broadleaf => broadleaf(painter, base, size, leaf, bark),
        Foliage::Fronds => palm(painter, base, size, leaf, bark),
    }
}

/// Three tiers of needles over a short trunk.
fn conifer(painter: &Painter, base: Pos2, size: f32, leaf: Color32, bark: Color32) {
    trunk(painter, base, size * 0.08, size * 0.2, bark);
    for tier in 0..3 {
        let tier = tier as f32;
        let bottom = base.y - size * (0.14 + tier * 0.22);
        let half = size * (0.36 - tier * 0.08);
        let top = bottom - size * 0.36;
        painter.add(Shape::convex_polygon(
            vec![
                pos2(base.x - half, bottom),
                pos2(base.x + half, bottom),
                pos2(base.x, top),
            ],
            leaf,
            Stroke::NONE,
        ));
    }
}

/// A round crown of three clumps on a trunk.
fn broadleaf(painter: &Painter, base: Pos2, size: f32, leaf: Color32, bark: Color32) {
    trunk(painter, base, size * 0.09, size * 0.42, bark);
    let crown = pos2(base.x, base.y - size * 0.6);
    painter.circle_filled(crown + vec2(-size * 0.17, size * 0.06), size * 0.2, leaf);
    painter.circle_filled(crown + vec2(size * 0.17, size * 0.06), size * 0.2, leaf);
    painter.circle_filled(crown + vec2(0.0, -size * 0.08), size * 0.25, leaf);
}

/// A leaning stem with fronds fanning from its top.
fn palm(painter: &Painter, base: Pos2, size: f32, leaf: Color32, bark: Color32) {
    let top = pos2(base.x + size * 0.08, base.y - size * 0.62);
    painter.line_segment([base, top], Stroke::new(size * 0.07, bark));
    let reach = size * 0.36;
    for frond in [-2.6_f32, -1.9, -1.2, -0.5, 0.2] {
        let tip = top + vec2(frond.cos(), frond.sin() * 0.6 + 0.25) * reach;
        painter.line_segment([top, tip], Stroke::new(size * 0.08, leaf));
    }
}

fn trunk(painter: &Painter, base: Pos2, width: f32, height: f32, bark: Color32) {
    painter.rect_filled(
        Rect::from_min_max(
            pos2(base.x - width * 0.5, base.y - height),
            pos2(base.x + width * 0.5, base.y),
        ),
        1.0,
        bark,
    );
}

/// An eye: looking around without touching anything.
pub fn eye(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let half = rect.width().min(rect.height()) * 0.38;
    let lid = |side: f32| -> Vec<Pos2> {
        (0..=16)
            .map(|i| {
                let t = i as f32 / 16.0;
                let x = -half + 2.0 * half * t;
                let y = side * half * 0.55 * (std::f32::consts::PI * t).sin();
                center + vec2(x, y)
            })
            .collect()
    };
    let stroke = Stroke::new(2.2_f32, color);
    painter.add(Shape::line(lid(-1.0), stroke));
    painter.add(Shape::line(lid(1.0), stroke));
    painter.circle_filled(center, half * 0.3, color);
}

/// A struck-through circle: clearing the ground.
pub fn clear(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.32;
    let stroke = Stroke::new(2.4_f32, color);
    painter.circle_stroke(center, radius, stroke);
    let slash = vec2(radius, -radius) * std::f32::consts::FRAC_1_SQRT_2;
    painter.line_segment([center - slash, center + slash], stroke);
}

/// Two bars.
pub fn pause(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let height = rect.height() * 0.42;
    for side in [-1.0, 1.0] {
        let x = center.x + side * height * 0.22;
        painter.rect_filled(
            Rect::from_center_size(pos2(x, center.y), vec2(height * 0.24, height)),
            1.0,
            color,
        );
    }
}

/// `arrows` play triangles side by side: the faster, the more.
pub fn play(painter: &Painter, rect: Rect, arrows: usize, color: Color32) {
    let height = rect.height() * 0.4;
    let width = height * 0.62;
    let total = width * arrows as f32;
    let left = rect.center().x - total * 0.5;
    for arrow in 0..arrows {
        let x = left + width * arrow as f32;
        let y = rect.center().y;
        painter.add(Shape::convex_polygon(
            vec![
                pos2(x, y - height * 0.5),
                pos2(x + width, y),
                pos2(x, y + height * 0.5),
            ],
            color,
            Stroke::NONE,
        ));
    }
}

/// Three bars: the menu.
pub fn menu(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let width = rect.height() * 0.46;
    for row in [-1.0, 0.0, 1.0] {
        let y = center.y + row * width * 0.34;
        painter.line_segment(
            [
                pos2(center.x - width * 0.5, y),
                pos2(center.x + width * 0.5, y),
            ],
            Stroke::new(2.2_f32, color),
        );
    }
}

/// A species colour from botany, sRGB in 0 to 1, as egui's.
pub fn species_color(color: [f32; 3]) -> Color32 {
    let [r, g, b] = color.map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8);
    Color32::from_rgb(r, g, b)
}
