//! A head to grow hair on: a mannequin bust as a signed distance field.
//!
//! DiffLocks grows every hairstyle on the scalp of the mean SMPL-X body and
//! keeps it off the body with shrinkwrap modifiers and a cloth simulation.
//! SMPL-X is not ours to ship, so the bust here is a handful of smoothly
//! united ellipsoids and capsules of an adult's proportions: cranium, jaw,
//! nose, ears, neck, shoulders and chest. Being a distance field it is its
//! own collider -- exact distance and outward direction anywhere, for the
//! shrinkwrap and the drape -- and [`Head::mesh`] extracts it as triangles
//! (surface nets) for drawing and for the scalp.
//!
//! The scalp is the part of that mesh above the hairline: a curve around the
//! head, high on the forehead, over the ears and down to the nape. Its UV map
//! is a polar projection from the cranium's centre -- the crown in the
//! middle, the hairline near the rim -- one chart, as the paper's ARAP
//! unwrap of the SMPL-X scalp is. Any other scalp with a UV map can stand in
//! for it ([`Surface::from_ply`]); only the styles' guide placement reads the
//! head's own frame.
//!
//! Units are metres. The head looks down `+z`, `+y` is up, and the origin is
//! between the ears.

use fabelgeist_math::Vec3;
use rayon::prelude::*;

use crate::collider::Collider;
use crate::surface::Surface;

/// An ellipsoid by centre and semi-axes.
#[derive(Clone, Copy, Debug)]
pub struct Ellipsoid {
    pub center: Vec3,
    pub radii: Vec3,
}

impl Ellipsoid {
    /// Inigo Quilez's bound for an ellipsoid's distance: exact on the
    /// surface, close near it.
    fn distance(&self, p: Vec3) -> f32 {
        let q = p - self.center;
        let k0 = (q / self.radii).length();
        let k1 = (q / (self.radii * self.radii)).length();
        if k1 < 1e-12 {
            return -self.radii.min_component();
        }
        k0 * (k0 - 1.0) / k1
    }
}

/// A capsule: the points within `radius` of a segment.
#[derive(Clone, Copy, Debug)]
pub struct Capsule {
    pub a: Vec3,
    pub b: Vec3,
    pub radius: f32,
}

impl Capsule {
    fn distance(&self, p: Vec3) -> f32 {
        let ab = self.b - self.a;
        let t = ((p - self.a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
        (p - (self.a + ab * t)).length() - self.radius
    }
}

/// Polynomial smooth minimum: the union of two fields, filleted over `k`.
fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}

/// A mannequin bust.
#[derive(Clone, Debug)]
pub struct Head {
    pub cranium: Ellipsoid,
    pub jaw: Ellipsoid,
    pub nose: Ellipsoid,
    pub ears: [Ellipsoid; 2],
    pub neck: Capsule,
    pub shoulders: Ellipsoid,
    pub chest: Ellipsoid,
    /// Where the bust is cut off below.
    pub base: f32,
    /// The hairline's height at the front, over the ears and at the nape.
    pub hairline: [f32; 3],
    /// How far the hairline recedes at the temples, metres (an M-shaped
    /// hairline), and how far it dips in the middle of the forehead (a
    /// widow's peak).
    pub recession: f32,
    pub peak: f32,
}

impl Default for Head {
    fn default() -> Self {
        Self::mannequin()
    }
}

impl Head {
    /// An adult head of average proportions: 15 cm wide, 20 cm long, 23 cm
    /// from crown to chin, on a neck and shoulders.
    pub fn mannequin() -> Self {
        Self {
            cranium: Ellipsoid {
                center: Vec3::new(0.0, 0.025, -0.012),
                radii: Vec3::new(0.076, 0.09, 0.1),
            },
            jaw: Ellipsoid {
                center: Vec3::new(0.0, -0.045, 0.028),
                radii: Vec3::new(0.058, 0.075, 0.068),
            },
            nose: Ellipsoid {
                center: Vec3::new(0.0, -0.022, 0.094),
                radii: Vec3::new(0.013, 0.024, 0.02),
            },
            ears: [-1.0, 1.0].map(|side| Ellipsoid {
                center: Vec3::new(0.074 * side, -0.004, -0.008),
                radii: Vec3::new(0.011, 0.03, 0.019),
            }),
            neck: Capsule {
                a: Vec3::new(0.0, -0.05, -0.022),
                b: Vec3::new(0.0, -0.24, -0.02),
                radius: 0.054,
            },
            shoulders: Ellipsoid {
                center: Vec3::new(0.0, -0.3, -0.02),
                radii: Vec3::new(0.2, 0.085, 0.1),
            },
            chest: Ellipsoid {
                center: Vec3::new(0.0, -0.42, -0.015),
                radii: Vec3::new(0.175, 0.13, 0.11),
            },
            base: -0.5,
            hairline: [0.062, 0.03, -0.075],
            recession: 0.0,
            peak: 0.0,
        }
    }

