//! Assemble conforming cut fragments while retaining their sewing ownership.
use super::{
    CORRESPONDENCE_PRECISION, Fragment, MINIMUM_TRIANGLE_AREA_SQUARED, PanelTriangle, SourcePoint,
    SurfaceCut, SurfacePartition, area_squared, edge_splits, interpolate, triangulation,
};
use std::collections::BTreeMap;

/// Native reference-chart metres, source indices and frozen barycentric samples.
pub(super) fn assemble(
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
    uv_faces: &[[u32; 3]],
    fragments: Vec<Vec<Fragment>>,
) -> SurfacePartition {
    let mut result = SurfacePartition {
        surface: SurfaceCut {
            apertures: Vec::new(),
            points: Vec::new(),
            faces: Vec::new(),
        },
        triangles: Vec::new(),
    };
    let mut vertices = BTreeMap::new();
    let shared = edge_splits::EdgeSplits::new(faces, &fragments);
    for (triangle, (face, kept)) in faces.iter().zip(fragments).enumerate() {
        let source = face.map(|v| positions[v as usize]);
        let regions = kept.iter().map(|f| f.region).collect::<Vec<_>>();
        let polygons = kept.into_iter().map(|f| f.polygon).collect();
        for fragment in triangulation::conforming(polygons, &shared.points(face)) {
            let corners = fragment.corners.map(|w| w.map(|v| v as f32));
            let p = corners.map(|w| interpolate(source, w));
            if area_squared(p) <= MINIMUM_TRIANGLE_AREA_SQUARED {
                continue;
            }
            let indices = corners.map(|weights| {
                let mut key = (0..3)
                    .filter_map(|i| {
                        let weight = (weights[i] * CORRESPONDENCE_PRECISION).round() as i32;
                        (weight > 0).then_some((face[i], uv_faces[triangle][i], weight))
                    })
                    .collect::<Vec<_>>();
                key.sort_unstable();
                *vertices.entry(key).or_insert_with(|| {
                    let index = result.surface.points.len() as u32;
                    result
                        .surface
                        .points
                        .push(SourcePoint { triangle, weights });
                    index
                })
            });
            if indices[0] != indices[1] && indices[1] != indices[2] && indices[2] != indices[0] {
                result.surface.faces.push(indices);
                result.triangles.push(PanelTriangle {
                    region: regions[fragment.polygon],
                });
            }
        }
    }
    result
}
