//! Boolean regions on a reference body, retaining source-triangle correspondence.
//!
//! Cuts are evaluated once. Barycentric source points subsequently follow every
//! body morph; neither a UV seam nor a changing body can reroute a cut vertex.
use std::collections::BTreeMap;
mod apertures;
mod assembly;
mod cloth_boundary_sampling;
mod cloth_sampling;
mod connected;
mod edge_splits;
mod triangulation;
pub(crate) use apertures::ClothAperture;
pub(crate) use cloth_sampling::{ClothSampling, SamplingAncestry};
pub(crate) use connected::{connected_patch, weld_patch};

/// Interior is the nonpositive half-space, in reference-body metres.
#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub normal: [f32; 3],
    pub offset: f32,
}

/// Intersection of half-spaces. Regions may overlap without duplicating faces.
pub type ConvexRegion = Vec<Plane>;

#[derive(Clone, Copy, Debug)]
pub struct SourcePoint {
    pub triangle: usize,
    pub weights: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct SurfaceCut {
    pub points: Vec<SourcePoint>,
    pub faces: Vec<[u32; 3]>,
    pub(crate) apertures: Vec<ClothAperture>,
}

/// Both sides of an inserted seam remain in the same physical cloth surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PanelRegion {
    Surrounding,
    /// Native ordinal of the construction panel supplied to the partition.
    Panel(usize),
}

/// Native original face ordinal and construction ownership after subdivision.
pub(crate) struct PanelTriangle {
    pub region: PanelRegion,
}

pub(crate) struct SurfacePartition {
    pub surface: SurfaceCut,
    pub triangles: Vec<PanelTriangle>,
}

struct Fragment {
    polygon: Polygon,
    region: PanelRegion,
}

#[derive(Clone, Copy)]
enum CutPrecision {
    SurfaceOpening,
    SewnPanel,
}

type Polygon = Vec<[f64; 3]>;
/// Native GPU words per source point: body indices, UV indices, barycentric
/// f32 bits. Positions evaluated from them are in canonical body metres.
pub const POINT_WORDS: usize = 9;
pub(crate) const CORRESPONDENCE_PRECISION: f32 = 1_000_000.0;
/// Accepted Boolean triangle area squared, in canonical metres to the fourth.
pub(crate) const MINIMUM_TRIANGLE_AREA_SQUARED: f32 = 1e-20;
// Cutting within 0.15 mm of a source vertex uses that vertex exactly. This
// prevents microscopic corner slivers and agrees on both sides of a body edge.
pub(crate) const CUT_VERTEX_SNAP_METRES: f32 = 0.00015;
const CUT_PLANE_ROUNDOFF_METRES: f64 = 1e-6;

impl SurfaceCut {
    pub fn source_table(&self, faces: &[[u32; 3]], uv_faces: &[[u32; 3]]) -> Vec<u32> {
        let mut table = Vec::with_capacity(self.points.len() * POINT_WORDS);
        for point in &self.points {
            table.extend(faces[point.triangle]);
            table.extend(uv_faces[point.triangle]);
            table.extend(point.weights.map(f32::to_bits));
        }
        table
    }

    pub fn new(
        positions: &[[f32; 3]],
        faces: &[[u32; 3]],
        uv_faces: &[[u32; 3]],
        include: &[ConvexRegion],
        subtract: &[ConvexRegion],
    ) -> Self {
        let fragments = faces
            .iter()
            .map(|face| {
                retained_polygons(face.map(|v| positions[v as usize]), include, subtract)
                    .into_iter()
                    .map(|polygon| Fragment {
                        polygon,
                        region: PanelRegion::Surrounding,
                    })
                    .collect()
            })
            .collect::<Vec<_>>();
        assembly::assemble(positions, faces, uv_faces, fragments).surface
    }