    /// The top of the head.
    pub fn crown(&self) -> Vec3 {
        self.cranium.center + Vec3::new(0.0, self.cranium.radii.y, 0.0)
    }

    /// Signed distance: negative inside the bust.
    pub fn distance(&self, p: Vec3) -> f32 {
        let head = smooth_min(self.cranium.distance(p), self.jaw.distance(p), 0.04);
        let head = smooth_min(head, self.nose.distance(p), 0.012);
        let ears = self.ears[0].distance(p).min(self.ears[1].distance(p));
        let head = smooth_min(head, ears, 0.006);
        let neck = smooth_min(head, self.neck.distance(p), 0.03);
        let torso = smooth_min(self.shoulders.distance(p), self.chest.distance(p), 0.08);
        let body = smooth_min(neck, torso, 0.06);
        body.max(self.base - p.y)
    }

    /// The field's gradient by central differences: the outward normal.
    pub fn gradient(&self, p: Vec3) -> Vec3 {
        let e = 2e-4;
        let d = |o: Vec3| self.distance(p + o) - self.distance(p - o);
        Vec3::new(
            d(Vec3::new(e, 0.0, 0.0)),
            d(Vec3::new(0.0, e, 0.0)),
            d(Vec3::new(0.0, 0.0, e)),
        )
        .normalize()
    }

    /// Azimuth around the vertical, from the front (`+z`) towards `+x`.
    pub fn azimuth(&self, p: Vec3) -> f32 {
        let q = p - self.cranium.center;
        q.x.atan2(q.z)
    }

    /// The hairline's height at an azimuth: a cosine series through its
    /// height at the front, over the ears and at the nape, raised at the
    /// temples as far as it recedes and lowered in the middle for a peak.
    pub fn hairline_height(&self, azimuth: f32) -> f32 {
        let [front, side, back] = self.hairline;
        let temples = (-((azimuth.abs() - 0.62) / 0.3).powi(2)).exp();
        let middle = (-(azimuth / 0.13).powi(2)).exp();
        around(front, side, back, azimuth) + self.recession * temples - self.peak * middle
    }

    /// How far above the hairline a point is (negative below it).
    pub fn above_hairline(&self, p: Vec3) -> f32 {
        p.y - self.hairline_height(self.azimuth(p))
    }

    /// Polar angle from straight up at the cranium's centre.
    pub fn polar(&self, p: Vec3) -> f32 {
        let offset = p - self.cranium.center;
        // Keep the small horizontal component near the crown: normalizing
        // first rounds the vertical component to one and loses that angle.
        Vec3::new(offset.x, 0.0, offset.z).length().atan2(offset.y)
    }

    /// Where on the scalp a point is: 1 at the crown, 0 at the hairline
    /// below it (by polar angle from the cranium's centre).
    pub fn elevation(&self, p: Vec3) -> f32 {
        let a = self.azimuth(p);
        let h = self.hairline_height(a);
        let edge = self.cranium.center
            + Vec3::new(a.sin() * 0.09, h - self.cranium.center.y, a.cos() * 0.09);
        let hairline = self.polar(edge).clamp(0.1, SCALP_POLAR_MAX);
        (1.0 - self.polar(p) / hairline).clamp(0.0, 1.0)
    }

    /// The scalp's UV map: a polar projection about the crown, radius
    /// proportional to the angle from straight up, `v` towards the face.
    pub fn scalp_uv(&self, p: Vec3) -> [f32; 2] {
        let r = self.polar(p) / SCALP_POLAR_MAX;
        let a = self.azimuth(p);
        [0.5 + 0.5 * r * a.sin(), 0.5 + 0.5 * r * a.cos()]
    }

    /// The bust's surface as triangles, from a grid of `cell` metres.
    pub fn mesh(&self, cell: f32) -> Surface {
        let lo = Vec3::new(-0.24, self.base - 0.01, -0.16);
        let hi = Vec3::new(0.24, 0.14, 0.14);
        let (positions, triangles) = surface_nets(|p| self.distance(p), lo, hi, cell);
        let normals = positions.iter().map(|&p| self.gradient(p)).collect();
        let uvs = positions.iter().map(|&p| self.scalp_uv(p)).collect();
        Surface::new(positions, Some(normals), Some(uvs), triangles)
    }

