//! The armor's mesh types, and the template tile every lamellar or scale
//! plate is placed from.

use crate::{
    Armor,
    csg::{self, Polygon, Vertex},
};
use fabelgeist_math::vector::{Vec2, Vec3};

/// One welded triangle mesh: per-vertex position, normal and texture
/// coordinate, and triangles indexing them.
#[derive(Debug, Default)]
pub struct ArmorMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub faces: Vec<[u32; 3]>,
}
/// A named piece of the armor.
#[derive(Debug)]
pub struct ArmorPart {
    pub name: String,
    pub mesh: ArmorMesh,
}

fn polygon(points: Vec<Vec3>) -> Polygon {
    let n = (points[1] - points[0])
        .cross(points[2] - points[0])
        .normalize();
    Polygon::new(
        points
            .into_iter()
            .map(|p| Vertex::new(p, n, Vec2::new(p.x, p.y), None))
            .collect(),
    )
}

// Outline is counterclockwise viewed from +Z. Closed, outward-wound solid.
pub(crate) fn extrude(outline: &[[f32; 2]], thickness: f32) -> Vec<Polygon> {
    let points = |z| {
        outline
            .iter()
            .map(|p| Vec3::new(p[0], p[1], z))
            .collect::<Vec<_>>()
    };
    let front = points(thickness * 0.5);
    let mut back = points(-thickness * 0.5);
    back.reverse();
    let mut result = vec![polygon(front.clone()), polygon(back)];
    for i in 0..outline.len() {
        let j = (i + 1) % outline.len();
        let a = front[i];
        let b = front[j];
        result.push(polygon(vec![
            b,
            a,
            Vec3::new(a.x, a.y, -thickness * 0.5),
            Vec3::new(b.x, b.y, -thickness * 0.5),
        ]));
    }
    result
}

pub(crate) fn tile(a: &Armor) -> Vec<Polygon> {
    let p = &a.plate;
    let w = p.width;
    let h = p.height;
    let outline = crate::pattern::plate_outline(p);
    let mut polys = extrude(&outline, a.thickness);
    if p.bevel > 0.0 {
        polys.remove(0);
        let scale = 1.0 - p.bevel * 2.0 / w.min(h);
        let inner = outline
            .iter()
            .map(|v| Vec3::new(v[0] * scale, v[1] * scale, a.thickness * 0.5))
            .collect::<Vec<_>>();
        // The cap and ring form a small rolled highlight at every plate edge.
        polys.push(polygon(inner.clone()));
        for i in 0..outline.len() {
            let j = (i + 1) % outline.len();
            let outer =
                |k: usize| Vec3::new(outline[k][0], outline[k][1], a.thickness * 0.5 - p.bevel);
            polys.push(polygon(vec![inner[i], outer(i), outer(j), inner[j]]));
        }
        // Lower the front edge of the original side walls to meet the bevel.
        for poly in polys.iter_mut().take(outline.len() + 1).skip(1) {
            for v in &mut poly.vertices {
                if v.position.z > 0.0 {
                    v.position.z -= p.bevel;
                }
            }
            *poly = polygon(poly.vertices.iter().map(|v| v.position).collect());
        }
    }
    if p.hole_radius > 0.0 {
        for pair in 0..p.hole_pairs {
            for side in [-1.0, 1.0] {
                let x = side * w * 0.27;
                let y = h * 0.5 - p.hole_radius * 2.2 - pair as f32 * (h * 0.55 / 3.0);
                let circle = (0..12)
                    .map(|i| {
                        let t = i as f32 * std::f32::consts::TAU / 12.0;
                        [x + p.hole_radius * t.cos(), y + p.hole_radius * t.sin()]
                    })
                    .collect::<Vec<_>>();
                polys = csg::subtract(polys, extrude(&circle, a.thickness * 4.0));
            }
        }
    }
    polys
}

#[cfg(test)]
mod tests {
    use super::*;
    fn signed_volume(polys: &[Polygon]) -> f32 {
        polys
            .iter()
            .map(|p| {
                (1..p.vertices.len() - 1)
                    .map(|i| {
                        p.vertices[0]
                            .position
                            .dot(p.vertices[i].position.cross(p.vertices[i + 1].position))
                            / 6.0
                    })
                    .sum::<f32>()
            })
            .sum()
    }
    #[test]
    fn bsp_holes_remove_expected_volume_and_leave_wall_faces() {
        let mut a = Armor::default();
        a.plate.roundness = 0.0;
        a.plate.bevel = 0.0;
        a.plate.hole_pairs = 2;
        let radius = a.plate.hole_radius;
        let cut = tile(&a);
        a.plate.hole_radius = 0.0;
        let solid = tile(&a);
        let removed = signed_volume(&solid) - signed_volume(&cut);
        let expected =
            4.0 * 6.0 * radius * radius * (std::f32::consts::TAU / 12.0).sin() * a.thickness;
        assert!(
            (removed - expected).abs() < expected * 0.01,
            "removed {removed}, expected {expected}"
        );
        let x = a.plate.width * 0.27;
        let y = a.plate.height * 0.5 - radius * 2.2;
        // A ray through a hole center must miss every front/back triangle.
        for p in &cut {
            if p.plane.normal.z.abs() < 0.9 {
                continue;
            }
            for i in 1..p.vertices.len() - 1 {
                let tri = [
                    p.vertices[0].position,
                    p.vertices[i].position,
                    p.vertices[i + 1].position,
                ];
                let signs = (0..3)
                    .map(|j| {
                        let u = tri[j];
                        let v = tri[(j + 1) % 3];
                        (v.x - u.x) * (y - u.y) - (v.y - u.y) * (x - u.x)
                    })
                    .collect::<Vec<_>>();
                assert!(
                    !(signs.iter().all(|s| *s > 1e-10) || signs.iter().all(|s| *s < -1e-10)),
                    "hole is capped"
                );
            }
        }
    }
}
