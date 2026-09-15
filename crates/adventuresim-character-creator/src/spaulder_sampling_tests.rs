use super::*;
use adventuresim_armor_model::{ArmorDetail, ArmorLod, Permille};

#[test]
fn runtime_crowns_approximate_curved_source_across_coverage_and_dimensions() {
    for lod in [ArmorLod::Lod4, ArmorLod::Lod5, ArmorLod::Lod6] {
        for coverage in [200, 400, 650, 999, 1000] {
            for dimensions in [
                [0.04_f32, 0.07, 0.05],
                [0.07, 0.14, 0.06],
                [0.1, 0.11, 0.09],
            ] {
                let frame = PartFrame {
                    detail: ArmorDetail::Runtime(lod),
                    origin: [0.0; 3],
                    axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                    half_extents: dimensions,
                };
                let design = SpaulderDesign {
                    crown_coverage: Permille(coverage),
                    fluting: None,
                    ..Default::default()
                };
                let runtime = SpaulderPlates::new(&design, &frame).unwrap().crown;
                let source = SpaulderPlates::new(
                    &design,
                    &PartFrame {
                        detail: ArmorDetail::BakeSource,
                        ..frame
                    },
                )
                .unwrap()
                .crown;
                runtime.normals().unwrap();
                let limit = dimensions.into_iter().fold(0.0_f32, f32::max) * 0.055 + 0.001;
                for point in &source.positions {
                    let point = Vec3::from_array(*point);
                    let distance = runtime
                        .indices
                        .as_chunks::<3>()
                        .0
                        .iter()
                        .map(|face| {
                            triangle_distance(
                                point,
                                face.map(|i| Vec3::from_array(runtime.positions[i as usize])),
                            )
                        })
                        .fold(f32::INFINITY, f32::min);
                    assert!(
                        distance <= limit,
                        "crown loses curved silhouette: lod={lod:?}, coverage={coverage}, dimensions={dimensions:?}, gap={distance}, limit={limit}"
                    );
                }
            }
        }
    }
}

fn triangle_distance(p: Vec3, [a, b, c]: [Vec3; 3]) -> f32 {
    let ab = b - a;
    let ac = c - a;
    let normal = ab.cross(ac);
    let determinant = normal.length_squared();
    if determinant > 0.0 {
        let ap = p - a;
        let u = (ac.length_squared() * ap.dot(ab) - ab.dot(ac) * ap.dot(ac)) / determinant;
        let v = (ab.length_squared() * ap.dot(ac) - ab.dot(ac) * ap.dot(ab)) / determinant;
        if u >= 0.0 && v >= 0.0 && u + v <= 1.0 {
            return ap.dot(normal).abs() / normal.length();
        }
    }
    [(a, b), (b, c), (c, a)]
        .into_iter()
        .map(|(a, b)| {
            let edge = b - a;
            let t = ((p - a).dot(edge) / edge.length_squared()).clamp(0.0, 1.0);
            p.distance(a + edge * t)
        })
        .fold(f32::INFINITY, f32::min)
}
