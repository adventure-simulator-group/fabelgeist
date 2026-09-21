// Vendored from Sensorial Prism, products/prism/libraries/csg/src.
// Copyright Sensorial Systems. Apache-2.0; see LICENSE.
// Adaptation: vector imports use fabelgeist-math (the vendored Prism math),
// and items the armor does not use are removed.
mod bsp_node;
mod plane;
mod polygon;
mod vertex;
use bsp_node::BspNode;
pub use polygon::Polygon;
pub use vertex::Vertex;

pub fn subtract(a: Vec<Polygon>, b: Vec<Polygon>) -> Vec<Polygon> {
    let mut a = BspNode::new(a);
    let mut b = BspNode::new(b);
    a.invert();
    a.clip_to(&b);
    b.clip_to(&a);
    b.invert();
    b.clip_to(&a);
    b.invert();
    a.build(b.all_polygons());
    a.invert();
    a.all_polygons()
}
