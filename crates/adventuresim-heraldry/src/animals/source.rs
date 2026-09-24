//! Resolve sourced SVG commands, instances and transforms into shared geometry.
use crate::artwork::{Path, Segment};
use usvg::tiny_skia_path::{PathSegment, Point, Transform};

pub(super) struct SvgDrawing {
    pub paths: Vec<usvg::Path>,
    size: [f32; 2],
}
impl SvgDrawing {
    pub fn from_svg(source: &str) -> Self {
        let tree = usvg::Tree::from_data(source.as_bytes(), &usvg::Options::default())
            .expect("vendored heraldic SVG");
        let mut paths = Vec::new();
        collect(tree.root(), &mut paths);
        Self {
            paths,
            size: [tree.size().width(), tree.size().height()],
        }
    }
    /// Preserve arcs, compound contours and nested transforms in the UV chart.
    pub fn geometry(&self, data: &usvg::tiny_skia_path::Path, transform: Transform) -> Path {
        let map = |mut p: Point| {
            transform.map_point(&mut p);
            [p.x / self.size[0], p.y / self.size[1]]
        };
        let mut path = Path::default();
        let mut last = [0.0; 2];
        let mut start = last;
        for s in data.segments() {
            path.segments.push(match s {
                PathSegment::MoveTo(p) => {
                    last = map(p);
                    start = last;
                    Segment::Move(last)
                }
                PathSegment::LineTo(p) => {
                    last = map(p);
                    Segment::Line(last)
                }
                PathSegment::QuadTo(a, b) => {
                    let a = map(a);
                    let b = map(b);
                    let controls = [
                        std::array::from_fn(|i| last[i] + (a[i] - last[i]) * 2.0 / 3.0),
                        std::array::from_fn(|i| b[i] + (a[i] - b[i]) * 2.0 / 3.0),
                        b,
                    ];
                    last = b;
                    Segment::Curve(controls)
                }
                PathSegment::CubicTo(a, b, c) => {
                    last = map(c);
                    Segment::Curve([map(a), map(b), last])
                }
                PathSegment::Close => {
                    last = start;
                    Segment::Close
                }
            });
        }
        path
    }
}
fn collect(group: &usvg::Group, paths: &mut Vec<usvg::Path>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => collect(g, paths),
            usvg::Node::Path(p) if p.is_visible() => paths.push(p.as_ref().clone()),
            _ => (),
        }
    }
}
