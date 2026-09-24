//! Sourced lion artwork and geometric charges share composition and paint roles.
mod eagle;
mod lion;
mod source;
use crate::{artwork::*, document::*};
pub(crate) struct Painter<'a> {
    pub art: Artwork,
    pub style: &'a DrawingStyle,
    pub fill: Tincture,
    pub ink: Tincture,
}
impl Painter<'_> {
    pub fn plate(&mut self, path: Path) {
        self.art.shapes.push(Shape {
            path: path.clone(),
            tincture: self.fill,
            stroke: 0.0,
            clips: vec![],
            role: PaintRole::Charge,
            opacity: Ratio(1.0),
            rule: FillRule::Winding,
        });
        if self.fill != Tincture::Sable {
            self.art.shapes.push(Shape {
                path,
                tincture: self.ink,
                stroke: self.style.stroke_width.0 * 0.55,
                clips: vec![],
                role: PaintRole::Accent,
                opacity: Ratio(1.0),
                rule: FillRule::Winding,
            });
        }
    }
}
pub(crate) fn draw(c: &Charge, s: &DrawingStyle) -> Artwork {
    let fill = match c.color {
        Coloring::Solid { tincture } => tincture,
        Coloring::Counterchanged { tinctures } => tinctures[0],
    };
    let mut p = Painter {
        art: Artwork::default(),
        style: s,
        fill,
        ink: if fill == Tincture::Sable {
            Tincture::Or
        } else {
            Tincture::Sable
        },
    };
    match c.shape {
        ChargeKind::Lion { tails, facing } => lion::draw(&mut p, tails, c.armed, c.langued, facing),
        ChargeKind::Eagle { heads, facing } => {
            eagle::draw(&mut p, heads, c.armed, c.langued, facing)
        }
        ChargeKind::Roundel => p.plate(Path::ellipse(0.5, 0.5, 0.46, 0.46)),
        ChargeKind::Lozenge => p.plate(
            Path::new(0.5, 0.02)
                .line(0.96, 0.5)
                .line(0.5, 0.98)
                .line(0.04, 0.5)
                .close(),
        ),
        ChargeKind::Star { points } => {
            let mut path = Path::default();
            for i in 0..points * 2 {
                let a =
                    i as f32 * std::f32::consts::PI / points as f32 - std::f32::consts::FRAC_PI_2;
                let r = if i % 2 == 0 { 0.48 } else { 0.21 };
                let point = [0.5 + r * a.cos(), 0.5 + r * a.sin()];
                if i == 0 {
                    path = Path::new(point[0], point[1]);
                } else {
                    path = path.line(point[0], point[1]);
                }
            }
            p.plate(path.close());
        }
    }
    p.art
}
