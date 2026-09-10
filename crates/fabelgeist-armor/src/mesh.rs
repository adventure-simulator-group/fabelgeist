use crate::{
    Armor, Construction,
    csg::{self, Polygon, Vertex},
};
use fabelgeist_math::vector::{Vec2, Vec3};

#[derive(Debug, Default)]
pub struct ArmorMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub faces: Vec<[u32; 3]>,
}
impl ArmorMesh {
    fn weld(&mut self) {
        let mut vertices = std::collections::HashMap::new();
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut remap = Vec::with_capacity(self.positions.len());
        for i in 0..self.positions.len() {
            let p = self.positions[i];
            let n = self.normals[i];
            let uv = self.uvs[i];
            let key = [p[0], p[1], p[2], n[0], n[1], n[2], uv[0], uv[1]]
                .map(|v| (v * 1_000_000.0).round() as i64);
            let index = *vertices.entry(key).or_insert_with(|| {
                let index = positions.len() as u32;
                positions.push(p);
                normals.push(n);
                uvs.push(uv);
                index
            });
            remap.push(index);
        }
        for face in &mut self.faces {
            for index in face {
                *index = remap[*index as usize];
            }
        }
        self.faces
            .retain(|f| f[0] != f[1] && f[1] != f[2] && f[2] != f[0]);
        self.positions = positions;
        self.normals = normals;
        self.uvs = uvs;
    }
}
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
fn extrude(outline: &[[f32; 2]], thickness: f32) -> Vec<Polygon> {
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

fn tile(a: &Armor) -> Vec<Polygon> {
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

fn surface(a: &Armor, x: f32, y: f32, z: f32, flare: f32) -> Vec3 {
    let t = (y / a.height).clamp(0.0, 1.0);
    let width = a.width * (a.waist + (1.0 - a.waist) * (t * std::f32::consts::FRAC_PI_2).sin())
        + flare * 2.0;
    let u = (x / (a.width * 0.5)).clamp(-1.0, 1.0);
    let bulge = a.depth * (1.0 - u * u).max(0.0).sqrt();
    let ridge =
        a.ridge * (1.0 - u.abs()).powf(a.ridge_sharpness) * (std::f32::consts::PI * t).sin();
    Vec3::new(
        u * width * 0.5 + a.translation[0],
        y + a.translation[1],
        bulge + ridge + z + a.translation[2] + flare * 0.5,
    )
}

fn emit(mesh: &mut ArmorMesh, tri: [Vec3; 3], map: &impl Fn(Vec3) -> Vec3, divisions: u32) {
    if divisions > 0 {
        let [a, b, c] = tri;
        let ab = (a + b) * 0.5;
        let bc = (b + c) * 0.5;
        let ca = (c + a) * 0.5;
        for t in [[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]] {
            emit(mesh, t, map, divisions - 1);
        }
        return;
    }
    let p = tri.map(map);
    let cross = (p[1] - p[0]).cross(p[2] - p[0]);
    if cross.length_squared() < 1e-16 {
        return;
    }
    let flat = cross.normalize();
    let source_normal = (tri[1] - tri[0]).cross(tri[2] - tri[0]).normalize();
    let start = mesh.positions.len() as u32;
    for i in 0..3 {
        let normal = if source_normal.z.abs() > 0.99 {
            let v = tri[i];
            let e = 0.00001;
            let dx = map(v + Vec3::new(e, 0.0, 0.0)) - map(v - Vec3::new(e, 0.0, 0.0));
            let dy = map(v + Vec3::new(0.0, e, 0.0)) - map(v - Vec3::new(0.0, e, 0.0));
            dx.cross(dy).normalize() * source_normal.z.signum()
        } else {
            flat
        };
        mesh.positions.push([p[i].x, p[i].y, p[i].z]);
        mesh.normals.push([normal.x, normal.y, normal.z]);
        mesh.uvs.push(if source_normal.z.abs() > 0.99 {
            [tri[i].x * 4.0, tri[i].y * 4.0]
        } else if source_normal.x.abs() > source_normal.y.abs() {
            [tri[i].y * 4.0, tri[i].z * 4.0]
        } else {
            [tri[i].x * 4.0, tri[i].z * 4.0]
        });
    }
    mesh.faces.push([start, start + 1, start + 2]);
}
fn mapped(polys: &[Polygon], map: impl Fn(Vec3) -> Vec3, divisions: u32) -> ArmorMesh {
    let mut mesh = ArmorMesh::default();
    for p in polys {
        for i in 1..p.vertices.len() - 1 {
            emit(
                &mut mesh,
                [
                    p.vertices[0].position,
                    p.vertices[i].position,
                    p.vertices[i + 1].position,
                ],
                &map,
                divisions,
            );
        }
    }
    mesh
}

pub fn build(a: &Armor) -> Result<Vec<ArmorPart>, String> {
    a.validate()?;
    let mut parts = Vec::new();
    if a.construction == Construction::Solid {
        // Grid cells follow the neckline and arm openings without concave caps.
        let mut mesh = ArmorMesh::default();
        let nx = 32;
        let ny = 24;
        for x in 0..nx {
            for y in 0..ny {
                let point = |i: i32, j: i32, z: f32| {
                    let u = i as f32 / nx as f32 * 2.0 - 1.0;
                    let t = j as f32 / ny as f32;
                    let top = a.height - a.neck * (1.0 - (u.abs() / 0.65).min(1.0).powi(2));
                    let bottom = -a.center_point * (1.0 - u.abs());
                    let xx = u * (a.width * 0.5 - a.arm_cut * t.powi(4));
                    Vec3::new(xx, bottom + (top - bottom) * t, z)
                };
                let mut quad = |coords: [(i32, i32, f32); 4]| {
                    let p = coords.map(|(i, j, z)| point(i, j, z));
                    for tri in [[p[0], p[1], p[2]], [p[0], p[2], p[3]]] {
                        emit(&mut mesh, tri, &|v| surface(a, v.x, v.y, v.z, 0.0), 0);
                    }
                };
                let z = a.thickness * 0.5;
                quad([(x, y, z), (x + 1, y, z), (x + 1, y + 1, z), (x, y + 1, z)]);
                quad([
                    (x, y, -z),
                    (x, y + 1, -z),
                    (x + 1, y + 1, -z),
                    (x + 1, y, -z),
                ]);
                if x == 0 {
                    quad([(x, y, z), (x, y + 1, z), (x, y + 1, -z), (x, y, -z)]);
                }
                if x == nx - 1 {
                    quad([
                        (x + 1, y + 1, z),
                        (x + 1, y, z),
                        (x + 1, y, -z),
                        (x + 1, y + 1, -z),
                    ]);
                }
                if y == 0 {
                    quad([(x + 1, y, z), (x, y, z), (x, y, -z), (x + 1, y, -z)]);
                }
                if y == ny - 1 {
                    quad([
                        (x, y + 1, z),
                        (x + 1, y + 1, z),
                        (x + 1, y + 1, -z),
                        (x, y + 1, -z),
                    ]);
                }
            }
        }
        parts.push(ArmorPart {
            name: "Breastplate".into(),
            mesh,
        });
    } else {
        let tile = tile(a);
        let p = &a.plate;
        let rows = ((a.height - p.height) / (p.height * (1.0 - p.overlap)))
            .ceil()
            .max(0.0) as u32
            + 1;
        for row in 0..rows {
            let y = a.height
                - p.height * 0.5
                - row as f32 * (a.height - p.height) / (rows - 1).max(1) as f32;
            let usable = a.width - 2.0 * a.arm_cut * (y / a.height).powi(4);
            let pitch = p.width + p.gap;
            let cols = ((usable + p.gap) / pitch).floor().max(1.0) as u32;
            let stagger = if row % 2 == 1 { p.stagger } else { 0.0 };
            let count = if stagger > 0.0 && cols > 1 {
                cols - 1
            } else {
                cols
            };
            for col in 0..count {
                let x = (col as f32 - (cols - 1) as f32 * 0.5 + stagger) * pitch;
                let top =
                    a.height - a.neck * (1.0 - (x.abs() / (a.width * 0.325)).min(1.0).powi(2));
                if y + p.height * 0.5 > top + 0.001 {
                    continue;
                }
                let z = (rows - row) as f32 * (a.thickness + 0.0006);
                let mesh = mapped(&tile, |v| surface(a, v.x + x, v.y + y, v.z + z, 0.0), 0);
                parts.push(ArmorPart {
                    name: format!("Plate {}:{}", row + 1, col + 1),
                    mesh,
                });
            }
        }
    }
    let mut bottom = 0.0;
    let mut flare = 0.0;
    for i in 0..a.fauld.layer_count as usize {
        bottom -= a.fauld.layer_height * (1.0 - a.fauld.overlap);
        flare += a.fauld.flare;
        let z = (a.fauld.layer_count as usize - i) as f32 * (a.thickness + 0.001);
        let mesh = if a.fauld.construction == Construction::Solid {
            let polys = extrude(
                &[
                    [-a.width * 0.5, 0.0],
                    [a.width * 0.5, 0.0],
                    [a.width * 0.5, a.fauld.layer_height],
                    [-a.width * 0.5, a.fauld.layer_height],
                ],
                a.thickness,
            );
            mapped(&polys, |v| surface(a, v.x, v.y + bottom, v.z + z, flare), 4)
        } else {
            let p = &a.plate;
            let rows = (a.fauld.layer_height / (p.height * (1.0 - p.overlap)))
                .ceil()
                .max(1.0) as u32;
            let cols = ((a.width + p.gap) / (p.width + p.gap)).floor().max(1.0) as u32;
            let polys = tile(a);
            let mut mesh = ArmorMesh::default();
            for row in 0..rows {
                let y = bottom + a.fauld.layer_height - p.height * 0.5
                    - row as f32 * p.height * (1.0 - p.overlap);
                if y - p.height * 0.5 < bottom - 0.001 {
                    continue;
                }
                let stagger = if row % 2 == 1 { p.stagger } else { 0.0 };
                for col in 0..cols {
                    let x = (col as f32 - (cols - 1) as f32 * 0.5 + stagger) * (p.width + p.gap);
                    let tile_mesh = mapped(
                        &polys,
                        |v| surface(a, v.x + x, v.y + y, v.z + z, flare),
                        0,
                    );
                    let offset = mesh.positions.len() as u32;
                    mesh.positions.extend(tile_mesh.positions);
                    mesh.normals.extend(tile_mesh.normals);
                    mesh.uvs.extend(tile_mesh.uvs);
                    mesh.faces.extend(tile_mesh.faces.into_iter().map(|f| f.map(|n| n + offset)));
                }
            }
            mesh
        };
        parts.push(ArmorPart {
            name: format!("Fauld layer {}", i + 1),
            mesh,
        });
    }
    for part in &mut parts {
        part.mesh.weld();
    }
    Ok(parts)
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
    #[test]
    fn all_constructions_have_finite_outward_surfaces() {
        for construction in [
            Construction::Solid,
            Construction::Lamellar,
            Construction::Scale,
        ] {
            let a = Armor {
                construction,
                ..Armor::default()
            };
            let parts = build(&a).unwrap();
            assert!(parts.len() > a.fauld.layer_count as usize);
            assert!(parts.iter().map(|p| p.mesh.positions.len()).sum::<usize>() < 250_000);
            for part in &parts {
                let m = &part.mesh;
                assert!(!m.faces.is_empty());
                assert!(
                    m.faces
                        .iter()
                        .flatten()
                        .all(|i| (*i as usize) < m.positions.len())
                );
                assert_eq!(m.positions.len(), m.normals.len());
                assert!(m.positions.iter().flatten().all(|v| v.is_finite()));
                assert!(m.normals.iter().flatten().all(|v| v.is_finite()));
                for n in &m.normals {
                    assert!((n.iter().map(|x| x * x).sum::<f32>() - 1.0).abs() < 0.01);
                }
            }
        }
    }
    #[test]
    fn ridge_and_layers_change_geometry() {
        let mut a = Armor::default();
        a.fauld.layer_count = 0;
        a.ridge = 0.0;
        let flat = build(&a).unwrap();
        a.ridge = 0.1;
        let ridged = build(&a).unwrap();
        assert!(
            flat[0]
                .mesh
                .positions
                .iter()
                .zip(&ridged[0].mesh.positions)
                .any(|(p, q)| q[2] - p[2] > 0.09)
        );
        a.fauld.layer_count = 1;
        a.fauld.layer_height = 0.1;
        a.fauld.overlap = 0.25;
        a.fauld.flare = 0.02;
        assert_eq!(build(&a).unwrap().len(), 2);
        a.width = f32::NAN;
        assert!(build(&a).is_err());
    }
    #[test]
    fn fauld_constructions_generate_meshes() {
        for construction in [
            Construction::Solid,
            Construction::Lamellar,
            Construction::Scale,
        ] {
            let mut a = Armor::default();
            a.fauld.construction = construction;
            if construction == Construction::Scale {
                a.plate.roundness = 1.0;
            }
            let parts = build(&a).unwrap();
            let fauld = parts
                .iter()
                .filter(|part| part.name.starts_with("Fauld layer"))
                .collect::<Vec<_>>();
            assert_eq!(fauld.len(), a.fauld.layer_count as usize);
            assert!(fauld.iter().all(|part| !part.mesh.faces.is_empty()));
        }
    }
}
