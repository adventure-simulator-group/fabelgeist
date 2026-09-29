//! The template every lamellar or scale plate is placed from: its outline
//! extruded to its gauge, its front edge bevelled and its lacing holes
//! pierced through it.
//!
//! The front and back faces are triangulated with their outline and holes
//! as constraints, and the walls are built along the same boundary points,
//! so the plate is closed without any cut splitting its faces. Each wall
//! facet keeps its own flat normal.
//!
//! The template lies in the plate's own frame: `x` across the row, `y` up
//! towards the row above, `z` out of the piece, centred on its middle.

use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

use crate::{Plate, PlateFace};

/// Sides of the polygon a lacing hole is cut as.
const HOLE_SIDES: usize = 12;
/// Spacing of the points seeded inside each face, as a share of the plate's
/// narrower side. The faces bend with the surface between them, and a trim
/// band finds its inner border across them.
const INTERIOR_SPACING: f32 = 0.125;
/// Seeded points keep this share of their spacing clear of every boundary,
/// so no sliver forms against it.
const INTERIOR_CLEARANCE: f32 = 0.4;

/// One corner of the template.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TemplateVertex {
    pub position: [f32; 3],
    /// In the plate's frame; flat across each wall facet.
    pub normal: [f32; 3],
    /// Metres across the face it lies on.
    pub texcoord: [f32; 2],
}

/// The plate as an indexed mesh.
#[derive(Clone, Debug, Default)]
pub(super) struct TileTemplate {
    pub vertices: Vec<TemplateVertex>,
    pub triangles: Vec<[u32; 3]>,
    /// The plate face of each triangle.
    pub faces: Vec<PlateFace>,
}

impl TileTemplate {
    /// `None` when a hole reaches the plate's outline.
    pub(super) fn new(plate: &Plate) -> Option<Self> {
        let outline = crate::pattern::plate_outline(plate);
        let holes = plate
            .holes()
            .into_iter()
            .map(|center| circle(center, plate.hole_radius))
            .collect::<Vec<_>>();
        let [front, back] = [0.5, -0.5].map(|side| plate.thickness * side);
        let rim = front - plate.bevel;
        let cap = if plate.bevel > 0.0 {
            let scale = 1.0 - plate.bevel * 2.0 / plate.width.min(plate.height);
            outline.iter().map(|p| p.map(|x| x * scale)).collect()
        } else {
            outline.clone()
        };
        let mut template = Self::default();
        let spacing = plate.width.min(plate.height) * INTERIOR_SPACING;
        template.face(&cap, &holes, front, PlateFace::Outer, spacing)?;
        template.face(&outline, &holes, back, PlateFace::Inner, spacing)?;
        for (a, b) in edges(&outline) {
            template.quad([(a, back), (b, back), (b, rim), (a, rim)]);
        }
        if plate.bevel > 0.0 {
            for ((a, b), (c, d)) in edges(&outline).zip(edges(&cap)) {
                template.quad([(a, rim), (b, rim), (d, front), (c, front)]);
            }
        }
        for hole in &holes {
            // The material lies outside the hole, so its wall faces inward.
            for (a, b) in edges(hole) {
                template.quad([(b, back), (a, back), (a, front), (b, front)]);
            }
        }
        Some(template)
    }

    fn push(&mut self, position: [f32; 3], normal: [f32; 3]) -> u32 {
        let texcoord = if normal[2].abs() > normal[0].abs().max(normal[1].abs()) {
            [position[0], position[1]]
        } else if normal[0].abs() > normal[1].abs() {
            [position[1], position[2]]
        } else {
            [position[0], position[2]]
        };
        self.vertices.push(TemplateVertex {
            position,
            normal,
            texcoord,
        });
        self.vertices.len() as u32 - 1
    }

