//! Triangle surfaces: the scalp strands grow from, and meshes they collide
//! with.
//!
//! DiffLocks grows hair on the scalp of an SMPL-X body, a 513-vertex patch
//! unwrapped once into a single UV chart. A [`Surface`] is any such patch:
//! positions, normals and a UV map, sampled by area to place roots, and
//! queried for its closest point to attach them (Blender's `Sample UV
//! Surface` and `Geometry Proximity`).

use anyhow::{Context, Result, bail};
use fabelgeist_bvh::TriangleBvh;
use fabelgeist_math::Vec3;

use crate::rng::Rng;

/// A triangle mesh with normals and a UV map.
pub struct Surface {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<[f32; 2]>,
    pub triangles: Vec<[u32; 3]>,
    /// Running total of triangle areas, for sampling by area.
    cdf: Vec<f32>,
    bvh: TriangleBvh,
}

/// A point on a surface.
#[derive(Clone, Copy, Debug)]
pub struct SurfacePoint {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: [f32; 2],
    pub triangle: u32,
}

impl Surface {
    /// A surface from its parts. Missing normals are computed from the
    /// triangles, area-weighted; missing UVs are zero.
    pub fn new(
        positions: Vec<Vec3>,
        normals: Option<Vec<Vec3>>,
        uvs: Option<Vec<[f32; 2]>>,
        triangles: Vec<[u32; 3]>,
    ) -> Self {
        let normals = normals.unwrap_or_else(|| vertex_normals(&positions, &triangles));
        let uvs = uvs.unwrap_or_else(|| vec![[0.0; 2]; positions.len()]);
        let mut cdf = Vec::with_capacity(triangles.len());
        let mut total = 0.0;
        for t in &triangles {
            let [a, b, c] = t.map(|i| positions[i as usize]);
            total += (b - a).cross(c - a).length() * 0.5;
            cdf.push(total);
        }
        let bvh = TriangleBvh::new(positions.clone(), triangles.clone());
        Self {
            positions,
            normals,
            uvs,
            triangles,
            cdf,
            bvh,
        }
    }

    pub fn area(&self) -> f32 {
        *self.cdf.last().unwrap_or(&0.0)
    }

    /// A point uniformly distributed by area.
    pub fn sample(&self, rng: &mut Rng) -> SurfacePoint {
        let share = rng.next_f32();
        self.sample_at(share, rng.next_f32(), rng.next_f32())
    }

    /// The point at `share` of the area (which triangle) and `u`, `v` across
    /// it (where in it), each in `[0, 1)`: [`Surface::sample`] from numbers
    /// drawn elsewhere.
    pub fn sample_at(&self, share: f32, u: f32, v: f32) -> SurfacePoint {
        let target = share * self.area();
        let t = self
            .cdf
            .partition_point(|&c| c < target)
            .min(self.cdf.len() - 1);
        let (mut u, mut v) = (u, v);
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        self.at_barycentric(t as u32, [1.0 - u - v, u, v])
    }

    fn at_barycentric(&self, triangle: u32, w: [f32; 3]) -> SurfacePoint {
        let [a, b, c] = self.triangles[triangle as usize].map(|i| i as usize);
        let position =
            self.positions[a] * w[0] + self.positions[b] * w[1] + self.positions[c] * w[2];
        let normal =
            (self.normals[a] * w[0] + self.normals[b] * w[1] + self.normals[c] * w[2]).normalize();
        let uv =
            [0, 1].map(|k| self.uvs[a][k] * w[0] + self.uvs[b][k] * w[1] + self.uvs[c][k] * w[2]);
        SurfacePoint {
            position,
            normal,
            uv,
            triangle,
        }
    }

