//! Shared vector paths, clipping and rasterization for every output format.
mod paint;
use crate::{Error, document::*, paint::PaintPalette};
pub use paint::{PaintRole, PaintTone};
pub use tiny_skia::FillRule;
use tiny_skia::{Mask, Paint, PathBuilder, Pixmap, Stroke, Transform};
pub const ARTBOARD: f32 = 1000.0;

#[derive(Clone, Debug)]
pub enum Segment {
    Move([f32; 2]),
    Line([f32; 2]),
    Curve([[f32; 2]; 3]),
    Close,
}
#[derive(Clone, Debug, Default)]
pub struct Path {
    pub segments: Vec<Segment>,
}
impl Path {
    pub fn joined(mut self, other: Self) -> Self {
        self.segments.extend(other.segments);
        self
    }
    /// Samples a single closed outline without duplicating its first vertex.
    pub fn flattened(&self, steps: usize) -> Vec<[f32; 2]> {
        let mut out = Vec::new();
        let mut last = [0.0; 2];
        for s in &self.segments {
            match s {
                Segment::Move(p) | Segment::Line(p) => {
                    out.push(*p);
                    last = *p;
                }
                Segment::Curve([a, b, c]) => {
                    let start = last;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let u = 1.0 - t;
                        out.push(std::array::from_fn(|k| {
                            u * u * u * start[k]
                                + 3.0 * u * u * t * a[k]
                                + 3.0 * u * t * t * b[k]
                                + t * t * t * c[k]
                        }));
                    }
                    last = *c;
                }
                Segment::Close => (),
            }
        }
        if out.first() == out.last() {
            out.pop();
        }
        out
    }
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            segments: vec![Segment::Move([x, y])],
        }
    }
    pub fn line(mut self, x: f32, y: f32) -> Self {
        self.segments.push(Segment::Line([x, y]));
        self
    }
    pub fn curve(mut self, a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> Self {
        self.segments.push(Segment::Curve([a, b, c]));
        self
    }
    pub fn close(mut self) -> Self {
        self.segments.push(Segment::Close);
        self
    }
    pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self::new(x, y)
            .line(x + w, y)
            .line(x + w, y + h)
            .line(x, y + h)
            .close()
    }
    pub fn ellipse(x: f32, y: f32, rx: f32, ry: f32) -> Self {
        const CIRCLE_HANDLE: f32 = 0.552_284_8;
        let k = CIRCLE_HANDLE;
        Self::new(x + rx, y)
            .curve([x + rx, y + ry * k], [x + rx * k, y + ry], [x, y + ry])
            .curve([x - rx * k, y + ry], [x - rx, y + ry * k], [x - rx, y])
            .curve([x - rx, y - ry * k], [x - rx * k, y - ry], [x, y - ry])
            .curve([x + rx * k, y - ry], [x + rx, y - ry * k], [x + rx, y])
            .close()
    }
    pub fn mapped(&self, map: impl Fn([f32; 2]) -> [f32; 2]) -> Self {
        Self {
            segments: self
                .segments
                .iter()
                .map(|s| match s {
                    Segment::Move(p) => Segment::Move(map(*p)),
                    Segment::Line(p) => Segment::Line(map(*p)),
                    Segment::Curve(p) => Segment::Curve(p.map(&map)),
                    Segment::Close => Segment::Close,
                })
                .collect(),
        }
    }
    fn skia(&self) -> Option<tiny_skia::Path> {
        let mut p = PathBuilder::new();
        for s in &self.segments {
            match s {
                Segment::Move(v) => p.move_to(v[0], v[1]),
                Segment::Line(v) => p.line_to(v[0], v[1]),
                Segment::Curve(v) => {
                    p.cubic_to(v[0][0], v[0][1], v[1][0], v[1][1], v[2][0], v[2][1])
                }
                Segment::Close => p.close(),
            }
        }
        p.finish()
    }
    pub fn svg(&self) -> String {
        use std::fmt::Write;
        let mut d = String::new();
        for s in &self.segments {
            match s {
                Segment::Move(p) => {
                    write!(d, "M{:.3},{:.3}", p[0], p[1]).unwrap();
                }
                Segment::Line(p) => {
                    write!(d, "L{:.3},{:.3}", p[0], p[1]).unwrap();
                }
                Segment::Curve(p) => {
                    write!(
                        d,
                        "C{:.3},{:.3} {:.3},{:.3} {:.3},{:.3}",
                        p[0][0], p[0][1], p[1][0], p[1][1], p[2][0], p[2][1]
                    )
                    .unwrap();
                }
                Segment::Close => d.push('Z'),
            }
        }
        d
    }
}