    /// A flat face at height `z` inside `outline` and outside `holes`, facing
    /// out of the plate, with points seeded `spacing` apart inside it. `None`
    /// when the holes cross the outline.
    fn face(
        &mut self,
        outline: &[[f32; 2]],
        holes: &[Vec<[f32; 2]>],
        z: f32,
        face: PlateFace,
        spacing: f32,
    ) -> Option<()> {
        let mut points = Vec::new();
        let mut constraints = Vec::new();
        for ring in std::iter::once(outline).chain(holes.iter().map(Vec::as_slice)) {
            let first = points.len();
            points.extend(
                ring.iter()
                    .map(|p| Point2::new(f64::from(p[0]), f64::from(p[1]))),
            );
            constraints.extend((0..ring.len()).map(|i| [first + i, first + (i + 1) % ring.len()]));
        }
        points.extend(
            interior(outline, holes, spacing).map(|[x, y]| Point2::new(f64::from(x), f64::from(y))),
        );
        let cdt =
            ConstrainedDelaunayTriangulation::<Point2<f64>>::bulk_load_cdt(points, constraints)
                .ok()?;
        let up = face == PlateFace::Outer;
        let normal = [0.0, 0.0, if up { 1.0 } else { -1.0 }];
        let first = self.vertices.len() as u32;
        for vertex in cdt.vertices() {
            let p = vertex.position();
            self.push([p.x as f32, p.y as f32, z], normal);
        }
        for triangle in cdt.inner_faces() {
            let corners = triangle.vertices();
            let [a, b, c] = corners.map(|v| v.position());
            let centroid = [(a.x + b.x + c.x) / 3.0, (a.y + b.y + c.y) / 3.0].map(|x| x as f32);
            if !inside(outline, centroid) || holes.iter().any(|hole| inside(hole, centroid)) {
                continue;
            }
            // Faces come counterclockwise, seen from the front.
            let [a, b, c] = corners.map(|v| first + v.fix().index() as u32);
            self.triangles.push(if up { [a, b, c] } else { [a, c, b] });
            self.faces.push(face);
        }
        Some(())
    }

    /// A flat wall facet through boundary points at heights, counterclockwise
    /// seen from outside the plate.
    fn quad(&mut self, corners: [([f32; 2], f32); 4]) {
        let points = corners.map(|([x, y], z)| [x, y, z]);
        let [a, b, c] = [points[0], points[1], points[3]];
        let u = std::array::from_fn::<f32, 3, _>(|k| b[k] - a[k]);
        let v = std::array::from_fn::<f32, 3, _>(|k| c[k] - a[k]);
        let n = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let length = n.iter().map(|x| x * x).sum::<f32>().sqrt();
        if length.is_nan() || length <= 0.0 {
            return;
        }
        let normal = n.map(|x| x / length);
        let ids = points.map(|p| self.push(p, normal));
        self.triangles
            .extend([[ids[0], ids[1], ids[2]], [ids[0], ids[2], ids[3]]]);
        self.faces.extend([PlateFace::Edge; 2]);
    }
}

/// Points on a square lattice `spacing` apart inside `outline`, outside
/// `holes`, and clear of both.
fn interior<'a>(
    outline: &'a [[f32; 2]],
    holes: &'a [Vec<[f32; 2]>],
    spacing: f32,
) -> impl Iterator<Item = [f32; 2]> + 'a {
    let [low, high] = [f32::min, f32::max]
        .map(|pick| [0, 1].map(|k| outline.iter().map(|p| p[k]).fold(outline[0][k], pick)));
    let steps = move |k: usize| ((high[k] - low[k]) / spacing).floor() as usize;
    let clearance = spacing * INTERIOR_CLEARANCE;
    let rings = std::iter::once(outline).chain(holes.iter().map(Vec::as_slice));
    let boundary = rings.flat_map(edges).collect::<Vec<_>>();
    (1..steps(1))
        .flat_map(move |j| (1..steps(0)).map(move |i| (i, j)))
        .filter_map(move |(i, j)| {
            let point = [low[0] + i as f32 * spacing, low[1] + j as f32 * spacing];
            let clear = boundary
                .iter()
                .all(|(a, b)| distance_to_segment(point, *a, *b) >= clearance);
            (clear && inside(outline, point) && !holes.iter().any(|hole| inside(hole, point)))
                .then_some(point)
        })
}

