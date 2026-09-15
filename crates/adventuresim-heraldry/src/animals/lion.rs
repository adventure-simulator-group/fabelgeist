//! German c.1530 drawing with independently classified pigment modeling.
//! Original SVG and attribution: references/ATTRIBUTION.md.
mod rig;
mod source;
use super::*;

const REFERENCE_STROKE: f32 = 0.004;

pub(super) fn draw(
    p: &mut Painter<'_>,
    tails: LionTails,
    crowned: bool,
    armed: Tincture,
    langued: Tincture,
    facing: Facing,
) {
    if tails == LionTails::Two {
        draw_source(p, armed, langued, Some(rig::tail_clip()), rig::second_tail);
    }
    draw_source(p, armed, langued, None, |q| q);
    if crowned {
        let center = rig::deform([0.575, 0.045], p.style);
        p.crown(center, 0.13 * p.style.lion.head_size.0);
    }
    if facing == Facing::Sinister {
        for shape in &mut p.art.shapes {
            shape.path = shape.path.mapped(|[x, y]| [1.0 - x, y]);
            for clip in &mut shape.clips {
                *clip = clip.mapped(|[x, y]| [1.0 - x, y]);
            }
        }
    }
}
fn draw_source(
    p: &mut Painter<'_>,
    armed: Tincture,
    langued: Tincture,
    clip: Option<Path>,
    variant: impl Fn([f32; 2]) -> [f32; 2],
) {
    let map = |q| variant(rig::deform(q, p.style));
    let clips: Vec<_> = clip.into_iter().map(|c| c.mapped(map)).collect();
    for path in source::paths() {
        let geometry = |data: &usvg::tiny_skia_path::Path| {
            source::convert(data, path.abs_transform()).mapped(map)
        };
        let mut fills = Vec::new();
        if let Some(fill) = path.fill() {
            let usvg::Paint::Color(c) = fill.paint() else {
                unreachable!("vendored German lion uses solid fills")
            };
            let rgb = [c.red, c.green, c.blue];
            let (tincture, role) = color(rgb, path.id(), p, armed, langued);
            let mut shape = Shape {
                path: geometry(path.data()),
                tincture,
                role,
                opacity: Ratio(if role == PaintRole::Highlight {
                    p.style.painted_modeling.highlights.0
                } else {
                    1.0
                }),
                stroke: 0.0,
                clips: clips.clone(),
                rule: match fill.rule() {
                    usvg::FillRule::NonZero => FillRule::Winding,
                    usvg::FillRule::EvenOdd => FillRule::EvenOdd,
                },
            };
            if shape.opacity.0 > 0.0 {
                fills.push(shape.clone());
            }
            // The source dark underpainting is also the complete silhouette.
            // Retain an opaque base under it, including with flat paint.
            if rgb == [219, 186, 46] && p.style.painted_modeling.shadows.0 > 0.0 {
                shape.role = PaintRole::Shadow;
                shape.opacity = p.style.painted_modeling.shadows;
                fills.push(shape);
            }
        }
        let stroke = path.stroke().and_then(|s| {
            let mut pen = s.to_tiny_skia();
            pen.width *= p.style.stroke_width.0 / REFERENCE_STROKE;
            path.data().stroke(&pen, 1.0).map(|outline| Shape {
                path: geometry(&outline),
                tincture: p.ink,
                role: PaintRole::Accent,
                opacity: Ratio(1.0),
                stroke: 0.0,
                clips: clips.clone(),
                rule: FillRule::Winding,
            })
        });
        match path.paint_order() {
            usvg::PaintOrder::FillAndStroke => p.art.shapes.extend(fills.into_iter().chain(stroke)),
            usvg::PaintOrder::StrokeAndFill => p.art.shapes.extend(stroke.into_iter().chain(fills)),
        }
    }
}
fn color(
    rgb: [u8; 3],
    id: &str,
    p: &Painter<'_>,
    armed: Tincture,
    langued: Tincture,
) -> (Tincture, PaintRole) {
    match rgb {
        [219, 186, 46] | [255, 219, 67] => (p.fill, PaintRole::Charge),
        [255, 234, 128] => (p.fill, PaintRole::Highlight),
        [233, 69, 69] => (
            if id == "path5209" { langued } else { armed },
            PaintRole::Accent,
        ),
        [255, 255, 255] => (Tincture::Argent, PaintRole::Accent),
        [158, 120, 0] => (p.ink, PaintRole::Accent),
        [0, 0, 0] => (Tincture::Sable, PaintRole::Accent),
        _ => unreachable!("vendored German lion paint changed: {rgb:?}"),
    }
}
