//! Extrema over clipped production vista triangles, including LOD vertex morphs.
use super::*;
use adventuresim_tactical_core::{
    prelude::VistaLod,
    vista_surface::{
        cell_rectangles_outside_inner_rectangle, presented_vista_vertex_height,
        subdivide_playable_boundary_rectangle,
    },
};
use bevy::math::Vec3Swizzles;

pub(crate) fn extrema(
    lod: &VistaLod,
    coarser: Option<&VistaLod>,
    terrain: &SceneTerrain,
    corners: [Vec2; 4],
) -> Value {
    let mut result = Intersection::default();
    for triangle in triangles(lod, coarser, terrain, corners) {
        let points = triangle.map(|p| p.xz());
        let polygon = clip(points.to_vec(), &corners);
        result.area_m2 += area(&polygon);
        for point in polygon {
            result.observe(point, height_in_triangle(triangle, point));
        }
    }
    json!({"required_footprint_area_m2":area(&corners),"clipped_vista_area_m2":result.area_m2,
        "intersection_vertices":result.vertices,
        "minimum_metres":result.low.map(|(_,h)|h),"minimum_location":result.low.map(|(p,_)|p),
        "maximum_metres":result.high.map(|(_,h)|h),"maximum_location":result.high.map(|(p,_)|p),
        "method":"Production clipped vista triangles with playable-boundary subdivision and LOD vertex morph. Area coverage is reported explicitly; playable terrain and other LODs are not substituted."})
}

pub(crate) fn triangles(
    lod: &VistaLod,
    coarser: Option<&VistaLod>,
    terrain: &SceneTerrain,
    corners: [Vec2; 4],
) -> Vec<[bevy::math::Vec3; 3]> {
    let origin = Vec2::new(
        lod.origin_east_metres as f32,
        lod.origin_north_metres as f32,
    );
    let corners = corners.map(|p| p - origin);
    let size = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1));
    let min = ((corners
        .into_iter()
        .fold(Vec2::splat(f32::INFINITY), Vec2::min)
        / lod.spacing_metres
        + size * 0.5)
        .floor())
    .max(Vec2::ZERO);
    let max = ((corners
        .into_iter()
        .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max)
        / lod.spacing_metres
        + size * 0.5)
        .ceil())
    .min(size);
    let half = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
    let mut result = Vec::new();
    for z in min.y as usize..max.y as usize {
        for x in min.x as usize..max.x as usize {
            let a = (Vec2::new(x as f32, z as f32) - size * 0.5) * lod.spacing_metres;
            let b = a + Vec2::splat(lod.spacing_metres);
            for rectangle in cell_rectangles_outside_inner_rectangle(a, b, half) {
                for [x0, x1, z0, z1] in
                    subdivide_playable_boundary_rectangle(rectangle, half, Some(terrain))
                {
                    let quad = [
                        Vec2::new(x0, z0),
                        Vec2::new(x1, z0),
                        Vec2::new(x1, z1),
                        Vec2::new(x0, z1),
                    ];
                    let Some(heights) = quad
                        .into_iter()
                        .map(|p| {
                            presented_vista_vertex_height(lod, coarser, Some(terrain), p, half)
                        })
                        .collect::<Option<Vec<_>>>()
                    else {
                        continue;
                    };
                    // Vista uses the a--c diagonal, unlike the playable field.
                    for indices in [[0, 1, 2], [0, 2, 3]] {
                        result.push(indices.map(|i| {
                            let point = quad[i] + origin;
                            bevy::math::Vec3::new(point.x, heights[i], point.y)
                        }));
                    }
                }
            }
        }
    }
    result
}

pub(crate) fn height_in_triangle(triangle: [bevy::math::Vec3; 3], point: Vec2) -> f32 {
    let [a, b, c] = triangle;
    let ab = b.xz() - a.xz();
    let ac = c.xz() - a.xz();
    let ap = point - a.xz();
    let determinant = ab.perp_dot(ac);
    let u = ap.perp_dot(ac) / determinant;
    let v = ab.perp_dot(ap) / determinant;
    a.y + (b.y - a.y) * u + (c.y - a.y) * v
}

#[derive(Default)]
struct Intersection {
    area_m2: f64,
    vertices: usize,
    low: Option<(Vec2, f32)>,
    high: Option<(Vec2, f32)>,
}
impl Intersection {
    fn observe(&mut self, point: Vec2, height: f32) {
        self.vertices += 1;
        if self.low.is_none_or(|(_, h)| height < h) {
            self.low = Some((point, height));
        }
        if self.high.is_none_or(|(_, h)| height > h) {
            self.high = Some((point, height));
        }
    }
}
fn area(polygon: &[Vec2]) -> f64 {
    let Some(origin) = polygon.first() else {
        return 0.0;
    };
    polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
        .map(|(a, b)| {
            let a = (*a - *origin).as_dvec2();
            let b = (*b - *origin).as_dvec2();
            a.perp_dot(b)
        })
        .sum::<f64>()
        .abs()
        * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vista_footprint_finds_interior_peak_and_accounts_for_complete_area() {
        let mut heights = vec![0.0; 81];
        heights[20] = 10.0;
        let lod = VistaLod {
            level: adventuresim_tactical_core::scene_input::VistaLevelIndex::new(0),
            spacing_metres: 50.0,
            width: 9,
            depth: 9,
            origin_east_metres: 500.0,
            origin_north_metres: 500.0,
            heights_metres: heights,
            environment: vec![],
        };
        let terrain = SceneTerrain::from_heightmap(2, 2, 1.0, vec![0.0; 4]).unwrap();
        let corners = [
            Vec2::new(400.0, 355.0),
            Vec2::new(445.0, 400.0),
            Vec2::new(400.0, 445.0),
            Vec2::new(355.0, 400.0),
        ];
        let report = extrema(&lod, None, &terrain, corners);
        assert_eq!(report["maximum_metres"], 10.0);
        assert!((report["clipped_vista_area_m2"].as_f64().unwrap() - 4050.0).abs() < 0.05);
        assert_eq!(report["required_footprint_area_m2"], 4050.0);
    }
    #[test]
    fn vista_diagonal_is_distinct_from_the_playable_collision_diagonal() {
        let lod = VistaLod {
            level: adventuresim_tactical_core::scene_input::VistaLevelIndex::new(0),
            spacing_metres: 50.0,
            width: 2,
            depth: 2,
            origin_east_metres: 500.0,
            origin_north_metres: 500.0,
            heights_metres: vec![0.0, 2.0, 2.0, 0.0],
            environment: vec![],
        };
        let terrain = SceneTerrain::from_heightmap(2, 2, 0.001, vec![0.0; 4]).unwrap();
        let corners = [
            Vec2::new(499.9, 499.9),
            Vec2::new(500.1, 499.9),
            Vec2::new(500.1, 500.1),
            Vec2::new(499.9, 500.1),
        ];
        let report = extrema(&lod, None, &terrain, corners);
        assert!(report["maximum_metres"].as_f64().unwrap() < 0.01);
        assert_eq!(report["minimum_metres"], 0.0);
    }
}