    /// Insert a closed panel boundary without removing either fabric. Source
    /// The chart and physical source positions are separate metre coordinates.
    /// A chart can collapse outside its panel without erasing real cloth.
    /// Neighboring
    /// fragments share subdivisions before triangulation and material splitting.
    pub(crate) fn partition_panels(
        chart: &[[f32; 3]],
        surface_positions: &[[f32; 3]],
        faces: &[[u32; 3]],
        uv_faces: &[[u32; 3]],
        panels: &[ConvexRegion],
    ) -> SurfacePartition {
        let fragments = faces
            .iter()
            .map(|face| {
                let source = face.map(|v| chart[v as usize]);
                let mut kept = vec![Fragment {
                    polygon: vec![[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
                    region: PanelRegion::Surrounding,
                }];
                for (ordinal, panel) in panels.iter().enumerate() {
                    kept = kept
                        .into_iter()
                        .flat_map(|fragment| {
                            if fragment.region != PanelRegion::Surrounding {
                                return vec![fragment];
                            }
                            let (inside, outside) =
                                partition(fragment.polygon, panel, source, CutPrecision::SewnPanel);
                            outside
                                .into_iter()
                                .map(|polygon| Fragment {
                                    polygon,
                                    region: PanelRegion::Surrounding,
                                })
                                .chain((inside.len() >= 3).then_some(Fragment {
                                    polygon: inside,
                                    region: PanelRegion::Panel(ordinal),
                                }))
                                .collect()
                        })
                        .collect();
                }
                kept
            })
            .collect();
        assembly::assemble(surface_positions, faces, uv_faces, fragments)
    }

    /// Only physical border edges; UV seams are identified by body position.
    pub fn borders(&self, positions: &[[f32; 3]]) -> Vec<[u32; 2]> {
        let mut welded = BTreeMap::new();
        let ids = positions
            .iter()
            .map(|p| {
                let next = welded.len();
                *welded
                    .entry(p.map(|v| (v * CORRESPONDENCE_PRECISION).round() as i64))
                    .or_insert(next)
            })
            .collect::<Vec<_>>();
        let mut edges = BTreeMap::<[usize; 2], Vec<[u32; 2]>>::new();
        for &[a, b, c] in &self.faces {
            for edge in [[a, b], [b, c], [c, a]] {
                let mut key = edge.map(|v| ids[v as usize]);
                key.sort_unstable();
                edges.entry(key).or_default().push(edge);
            }
        }
        edges
            .into_values()
            .filter_map(|e| (e.len() == 1).then_some(e[0]))
            .collect()
    }
}

fn retained_polygons(
    source: [[f32; 3]; 3],
    include: &[ConvexRegion],
    subtract: &[ConvexRegion],
) -> Vec<Polygon> {
    let mut remaining = vec![vec![[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]];
    let mut kept = Vec::new();
    for volume in include {
        let mut outside = Vec::new();
        for polygon in remaining {
            let (inside, rest) = partition(polygon, volume, source, CutPrecision::SurfaceOpening);
            if inside.len() >= 3 {
                kept.push(inside);
            }
            outside.extend(rest);
        }
        remaining = outside;
    }
    for volume in subtract {
        kept = kept
            .into_iter()
            .flat_map(|p| partition(p, volume, source, CutPrecision::SurfaceOpening).1)
            .collect();
    }
    kept
}

fn partition(
    mut polygon: Polygon,
    volume: &[Plane],
    source: [[f32; 3]; 3],
    precision: CutPrecision,
) -> (Polygon, Vec<Polygon>) {
    // Openings avoid tiny slivers in cloned body triangles. An inserted
    // tailor-rest panel must retain its authored corners at correspondence
    // precision; its two fabrics remain present on both sides of the seam.
    let vertex_snap = match precision {
        CutPrecision::SurfaceOpening => f64::from(CUT_VERTEX_SNAP_METRES),
        CutPrecision::SewnPanel => CUT_PLANE_ROUNDOFF_METRES,
    };
    let planes = volume
        .iter()
        .map(|plane| {
            source.map(|p| {
                let distance = (0..3)
                    .map(|i| f64::from(p[i]) * f64::from(plane.normal[i]))
                    .sum::<f64>()
                    - f64::from(plane.offset);
                if distance.abs() < vertex_snap {
                    0.0
                } else {
                    distance
                }
            })
        })
        .collect::<Vec<_>>();
    // A polygon wholly outside any one half-space cannot meet this convex
    // cutter. Keep it intact instead of splitting it along unrelated planes.
    if planes.iter().any(|&distances| {
        let mut positive = false;
        polygon.iter().all(|&p| {
            let d = clipped_distance(p, distances);
            positive |= d > 0.;
            d >= 0.
        }) && positive
    }) {
        return (Vec::new(), vec![polygon]);
    }
    let mut outside = Vec::new();
    for distances in planes {
        let rejected = clip(&polygon, distances, false);
        if rejected.len() >= 3 {
            outside.push(rejected);
        }
        polygon = clip(&polygon, distances, true);
        if polygon.len() < 3 {
            break;
        }
    }
    (polygon, outside)
}

fn clip(polygon: &Polygon, distances: [f64; 3], inside: bool) -> Polygon {
    let mut result = Vec::new();
    if polygon.is_empty() {
        return result;
    }
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[(i + 1) % polygon.len()];
        // Successive planes can meet on a previously cut edge. Preserve that
        // shared corner instead of inserting a second point a few nm away.
        let da = clipped_distance(a, distances);
        let db = clipped_distance(b, distances);
        let keep_a = if inside { da <= 0.0 } else { da > 0.0 };
        let keep_b = if inside { db <= 0.0 } else { db > 0.0 };
        if keep_a {
            result.push(a);
        }
        if keep_a != keep_b {
            let t = da / (da - db);
            result.push(std::array::from_fn(|j| a[j] + t * (b[j] - a[j])));
        }
    }
    result
}

/// Native barycentric interpolation of signed source-plane metre distances.
fn clipped_distance(point: [f64; 3], distances: [f64; 3]) -> f64 {
    let value: f64 = (0..3).map(|j| point[j] * distances[j]).sum();
    if value.abs() < CUT_PLANE_ROUNDOFF_METRES {
        0.
    } else {
        value
    }
}

pub fn interpolate<const N: usize>(values: [[f32; N]; 3], weights: [f32; 3]) -> [f32; N] {
    std::array::from_fn(|axis| {
        (0..3)
            .map(|i| f64::from(values[i][axis]) * f64::from(weights[i]))
            .sum::<f64>() as f32
    })
}

pub fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| a[i] * b[i]).sum()
}

