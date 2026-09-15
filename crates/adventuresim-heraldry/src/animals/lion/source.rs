//! Resolve the c.1530 German lion redraw without changing its path grammar.
use crate::artwork::{Path, Segment};
use std::sync::OnceLock;
use usvg::tiny_skia_path::{PathSegment, Point, Transform};

const SOURCE: &str = include_str!("../../../references/German_Lion_1530.svg");
pub(super) const SOURCE_SIZE: [f32; 2] = [326.7587, 367.24664];

pub(super) fn paths() -> &'static [usvg::Path] {
    static PATHS: OnceLock<Vec<usvg::Path>> = OnceLock::new();
    PATHS.get_or_init(|| {
        let tree = usvg::Tree::from_data(SOURCE.as_bytes(), &usvg::Options::default())
            .expect("vendored German lion SVG");
        let mut paths = Vec::new();
        collect(tree.root(), &mut paths);
        paths
    })
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
/// Preserve relative commands, arcs, compound contours and nested transforms.
pub(super) fn convert(data: &usvg::tiny_skia_path::Path, transform: Transform) -> Path {
    let map = |mut p: Point| {
        transform.map_point(&mut p);
        [p.x / SOURCE_SIZE[0], p.y / SOURCE_SIZE[1]]
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