    /// The closest point on the surface within `max_distance`.
    /// Points along the surface's open edges -- for a scalp, its hairline --
    /// no further than `step` apart.
    pub fn boundary(&self, step: f32) -> Vec<Vec3> {
        let mut edges = std::collections::HashMap::new();
        for t in &self.triangles {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                *edges.entry((a.min(b), a.max(b))).or_insert(0u32) += 1;
            }
        }
        let mut open: Vec<(u32, u32)> = edges
            .into_iter()
            .filter(|&(_, count)| count == 1)
            .map(|(edge, _)| edge)
            .collect();
        // In a fixed order: the map's is random.
        open.sort_unstable();
        let mut points = Vec::new();
        for (a, b) in open {
            let (pa, pb) = (self.positions[a as usize], self.positions[b as usize]);
            let steps = ((pb - pa).length() / step.max(1e-5)).ceil().max(1.0) as usize;
            for i in 0..steps {
                points.push(pa.lerp(pb, i as f32 / steps as f32));
            }
        }
        points
    }

    pub fn closest(&self, point: Vec3, max_distance: f32) -> Option<SurfacePoint> {
        let (triangle, closest, _) = self.bvh.closest_point(point, max_distance)?;
        let [a, b, c] = self.triangles[triangle as usize].map(|i| self.positions[i as usize]);
        Some(self.at_barycentric(triangle, barycentric(closest, a, b, c)))
    }

    /// A surface read from an ASCII or binary little-endian PLY with vertex
    /// `x y z`, optional `nx ny nz` and `s t` (or `u v`, `texture_u
    /// texture_v`), and triangle or quad faces -- the layout of DiffLocks'
    /// `scalp.ply`.
    pub fn from_ply(bytes: &[u8]) -> Result<Self> {
        let ply = crate::export::ply::read(bytes)?;
        let find = |names: &[&str]| {
            names
                .iter()
                .find_map(|n| ply.vertex_properties.iter().position(|p| p == n))
        };
        let (Some(x), Some(y), Some(z)) = (find(&["x"]), find(&["y"]), find(&["z"])) else {
            bail!("a PLY surface needs vertex x, y and z");
        };
        let positions: Vec<Vec3> = ply
            .vertices
            .iter()
            .map(|v| Vec3::new(v[x], v[y], v[z]))
            .collect();
        let normals = match (find(&["nx"]), find(&["ny"]), find(&["nz"])) {
            (Some(a), Some(b), Some(c)) => Some(
                ply.vertices
                    .iter()
                    .map(|v| Vec3::new(v[a], v[b], v[c]).normalize())
                    .collect(),
            ),
            _ => None,
        };
        let uvs = match (
            find(&["s", "u", "texture_u"]),
            find(&["t", "v", "texture_v"]),
        ) {
            (Some(a), Some(b)) => Some(ply.vertices.iter().map(|v| [v[a], v[b]]).collect()),
            _ => None,
        };
        let mut triangles = Vec::new();
        for face in &ply.faces {
            for k in 1..face.len().saturating_sub(1) {
                triangles.push([face[0], face[k], face[k + 1]]);
            }
        }
        if triangles.is_empty() {
            bail!("the PLY has no faces");
        }
        let count = positions.len() as u32;
        if triangles.iter().flatten().any(|&i| i >= count) {
            bail!("a face refers past the last vertex");
        }
        Ok(Self::new(positions, normals, uvs, triangles))
    }

    /// Read a PLY surface from a file.
    pub fn load_ply(path: &std::path::Path) -> Result<Self> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        Self::from_ply(&bytes)
    }

    /// The same surface moved by `offset` and scaled by `scale` about the
    /// origin, first scaled then moved.
    pub fn transformed(&self, scale: f32, offset: Vec3) -> Self {
        Self::new(
            self.positions.iter().map(|&p| p * scale + offset).collect(),
            Some(self.normals.clone()),
            Some(self.uvs.clone()),
            self.triangles.clone(),
        )
    }
}

/// Area-weighted vertex normals.
pub fn vertex_normals(positions: &[Vec3], triangles: &[[u32; 3]]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::default(); positions.len()];
    for t in triangles {
        let [a, b, c] = t.map(|i| positions[i as usize]);
        let n = (b - a).cross(c - a);
        for &i in t {
            normals[i as usize] += n;
        }
    }
    normals.iter().map(|n| n.normalize()).collect()
}

/// Barycentric weights of `p` in triangle `a b c`.
pub fn barycentric(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> [f32; 3] {
    let (v0, v1, v2) = (b - a, c - a, p - a);
    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-20 {
        return [1.0, 0.0, 0.0];
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    [1.0 - v - w, v, w]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Surface {
        Surface::new(
            vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, 1.0),
            ],
            None,
            Some(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
            vec![[0, 2, 1], [0, 3, 2]],
        )
    }

    #[test]
    fn samples_land_on_the_surface_with_their_uv() {
        let surface = square();
        assert!((surface.area() - 1.0).abs() < 1e-5);
        let mut rng = Rng::new(1);
        for _ in 0..100 {
            let p = surface.sample(&mut rng);
            assert!(p.position.y.abs() < 1e-6);
            assert!((p.uv[0] - p.position.x).abs() < 1e-4);
            assert!((p.uv[1] - p.position.z).abs() < 1e-4);
            assert!((p.normal.y - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn the_closest_point_interpolates_uv() {
        let surface = square();
        let hit = surface.closest(Vec3::new(0.25, 0.4, 0.75), 1.0).unwrap();
        assert!((hit.uv[0] - 0.25).abs() < 1e-4 && (hit.uv[1] - 0.75).abs() < 1e-4);
    }
}
