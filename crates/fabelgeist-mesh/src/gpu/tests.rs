use super::*;
use fabelgeist_gpu::data::vector::{Vec2, Vec3};
use fabelgeist_gpu::prelude::{Sampler, WgpuContext};
#[derive(serde::Deserialize)]
enum ShapeCase {
    PlaneDefault,
    PlaneCustom,
    BoxDefault,
    BoxCustom,
    SphereDefault,
    SphereCustom,
}
#[derive(serde::Deserialize)]
struct ShapeFixture {
    case: ShapeCase,
    vertex_count: u32,
    positions: Vec<u32>,
    normals: Vec<u32>,
    tex_coords: Vec<u32>,
    indices: Vec<u32>,
}
#[tokio::test]
async fn original_shape_upload_bits_counts_and_index_order() {
    let fixtures: Vec<ShapeFixture> =
        serde_json::from_str(include_str!("../../tests/fixtures/shape_contracts.json")).unwrap();
    assert_eq!(fixtures.len(), 6);
    let context = WgpuContext::new().await.unwrap();
    for fixture in fixtures {
        let mesh = match fixture.case {
            ShapeCase::PlaneDefault => PlaneGeometry::default().upload(&context),
            ShapeCase::PlaneCustom => PlaneGeometry::default()
                .with_subdivisions(PlaneSubdivisions::from(Vec2::new(2.8, 1.2)))
                .with_right(Vec3::new(2.0, 0.0, 1.0))
                .with_up(Vec3::new(0.0, 3.0, 0.0))
                .with_normal(Vec3::new(1.0, 1.0, 1.0))
                .upload(&context),
            ShapeCase::BoxDefault => BoxDimensions::default().upload(&context),
            ShapeCase::BoxCustom => BoxDimensions::from(Vec3::new(-2.0, 3.0, 4.0)).upload(&context),
            ShapeCase::SphereDefault => SphereGeometry::default().upload(&context),
            ShapeCase::SphereCustom => SphereGeometry::default()
                .with_radius(SphereRadius::from(-1.25))
                .with_rings(SphereRings::from(2.9))
                .with_sectors(SphereSectors::from(3.5))
                .upload(&context),
        }
        .unwrap();
        assert_eq!(
            mesh.vertex_count,
            crate::DrawVertexCount::from(fixture.vertex_count)
        );
        let positions: Vec<_> = mesh
            .positions
            .read::<f32>(&context)
            .await
            .unwrap()
            .into_iter()
            .map(f32::to_bits)
            .collect();
        let normals: Vec<_> = mesh
            .normals
            .read::<f32>(&context)
            .await
            .unwrap()
            .into_iter()
            .map(f32::to_bits)
            .collect();
        let tex_coords: Vec<_> = mesh
            .tex_coords
            .read::<f32>(&context)
            .await
            .unwrap()
            .into_iter()
            .map(f32::to_bits)
            .collect();
        assert_eq!(positions, fixture.positions);
        assert_eq!(normals, fixture.normals);
        assert_eq!(tex_coords, fixture.tex_coords);
        assert_eq!(
            mesh.indices
                .as_ref()
                .unwrap()
                .read::<u32>(&context)
                .await
                .unwrap(),
            fixture.indices
        );
    }
    let sampler = Sampler::new(&context, None, None, None, None, None);
    assert!(sampler.sampler.is_some());
}
