use super::polygon::Polygon;
use fabelgeist_math::vector::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub w: f32,
}

impl Plane {
    pub const EPSILON: f32 = 1e-5;

    pub fn new(normal: Vec3, w: f32) -> Self {
        Self { normal, w }
    }

    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let n = (b - a).cross(c - a).normalize();
        Self {
            normal: n,
            w: n.dot(a),
        }
    }

    pub fn invert(&mut self) {
        self.normal = -self.normal;
        self.w = -self.w;
    }

    pub fn split_polygon(
        &self,
        polygon: Polygon,
        coplanar_front: &mut Vec<Polygon>,
        coplanar_back: &mut Vec<Polygon>,
        front: &mut Vec<Polygon>,
        back: &mut Vec<Polygon>,
    ) {
        // Classify vertices
        let mut polygon_type = 0;
        let mut types = Vec::with_capacity(polygon.vertices.len());
        for v in &polygon.vertices {
            let t = self.normal.dot(v.position) - self.w;
            let side = if t < -Self::EPSILON {
                2 // BACK
            } else if t > Self::EPSILON {
                1 // FRONT
            } else {
                0 // COPLANAR
            };
            polygon_type |= side;
            types.push(side);
        }

        match polygon_type {
            0 => {
                // Coplanar
                if self.normal.dot(polygon.plane.normal) > 0.0 {
                    coplanar_front.push(polygon);
                } else {
                    coplanar_back.push(polygon);
                }
            }
            1 => {
                // Front
                front.push(polygon);
            }
            2 => {
                // Back
                back.push(polygon);
            }
            3 => {
                // Spanning
                let mut f_verts = Vec::new();
                let mut b_verts = Vec::new();
                let len = polygon.vertices.len();
                for i in 0..len {
                    let j = (i + 1) % len;
                    let vi = polygon.vertices[i];
                    let vj = polygon.vertices[j];
                    let ti = types[i];
                    let tj = types[j];

                    if ti != 2 {
                        f_verts.push(vi);
                    }
                    if ti != 1 {
                        b_verts.push(vi);
                    }

                    if (ti | tj) == 3 {
                        // Spanning front and back
                        let dist_i = self.normal.dot(vi.position) - self.w;
                        let dist_j = self.normal.dot(vj.position) - self.w;
                        let t = dist_i / (dist_i - dist_j);
                        let v = vi.lerp(vj, t);
                        f_verts.push(v);
                        b_verts.push(v);
                    }
                }
                if f_verts.len() >= 3 {
                    front.push(Polygon::new(f_verts));
                }
                if b_verts.len() >= 3 {
                    back.push(Polygon::new(b_verts));
                }
            }
            _ => unreachable!(),
        }
    }
}