    /// The part of the mesh above the hairline, with the scalp UV map.
    pub fn scalp(&self, cell: f32) -> Surface {
        let full = self.mesh(cell);
        let keep: Vec<bool> = full
            .positions
            .iter()
            .map(|&p| self.above_hairline(p) > 0.0 && p.y > self.neck.a.y - 0.04)
            .collect();
        let mut remap = vec![u32::MAX; full.positions.len()];
        let (mut positions, mut normals, mut uvs) = (Vec::new(), Vec::new(), Vec::new());
        let mut triangles = Vec::new();
        for t in &full.triangles {
            if !t.iter().all(|&i| keep[i as usize]) {
                continue;
            }
            triangles.push(t.map(|i| {
                let i = i as usize;
                if remap[i] == u32::MAX {
                    remap[i] = positions.len() as u32;
                    positions.push(full.positions[i]);
                    normals.push(full.normals[i]);
                    uvs.push(full.uvs[i]);
                }
                remap[i]
            }));
        }
        Surface::new(positions, Some(normals), Some(uvs), triangles)
    }
}

/// The polar angle the scalp UV map's rim stands for: past the nape.
pub const SCALP_POLAR_MAX: f32 = 2.6;

/// A value round the head from its value at the front, at either side and
/// at the back: the cosine series `a0 + a1 cos(a) + a2 cos(2a)` through the
/// three.
pub fn around(front: f32, side: f32, back: f32, azimuth: f32) -> f32 {
    let a1 = (front - back) * 0.5;
    let a0_plus_a2 = (front + back) * 0.5;
    let a0 = (a0_plus_a2 + side) * 0.5;
    let a2 = a0_plus_a2 - a0;
    a0 + a1 * azimuth.cos() + a2 * (2.0 * azimuth).cos()
}

impl Collider for Head {
    fn query(&self, p: Vec3) -> (f32, Vec3) {
        (Head::distance(self, p), self.gradient(p))
    }

    fn distance(&self, p: Vec3) -> f32 {
        Head::distance(self, p)
    }
}

/// Naive surface nets over a field sampled on a grid: a vertex in every cell
/// the surface crosses, at the mean of its edges' crossings pulled onto the
/// surface, and a quad across every grid edge the surface cuts, facing out.
pub fn surface_nets(
    field: impl Fn(Vec3) -> f32 + Sync,
    lo: Vec3,
    hi: Vec3,
    cell: f32,
) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let n = [0, 1, 2].map(|k| (((hi.axis(k) - lo.axis(k)) / cell).ceil() as usize).max(1));
    let corner = |i: usize, j: usize, k: usize| {
        lo + Vec3::new(i as f32 * cell, j as f32 * cell, k as f32 * cell)
    };
    let (nx, ny, nz) = (n[0] + 1, n[1] + 1, n[2] + 1);
    let index = |i: usize, j: usize, k: usize| (k * ny + j) * nx + i;
    let values: Vec<f32> = (0..nx * ny * nz)
        .into_par_iter()
        .map(|flat| {
            let i = flat % nx;
            let j = (flat / nx) % ny;
            let k = flat / (nx * ny);
            field(corner(i, j, k))
        })
        .collect();

    // A vertex for every cell with a sign change.
    const EDGES: [[usize; 2]; 12] = [
        [0, 1],
        [2, 3],
        [4, 5],
        [6, 7],
        [0, 2],
        [1, 3],
        [4, 6],
        [5, 7],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];
    let cells = n[0] * n[1] * n[2];
    let found: Vec<Option<Vec3>> = (0..cells)
        .into_par_iter()
        .map(|flat| {
            let i = flat % n[0];
            let j = (flat / n[0]) % n[1];
            let k = flat / (n[0] * n[1]);
            let corners: [(Vec3, f32); 8] = std::array::from_fn(|c| {
                let (di, dj, dk) = (c & 1, (c >> 1) & 1, (c >> 2) & 1);
                (
                    corner(i + di, j + dj, k + dk),
                    values[index(i + di, j + dj, k + dk)],
                )
            });
            let mut sum = Vec3::default();
            let mut count = 0;
            for [a, b] in EDGES {
                let ((pa, va), (pb, vb)) = (corners[a], corners[b]);
                if (va < 0.0) != (vb < 0.0) {
                    sum += pa.lerp(pb, va / (va - vb));
                    count += 1;
                }
            }
            if count == 0 {
                return None;
            }
            let mut p = sum * (1.0 / count as f32);
            // Two Newton steps onto the surface.
            for _ in 0..2 {
                let e = cell * 0.05;
                let d = field(p);
                let g = Vec3::new(
                    field(p + Vec3::new(e, 0.0, 0.0)) - field(p - Vec3::new(e, 0.0, 0.0)),
                    field(p + Vec3::new(0.0, e, 0.0)) - field(p - Vec3::new(0.0, e, 0.0)),
                    field(p + Vec3::new(0.0, 0.0, e)) - field(p - Vec3::new(0.0, 0.0, e)),
                ) * (0.5 / e);
                let g2 = g.length_squared();
                if g2 > 1e-12 {
                    let step = g * (d / g2);
                    // Stay near the cell: the mean is a good guess already.
                    if step.length() < cell {
                        p -= step;
                    }
                }
            }
            Some(p)
        })
        .collect();
    let mut vertex_of = vec![u32::MAX; cells];
    let mut positions = Vec::new();
    for (c, v) in found.into_iter().enumerate() {
        if let Some(p) = v {
            vertex_of[c] = positions.len() as u32;
            positions.push(p);
        }
    }