fn area_squared([a, b, c]: [[f32; 3]; 3]) -> f32 {
    let u: [f32; 3] = std::array::from_fn(|i| b[i] - a[i]);
    let v: [f32; 3] = std::array::from_fn(|i| c[i] - a[i]);
    let cross = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    dot(cross, cross) * 0.25
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successive_plane_roundoff_reuses_the_existing_corner() {
        let polygon = vec![[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
        let result = clip(&polygon, [5e-8, -1., 1.], true);
        assert!(result.contains(&polygon[0]));
        let triangles = triangulation::conforming(vec![result], &[]);
        assert_eq!(triangles.len(), 1);
        assert!(triangles[0].corners.contains(&polygon[0]));
    }

    #[test]
    fn subtraction_splits_faces_and_keeps_interpolated_correspondence() {
        let p = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let cut = vec![Plane {
            normal: [-1.0, 0.0, 0.0],
            offset: -0.5,
        }];
        let result = SurfaceCut::new(&p, &[[0, 1, 2]], &[[0, 1, 2]], &[vec![]], &[cut]);
        assert!(result.faces.len() > 1);
        let points = result
            .points
            .iter()
            .map(|s| interpolate(p, s.weights))
            .collect::<Vec<_>>();
        let area: f32 = result
            .faces
            .iter()
            .map(|f| area_squared(f.map(|i| points[i as usize])).sqrt())
            .sum();
        assert!((area - 0.375).abs() < 1e-6);
        assert!(points.iter().all(|v| v[0] <= 0.5));
        let moved = p.map(|v| [v[0] * 2.0, v[1] + 3.0, v[2]]);
        for s in &result.points {
            let a = interpolate(p, s.weights);
            let b = interpolate(moved, s.weights);
            assert!((b[0] - a[0] * 2.0).abs() < 1e-6);
            assert!((b[1] - a[1] - 3.0).abs() < 1e-6);
        }
    }

    #[test]
    fn overlapping_regions_do_not_duplicate_surface_or_close_uv_seams() {
        let p = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let faces = [[0, 1, 2], [2, 1, 3]];
        let result = SurfaceCut::new(&p, &faces, &[[0, 1, 2], [3, 4, 5]], &[vec![], vec![]], &[]);
        assert_eq!(result.faces.len(), 2);
        assert_eq!(result.points.len(), 6);
        let points = result
            .points
            .iter()
            .map(|s| interpolate(faces[s.triangle].map(|i| p[i as usize]), s.weights))
            .collect::<Vec<_>>();
        assert_eq!(result.borders(&points).len(), 4);
    }
}
