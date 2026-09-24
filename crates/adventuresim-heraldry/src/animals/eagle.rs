//! Wernigerode-based Commons drawings, preserving their shared wing instances.
mod paint;
use super::{source::SvgDrawing, *};
use std::sync::OnceLock;

const SINGLE: &str = include_str!("../../references/eagles/single.svg");
const DOUBLE: &str = include_str!("../../references/eagles/double.svg");
const REFERENCE_STROKE: f32 = 0.004;
const ASYMMETRY_SHEAR: f32 = 0.04;

pub(super) fn draw(
    p: &mut Painter<'_>,
    heads: EagleHeads,
    armed: Tincture,
    langued: Tincture,
    facing: Facing,
) {
    static SINGLE_DRAWING: OnceLock<SvgDrawing> = OnceLock::new();
    static DOUBLE_DRAWING: OnceLock<SvgDrawing> = OnceLock::new();
    let source = match heads {
        EagleHeads::One => SINGLE_DRAWING.get_or_init(|| SvgDrawing::from_svg(SINGLE)),
        EagleHeads::Two => DOUBLE_DRAWING.get_or_init(|| SvgDrawing::from_svg(DOUBLE)),
    };
    for path in &source.paths {
        let geometry = |data: &usvg::tiny_skia_path::Path| {
            source
                .geometry(data, path.abs_transform())
                .mapped(|[x, y]| {
                    let y = y + p.style.asymmetry.0 * (x - 0.5) * ASYMMETRY_SHEAR;
                    [
                        if facing == Facing::Sinister {
                            1.0 - x
                        } else {
                            x
                        },
                        y,
                    ]
                })
        };
        let paints = paint::EaglePaint {
            body: p.fill,
            armed,
            langued,
            modeling: p.style.painted_modeling,
        };
        let shape = |path, layer: paint::Layer, rule| Shape {
            path,
            tincture: layer.tincture,
            role: layer.role,
            opacity: layer.coverage,
            stroke: 0.0,
            clips: vec![],
            rule,
        };
        let fills: Vec<_> = path
            .fill()
            .into_iter()
            .flat_map(|fill| {
                let rule = match fill.rule() {
                    usvg::FillRule::NonZero => FillRule::Winding,
                    usvg::FillRule::EvenOdd => FillRule::EvenOdd,
                };
                paints
                    .fill(fill.paint(), path.id())
                    .into_iter()
                    .map(move |layer| shape(geometry(path.data()), layer, rule))
            })
            .collect();
        let stroke = path.stroke().and_then(|s| {
            let layer = paints.stroke(s.paint(), path.id())?;
            let mut pen = s.to_tiny_skia();
            pen.width *= p.style.stroke_width.0 / REFERENCE_STROKE;
            path.data()
                .stroke(&pen, 1.0)
                .map(|outline| shape(geometry(&outline), layer, FillRule::Winding))
        });
        match path.paint_order() {
            usvg::PaintOrder::FillAndStroke => p.art.shapes.extend(fills.into_iter().chain(stroke)),
            usvg::PaintOrder::StrokeAndFill => p.art.shapes.extend(stroke.into_iter().chain(fills)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extraction_preserves_every_original_eagle_contour_and_transform() {
        for (original, extracted) in [
            (
                include_str!("../../references/eagles/single.original.svg"),
                SINGLE,
            ),
            (
                include_str!("../../references/eagles/double.original.svg"),
                DOUBLE,
            ),
        ] {
            let original = SvgDrawing::from_svg(original);
            let extracted = SvgDrawing::from_svg(extracted);
            // Only the first visible path, the source shield, is removed.
            assert_eq!(original.paths.len() - 1, extracted.paths.len());
            for (a, b) in original.paths.iter().skip(1).zip(&extracted.paths) {
                let a = original.geometry(a.data(), a.abs_transform());
                let b = extracted.geometry(b.data(), b.abs_transform());
                assert_eq!(a.svg(), b.svg());
            }
        }
    }
}