fn distance_to_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let length = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if length > 0.0 {
        (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let d = [p[0] - a[0] - ab[0] * t, p[1] - a[1] - ab[1] * t];
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// Each edge of the closed polygon `ring`.
fn edges(ring: &[[f32; 2]]) -> impl Iterator<Item = ([f32; 2], [f32; 2])> + '_ {
    (0..ring.len()).map(|i| (ring[i], ring[(i + 1) % ring.len()]))
}

/// A lacing hole as a polygon, counterclockwise.
fn circle([x, y]: [f32; 2], radius: f32) -> Vec<[f32; 2]> {
    (0..HOLE_SIDES)
        .map(|i| {
            let t = i as f32 * std::f32::consts::TAU / HOLE_SIDES as f32;
            [x + radius * t.cos(), y + radius * t.sin()]
        })
        .collect()
}

/// Whether `point` lies inside the closed polygon `ring`.
fn inside(ring: &[[f32; 2]], [x, y]: [f32; 2]) -> bool {
    let mut inside = false;
    for ([ax, ay], [bx, by]) in edges(ring) {
        if (ay > y) != (by > y) && x < ax + (y - ay) * (bx - ax) / (by - ay) {
            inside = !inside;
        }
    }
    inside
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::Tiling;

    /// The template's signed volume, and whether its triangles close over
    /// shared positions with every edge met once each way.
    fn closure(template: &TileTemplate) -> (f32, bool) {
        let position = |i: u32| template.vertices[i as usize].position;
        let key = |p: [f32; 3]| p.map(|x| (x * 1e6).round() as i64);
        let mut edges = HashMap::<([i64; 3], [i64; 3]), i32>::new();
        let mut volume = 0.0;
        for [a, b, c] in &template.triangles {
            let [a, b, c] = [*a, *b, *c].map(position);
            volume += (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0;
            for (from, to) in [(a, b), (b, c), (c, a)] {
                *edges.entry((key(from), key(to))).or_default() += 1;
                *edges.entry((key(to), key(from))).or_default() -= 1;
            }
        }
        (volume, edges.values().all(|uses| *uses == 0))
    }

    #[test]
    fn plates_close_and_their_holes_take_their_volume() {
        for mut plate in [Tiling::scale().plate, Tiling::lamellar().plate] {
            let pierced = TileTemplate::new(&plate).unwrap();
            let (pierced_volume, closed) = closure(&pierced);
            assert!(closed, "the plate is not closed");
            let holes = plate.holes().len() as f32;
            let radius = plate.hole_radius;
            plate.hole_radius = 0.0;
            let (solid_volume, closed) = closure(&TileTemplate::new(&plate).unwrap());
            assert!(closed);
            let polygon =
                HOLE_SIDES as f32 * 0.5 * (std::f32::consts::TAU / HOLE_SIDES as f32).sin();
            let expected = holes * polygon * radius * radius * plate.thickness;
            let removed = solid_volume - pierced_volume;
            assert!(
                (removed - expected).abs() < expected * 0.01,
                "{removed} vs {expected}"
            );
        }
    }

    #[test]
    fn no_face_covers_a_hole() {
        let plate = Tiling::lamellar().plate;
        let template = TileTemplate::new(&plate).unwrap();
        for [x, y] in plate.holes() {
            for (triangle, face) in template.triangles.iter().zip(&template.faces) {
                if *face == PlateFace::Edge {
                    continue;
                }
                let ring = triangle.map(|i| {
                    let p = template.vertices[i as usize].position;
                    [p[0], p[1]]
                });
                assert!(!inside(&ring, [x, y]), "a face covers the hole at {x}, {y}");
            }
        }
    }

    #[test]
    fn the_faces_keep_their_own_normals() {
        let template = TileTemplate::new(&Tiling::scale().plate).unwrap();
        for (triangle, face) in template.triangles.iter().zip(&template.faces) {
            for corner in triangle {
                let normal = template.vertices[*corner as usize].normal;
                match face {
                    PlateFace::Outer => assert_eq!(normal, [0.0, 0.0, 1.0]),
                    PlateFace::Inner => assert_eq!(normal, [0.0, 0.0, -1.0]),
                    PlateFace::Edge => assert!(normal[2].abs() < 1.0),
                }
            }
        }
    }
}
