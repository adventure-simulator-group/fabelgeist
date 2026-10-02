//! Layout and counterchanging operate on the same clipped vector regions.
use crate::{artwork::*, document::*, fields};
#[derive(Clone, Copy)]
struct Frame {
    origin: [f32; 2],
    size: [f32; 2],
}
impl Frame {
    fn path(self, p: &Path) -> Path {
        p.mapped(|v| {
            [
                self.origin[0] + v[0] * self.size[0],
                self.origin[1] + v[1] * self.size[1],
            ]
        })
    }
    fn child(self, origin: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            origin: [
                self.origin[0] + origin[0] * self.size[0],
                self.origin[1] + origin[1] * self.size[1],
            ],
            size: [size[0] * self.size[0], size[1] * self.size[1]],
        }
    }
}
pub(crate) fn compose(d: &Document) -> Artwork {
    let mut art = Artwork::default();
    let frame = Frame {
        origin: [0.0; 2],
        size: [ARTBOARD; 2],
    };
    let clip = frame.path(&outline(&d.surface));
    arms(&d.arms, &d.drawing, frame, &[clip], &mut art);
    art.attribution = crate::provenance::attribution(d);
    art
}
pub(crate) fn outline(s: &PaintedSurface) -> Path {
    if s.shape == DisplayShape::Panel {
        return Path::rect(0.0, 0.0, 1.0, 1.0);
    }
    let sh = s.shoulder.0;
    let point = s.point.0;
    Path::new(0.04, 0.02)
        .curve([0.3, sh], [0.7, sh], [0.96, 0.02])
        .curve([1.0, point], [0.91, 0.76], [0.5, 0.98])
        .curve([0.09, 0.76], [0.0, point], [0.04, 0.02])
        .close()
}
fn add(art: &mut Artwork, path: Path, tincture: Tincture, clips: &[Path]) {
    art.shapes.push(Shape {
        path,
        tincture,
        stroke: 0.0,
        clips: clips.to_vec(),
        role: PaintRole::Field,
        opacity: Ratio(1.0),
        rule: FillRule::Winding,
    });
}
fn arms(a: &ArmsDesign, style: &DrawingStyle, f: Frame, clips: &[Path], art: &mut Artwork) {
    field(&a.field, style, f, clips, art, true);
    let mut background = Artwork::default();
    if a.charges
        .iter()
        .any(|c| matches!(c.color, Coloring::Counterchanged { .. }))
    {
        field(&a.field, style, f, clips, &mut background, false);
    }
    let fields = background.shapes;
    for o in &a.ordinaries {
        if o.kind == OrdinaryKind::Bordure {
            bordure(o, f, clips, art);
        } else {
            for p in fields::ordinary(o) {
                add(art, f.path(&p), o.tincture, clips);
                art.shapes.last_mut().unwrap().role = PaintRole::Charge;
            }
        }
    }
    for c in &a.charges {
        charge(c, style, f, clips, &fields, art);
    }
    if let Some(inset) = &a.inescutcheon {
        let f = f.child([0.34, 0.32], [0.32, 0.38]);
        let mut local = clips.to_vec();
        let shape = Path::new(0.0, 0.0)
            .line(1.0, 0.0)
            .curve([1.0, 0.6], [0.85, 0.85], [0.5, 1.0])
            .curve([0.15, 0.85], [0.0, 0.6], [0.0, 0.0])
            .close();
        let edge = f.path(&shape);
        local.push(edge.clone());
        arms(inset, style, f, &local, art);
        art.shapes.push(Shape {
            path: edge,
            tincture: Tincture::Sable,
            stroke: 2.0,
            clips: clips.to_vec(),
            role: PaintRole::Accent,
            opacity: Ratio(1.0),
            rule: FillRule::Winding,
        });
    }
}
fn field(
    field: &Field,
    style: &DrawingStyle,
    f: Frame,
    clips: &[Path],
    art: &mut Artwork,
    decorate: bool,
) {
    match field {
        Field::Solid { tincture } => add(
            art,
            f.path(&Path::rect(0.0, 0.0, 1.0, 1.0)),
            *tincture,
            clips,
        ),
        Field::Divided {
            division,
            boundary,
            tinctures,
        } => {
            add(
                art,
                f.path(&Path::rect(0.0, 0.0, 1.0, 1.0)),
                tinctures[1],
                clips,
            );
            add(
                art,
                f.path(&fields::division_path(*division, *boundary)),
                tinctures[0],
                clips,
            );
        }
        Field::Patterned {
            pattern,
            repeats,
            tinctures,
        } => {
            add(
                art,
                f.path(&Path::rect(0.0, 0.0, 1.0, 1.0)),
                tinctures[1],
                clips,
            );
            for p in fields::patterns(*pattern, *repeats) {
                add(art, f.path(&p), tinctures[0], clips);
            }
        }
        Field::Quarterly { quarters } => {
            for (i, q) in quarters.iter().enumerate() {
                let sub = f.child([(i % 2) as f32 * 0.5, (i / 2) as f32 * 0.5], [0.5; 2]);
                let mut clips = clips.to_vec();
                clips.push(sub.path(&Path::rect(0.0, 0.0, 1.0, 1.0)));
                if decorate {
                    arms(q, style, sub, &clips, art);
                } else {
                    self::field(&q.field, style, sub, &clips, art, false);
                }
            }
        }
    }
}
fn charge(
    c: &Charge,
    style: &DrawingStyle,
    f: Frame,
    clips: &[Path],
    fields: &[Shape],
    art: &mut Artwork,
) {
    let mut drawing = crate::animals::draw(c, style);
    let angle = c.rotation.0.to_radians();
    let (sin, cos) = angle.sin_cos();
    let mapped = |p: [f32; 2]| {
        let x = (p[0] - 0.5) * c.size[0].0;
        let y = (p[1] - 0.5) * c.size[1].0;
        [
            c.center[0] + x * cos - y * sin,
            c.center[1] + x * sin + y * cos,
        ]
    };
    for part in &mut drawing.shapes {
        part.path = f.path(&part.path.mapped(mapped));
        part.stroke *= f.size[0].min(f.size[1]) * c.size[0].0.min(c.size[1].0);
        part.clips = part
            .clips
            .iter()
            .map(|c| f.path(&c.mapped(mapped)))
            .collect();
        part.clips.extend_from_slice(clips);
        match c.color {
            Coloring::Solid { .. } => art.shapes.push(part.clone()),
            Coloring::Counterchanged { tinctures } => {
                if !part.role.is_charge() {
                    art.shapes.push(part.clone());
                    continue;
                }
                for field in fields {
                    let mut p = part.clone();
                    p.clips.extend(field.clips.clone());
                    p.clips.push(field.path.clone());
                    p.tincture = if field.tincture == tinctures[0] {
                        tinctures[1]
                    } else if field.tincture == tinctures[1] {
                        tinctures[0]
                    } else {
                        part.tincture
                    };
                    art.shapes.push(p);
                }
            }
        }
    }
}

fn bordure(o: &Ordinary, f: Frame, clips: &[Path], art: &mut Artwork) {
    let Some(edge) = clips.last() else { return };
    let points = edge.flattened(64);
    let center = [f.origin[0] + f.size[0] * 0.5, f.origin[1] + f.size[1] * 0.5];
    let mut inner = Path::default();
    for (i, p) in points.into_iter().rev().enumerate() {
        let factor = 1.0 - o.width.0 * 2.0 + fields::wave(i as f32 / 192.0, o.boundary);
        let p = [
            center[0] + (p[0] - center[0]) * factor,
            center[1] + (p[1] - center[1]) * factor,
        ];
        if i == 0 {
            inner = Path::new(p[0], p[1]);
        } else {
            inner = inner.line(p[0], p[1]);
        }
    }
    add(art, edge.clone().joined(inner.close()), o.tincture, clips);
    art.shapes.last_mut().unwrap().role = PaintRole::Charge;
}
