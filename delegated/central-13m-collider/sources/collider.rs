//! What strands must stay outside of.
//!
//! The pipeline keeps hair out of the body twice: shrinkwrap pushes points
//! that sank below the surface back onto it, and the drape collides every
//! point it moves. Both need the same query -- how far a point is from the
//! body, signed, and which way is out -- which a distance field answers
//! directly ([`crate::Head`]) and a mesh answers through its closest
//! triangle ([`MeshCollider`]).

use fabelgeist_bvh::TriangleBvh;
use fabelgeist_math::Vec3;

use crate::surface::vertex_normals;

pub trait Collider: Sync {
    /// Signed distance to the surface (negative inside) and the outward
    /// direction at the closest point.
    fn query(&self, p: Vec3) -> (f32, Vec3);

    /// Signed distance alone, where that is cheaper.
    fn distance(&self, p: Vec3) -> f32 {
        self.query(p).0
    }
}

/// Move `p` until it is at least `offset` outside, stepping along the
/// outward direction. A distance field built from smooth unions is only a
/// bound inside, and where two parts meet (an ear on the head) its gradient
/// can point into the other part, so a single step is not always enough.
pub fn push_out(collider: &(impl Collider + ?Sized), mut p: Vec3, offset: f32) -> Vec3 {
    for _ in 0..6 {
        if collider.distance(p) >= offset {
            break;
        }
        let (distance, out) = collider.query(p);
        p += out * (offset - distance + 1e-5);
    }
    p
}

/// Keep a coarse polyline -- a strand drawn with a few points, a card's
/// spine -- off the body. Its points lie on the fine strand, which is off
/// the body, but the straight segments between them cut the corners of a
/// strand wrapping a curved scalp and sink into it: at a distance the hair
/// there disappears, and the head looks bald. Wherever a segment comes
/// nearer than `clearance`, its ends are pushed out until it does not; the
/// root stays where it is, so a first segment is cleared by lifting its far
/// end.
pub fn keep_clear(
    collider: &(impl Collider + ?Sized),
    polyline: &mut [Vec3],
    clearance: f32,
    passes: usize,
) {
    let n = polyline.len();
    if n < 2 {
        return;
    }
    let mut push = vec![Vec3::default(); n];
    let longer = |a: Vec3, b: Vec3| {
        if b.length_squared() > a.length_squared() {
            b
        } else {
            a
        }
    };
    for _ in 0..passes {
        push.iter_mut().for_each(|p| *p = Vec3::default());
        let mut any = false;
        let mut near = collider.distance(polyline[0]);
        for i in 0..n - 1 {
            let (a, b) = (polyline[i], polyline[i + 1]);
            let far = collider.distance(b);
            let reach = 0.5 * (b - a).length();
            // A distance field changes no faster than distance: a segment
            // whose ends are well clear cannot come near in between.
            let closest = near.min(far);
            near = far;
            if closest - reach >= clearance {
                continue;
            }
            for t in [0.25, 0.5, 0.75] {
                let q = a.lerp(b, t);
                let d = collider.distance(q);
                if d >= clearance {
                    continue;
                }
                any = true;
                let (_, out) = collider.query(q);
                let need = clearance - d;
                if i == 0 {
                    push[1] = longer(push[1], out * (need / t).min(0.01));
                } else {
                    push[i] = longer(push[i], out * need);
                    push[i + 1] = longer(push[i + 1], out * need);
                }
            }
        }
        if !any {
            break;
        }
        for (p, d) in polyline.iter_mut().zip(&push).skip(1) {
            *p += *d;
        }
    }
}

/// A closed triangle mesh as a collider: distance to its closest triangle,
/// signed by the side of that triangle the point is on (by the
/// angle-weighted pseudo-normal of its closest feature, approximated by the
/// interpolated vertex normals).
pub struct MeshCollider {
    bvh: TriangleBvh,
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    triangles: Vec<[u32; 3]>,
    /// Queries further than this report "far outside".
    reach: f32,
}

impl MeshCollider {
    pub fn new(positions: Vec<Vec3>, triangles: Vec<[u32; 3]>, reach: f32) -> Self {
        let normals = vertex_normals(&positions, &triangles);
        Self {
            bvh: TriangleBvh::new(positions.clone(), triangles.clone()),
            positions,
            normals,
            triangles,
            reach,
        }
    }
}

impl Collider for MeshCollider {
    fn query(&self, p: Vec3) -> (f32, Vec3) {
        let Some((t, closest, distance)) = self.bvh.closest_point(p, self.reach) else {
            return (self.reach, Vec3::up());
        };
        let [a, b, c] = self.triangles[t as usize];
        let w = crate::surface::barycentric(
            closest,
            self.positions[a as usize],
            self.positions[b as usize],
            self.positions[c as usize],
        );
        let normal = (self.normals[a as usize] * w[0]
            + self.normals[b as usize] * w[1]
            + self.normals[c as usize] * w[2])
            .normalize();
        let outside = (p - closest).dot(normal) >= 0.0;
        let away = if distance > 1e-7 {
            (p - closest) * (1.0 / distance)
        } else {
            normal
        };
        if outside {
            (distance, away)
        } else {
            (-distance, -away)
        }
    }
}

/// Nothing to collide with.
pub struct NoCollider;

impl Collider for NoCollider {
    fn query(&self, _: Vec3) -> (f32, Vec3) {
        (f32::INFINITY, Vec3::up())
    }
}
