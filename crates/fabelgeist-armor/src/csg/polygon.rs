use super::plane::Plane;
use super::vertex::Vertex;

#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    pub vertices: Vec<Vertex>,
    pub plane: Plane,
}

impl Polygon {
    pub fn new(vertices: Vec<Vertex>) -> Self {
        let plane = Plane::from_points(
            vertices[0].position,
            vertices[1].position,
            vertices[2].position,
        );
        Self { vertices, plane }
    }

    pub fn invert(&mut self) {
        self.vertices.reverse();
        for v in &mut self.vertices {
            v.normal = -v.normal;
        }
        self.plane.invert();
    }
}