    // A quad for every interior grid edge with a sign change, joining the
    // four cells around it.
    let cell_index = |i: usize, j: usize, k: usize| (k * n[1] + j) * n[0] + i;
    let mut triangles = Vec::new();
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let v0 = values[index(i, j, k)];
                for axis in 0..3 {
                    let (di, dj, dk) = [(1, 0, 0), (0, 1, 0), (0, 0, 1)][axis];
                    let (i1, j1, k1) = (i + di, j + dj, k + dk);
                    if i1 >= nx || j1 >= ny || k1 >= nz {
                        continue;
                    }
                    let v1 = values[index(i1, j1, k1)];
                    if (v0 < 0.0) == (v1 < 0.0) {
                        continue;
                    }
                    // The four cells sharing this edge, around it.
                    let (u, w) = [(1, 2), (2, 0), (0, 1)][axis];
                    let at = |du: usize, dw: usize| -> Option<u32> {
                        let mut c = [i as isize, j as isize, k as isize];
                        c[u] -= du as isize;
                        c[w] -= dw as isize;
                        if c.iter().zip(n).any(|(&x, m)| x < 0 || x as usize >= m) {
                            return None;
                        }
                        let v = vertex_of[cell_index(c[0] as usize, c[1] as usize, c[2] as usize)];
                        (v != u32::MAX).then_some(v)
                    };
                    let (Some(a), Some(b), Some(c), Some(d)) =
                        (at(0, 0), at(1, 0), at(1, 1), at(0, 1))
                    else {
                        continue;
                    };
                    // Inside to outside along the axis: counter-clockwise
                    // seen from outside.
                    if v0 < 0.0 {
                        triangles.push([a, b, c]);
                        triangles.push([a, c, d]);
                    } else {
                        triangles.push([a, c, b]);
                        triangles.push([a, d, c]);
                    }
                }
            }
        }
    }
    (positions, triangles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bust_is_head_sized() {
        let head = Head::mannequin();
        assert!(head.distance(head.cranium.center) < 0.0);
        assert!(head.distance(Vec3::new(0.0, 0.3, 0.0)) > 0.1);
        // Crown to chin about 23 cm.
        let top = (0..400)
            .map(|i| i as f32 * 0.001)
            .find(|&y| head.distance(Vec3::new(0.0, y, 0.0)) > 0.0)
            .unwrap();
        assert!((0.1..0.13).contains(&top), "{top}");
    }

    #[test]
    fn the_scalp_is_a_cap_with_a_disc_of_uvs() {
        let head = Head::mannequin();
        let scalp = head.scalp(0.008);
        assert!(!scalp.triangles.is_empty());
        // Somewhere between 400 and 900 square centimetres.
        assert!((0.03..0.09).contains(&scalp.area()), "{}", scalp.area());
        for (p, uv) in scalp.positions.iter().zip(&scalp.uvs) {
            assert!(head.above_hairline(*p) > 0.0);
            let r = ((uv[0] - 0.5).powi(2) + (uv[1] - 0.5).powi(2)).sqrt();
            assert!(r <= 0.5, "{uv:?}");
        }
    }

    #[test]
    fn mesh_normals_face_out() {
        let head = Head::mannequin();
        let mesh = head.mesh(0.01);
        let mut agree = 0;
        for t in &mesh.triangles {
            let [a, b, c] = t.map(|i| mesh.positions[i as usize]);
            let n = (b - a).cross(c - a);
            let centre = (a + b + c) * (1.0 / 3.0);
            if n.dot(head.gradient(centre)) > 0.0 {
                agree += 1;
            }
        }
        assert!(agree as f32 > mesh.triangles.len() as f32 * 0.99);
    }
}
