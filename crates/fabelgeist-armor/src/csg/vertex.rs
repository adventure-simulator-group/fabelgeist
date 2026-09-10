use fabelgeist_math::vector::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub distance: Option<f32>,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2, distance: Option<f32>) -> Self {
        Self {
            position,
            normal,
            uv,
            distance,
        }
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        let distance = match (self.distance, other.distance) {
            (Some(d0), Some(d1)) => Some(d0 + (d1 - d0) * t),
            _ => None,
        };
        Self {
            position: self.position.lerp(other.position, t),
            normal: self.normal.lerp(other.normal, t).normalize(),
            uv: Vec2::new(
                self.uv.x + (other.uv.x - self.uv.x) * t,
                self.uv.y + (other.uv.y - self.uv.y) * t,
            ),
            distance,
        }
    }
}