#[derive(Clone, Debug)]
pub struct Shape {
    pub path: Path,
    pub tincture: Tincture,
    /// Zero fills the path; positive values stroke it in artboard coordinates.
    pub stroke: f32,
    pub clips: Vec<Path>,
    pub role: PaintRole,
    /// Painted layer coverage, independent of camera and lighting.
    pub opacity: Ratio,
    pub rule: FillRule,
}
#[derive(Clone, Debug, Default)]
pub struct Artwork {
    pub shapes: Vec<Shape>,
    pub attribution: Option<String>,
}
impl Artwork {
    pub fn compose(document: &Document) -> Result<Self, Error> {
        document.validate()?;
        Ok(crate::composition::compose(document))
    }
    pub fn svg(&self, palette: &PaintPalette) -> String {
        use std::fmt::Write;
        let mut s = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {ARTBOARD} {ARTBOARD}\"><defs>"
        );
        for (i, shape) in self.shapes.iter().enumerate() {
            for (j, c) in shape.clips.iter().enumerate() {
                write!(
                    s,
                    "<clipPath id=\"c{i}_{j}\"><path d=\"{}\"/></clipPath>",
                    c.svg()
                )
                .unwrap();
            }
        }
        s.push_str("</defs>");
        if let Some(credit) = &self.attribution {
            let escaped = credit.replace('&', "&amp;").replace('<', "&lt;");
            write!(s, "<metadata>{escaped}</metadata>").unwrap();
        }
        for (i, shape) in self.shapes.iter().enumerate() {
            for j in 0..shape.clips.len() {
                write!(s, "<g clip-path=\"url(#c{i}_{j})\">").unwrap();
            }
            let c = palette[shape.tincture].color(shape.role.tone());
            write!(s, "<g opacity=\"{}\">", shape.opacity.0).unwrap();
            if shape.stroke > 0.0 {
                write!(s,"<path d=\"{}\" fill=\"none\" stroke=\"#{:02x}{:02x}{:02x}\" stroke-width=\"{:.3}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",shape.path.svg(),c[0],c[1],c[2],shape.stroke).unwrap();
            } else {
                write!(
                    s,
                    "<path d=\"{}\" fill=\"#{:02x}{:02x}{:02x}\" fill-rule=\"{}\"/>",
                    shape.path.svg(),
                    c[0],
                    c[1],
                    c[2],
                    if shape.rule == FillRule::EvenOdd {
                        "evenodd"
                    } else {
                        "nonzero"
                    }
                )
                .unwrap();
            }
            s.push_str("</g>");
            for _ in &shape.clips {
                s.push_str("</g>");
            }
        }
        s.push_str("</svg>");
        s
    }
    /// Coverage is evaluated once, then applied to independently colored outputs.
    pub fn raster(&self, size: u32, palettes: &[PaintPalette]) -> Result<Vec<Vec<u8>>, Error> {
        self.raster_with(size, palettes.len(), |shape, channel| {
            palettes[channel][shape.tincture].color(shape.role.tone())
        })
    }
    /// Rasterize shared coverage with per-shape data channels.
    pub(crate) fn raster_with(
        &self,
        size: u32,
        channels: usize,
        color: impl Fn(&Shape, usize) -> [u8; 3],
    ) -> Result<Vec<Vec<u8>>, Error> {
        if !(32..=4096).contains(&size) {
            return Err(Error::Invalid("raster size must be 32..=4096".into()));
        }
        let mut pixmaps = (0..channels)
            .map(|_| {
                Pixmap::new(size, size).ok_or_else(|| Error::Invalid("raster allocation".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let transform = Transform::from_scale(size as f32 / ARTBOARD, size as f32 / ARTBOARD);
        for shape in &self.shapes {
            let path = shape
                .path
                .skia()
                .ok_or_else(|| Error::Invalid("empty vector path".into()))?;
            let mut mask = Mask::new(size, size).unwrap();
            mask.data_mut().fill(255);
            for clip in &shape.clips {
                mask.intersect_path(
                    &clip
                        .skia()
                        .ok_or_else(|| Error::Invalid("empty clipping path".into()))?,
                    FillRule::Winding,
                    true,
                    transform,
                );
            }
            for (channel, pixels) in pixmaps.iter_mut().enumerate() {
                let color = color(shape, channel);
                let mut paint = Paint::default();
                paint.set_color_rgba8(
                    color[0],
                    color[1],
                    color[2],
                    crate::bake::unit_byte(shape.opacity.0),
                );
                paint.anti_alias = true;
                if shape.stroke > 0.0 {
                    pixels.stroke_path(
                        &path,
                        &paint,
                        &Stroke {
                            width: shape.stroke,
                            line_cap: tiny_skia::LineCap::Round,
                            line_join: tiny_skia::LineJoin::Round,
                            ..Default::default()
                        },
                        transform,
                        Some(&mask),
                    );
                } else {
                    pixels.fill_path(&path, &paint, shape.rule, transform, Some(&mask));
                }
            }
        }
        Ok(pixmaps
            .into_iter()
            .map(|p| {
                p.pixels()
                    .iter()
                    .flat_map(|p| {
                        let c = p.demultiply();
                        [c.red(), c.green(), c.blue(), c.alpha()]
                    })
                    .collect()
            })
            .collect())
    }
}
