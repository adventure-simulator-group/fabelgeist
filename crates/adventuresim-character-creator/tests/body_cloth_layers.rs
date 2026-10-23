//! Native shell integration preserves concealed layers and real apertures.
use anyhow::Result;
use fabelgeist_armor::gpu::{
    BODY_CLOTH_FIRST_POINT, BODY_CLOTH_POINT_WORDS, BodyClothLayer, record_body_cloth,
};
use fabelgeist_armor::{ArmorComponentRole, ArmorGpu, BuiltPart};
use fabelgeist_gpu::prelude::BufferUpload;
use std::collections::BTreeMap;

fn generate(gpu: &ArmorGpu, width: f32) -> Result<BuiltPart> {
    // Native numerical fixture: a planar cloth ring with an unobstructed
    // square opening; three materials are stacked along its +Z normal.
    let points = [
        [-0.2, 0.4],
        [0.2, 0.4],
        [0.2, 0.8],
        [-0.2, 0.8],
        [-0.05, 0.55],
        [0.05, 0.55],
        [0.05, 0.65],
        [-0.05, 0.65],
    ];
    let mut fit = vec![0.; BODY_CLOTH_FIRST_POINT as usize];
    fit[3] = 1.;
    fit[7] = 1.;
    fit[11] = 1.;
    let mut layers = Vec::new();
    for (index, role) in [
        ArmorComponentRole::Lining,
        ArmorComponentRole::Interlining,
        ArmorComponentRole::OuterFabric,
    ]
    .into_iter()
    .enumerate()
    {
        let mut indices = Vec::new();
        for column in 0..4u32 {
            let next = (column + 1) % 4;
            indices.extend([column, next, next + 4, column, next + 4, column + 4]);
        }
        for [x, y] in points {
            fit.extend([x * width, y, 0., 0., 0., 1.]);
        }
        layers.push(BodyClothLayer {
            chart: Some(points.iter().map(|[x, y]| [x + 0.5, 1. - y]).collect()),
            point_count: points.len() as u32,
            indices,
            role,
            offset_metres: (index + 2) as f32 * 0.001,
            thickness_metres: 0.001,
        });
    }
    assert_eq!(
        fit.len(),
        BODY_CLOTH_FIRST_POINT as usize
            + points.len() * BODY_CLOTH_POINT_WORDS as usize * layers.len()
    );
    let fit = gpu.upload(BufferUpload::from_elements(&fit))?;
    let mut batch = gpu.batch("layered cloth shell integration");
    let mut part = record_body_cloth(gpu, &mut batch, &layers, &fit)?;
    part.record_shells(gpu, &mut batch)?;
    batch.submit();
    Ok(part.read(gpu)?)
}

#[test]
fn hidden_laminates_survive_thickening_and_refit_without_filling_the_opening() -> Result<()> {
    let gpu = ArmorGpu::open()?;
    let base = generate(&gpu, 1.)?;
    let wider = generate(&gpu, 1.3)?;
    assert_eq!(base.indices, wider.indices);
    assert_eq!(base.positions.len(), wider.positions.len());
    assert_ne!(base.positions, wider.positions);
    // Construction-authored coordinates survive refit and all shell copies.
    assert_eq!(base.texcoords, wider.texcoords);
    assert_eq!(base.components.len(), 3);
    for part in [&base, &wider] {
        assert!(
            part.positions
                .iter()
                .chain(&part.normals)
                .flatten()
                .all(|v| v.is_finite())
        );
        assert!(
            part.texcoords
                .as_ref()
                .unwrap()
                .iter()
                .flatten()
                .all(|v| v.is_finite())
        );
        for (index, component) in part.components.iter().enumerate() {
            let expected = [
                ArmorComponentRole::Lining,
                ArmorComponentRole::Interlining,
                ArmorComponentRole::OuterFabric,
            ][index];
            assert_eq!(component.role, expected);
            assert!(!component.indices.is_empty());
            let points = &part.positions[component.vertices.clone()];
            for (point, uv) in points
                .iter()
                .zip(&part.texcoords.as_ref().unwrap()[component.vertices.clone()])
            {
                let scale = if std::ptr::eq(part, &base) { 1. } else { 1.3 };
                assert!((uv[0] - (point[0] / scale + 0.5)).abs() < 1e-6);
                assert!((uv[1] - (1. - point[1])).abs() < 1e-6);
            }
            let near = points.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
            let far = points
                .iter()
                .map(|p| p[2])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!((near - (index + 1) as f32 * 0.001).abs() < 1e-6);
            assert!((far - (index + 2) as f32 * 0.001).abs() < 1e-6);
            let mut edges = BTreeMap::new();
            for face in part.indices[component.indices.clone()].as_chunks::<3>().0 {
                let positions = face.map(|v| part.positions[v as usize]);
                let centroid = std::array::from_fn::<_, 3, _>(|axis| {
                    positions.iter().map(|p| p[axis]).sum::<f32>() / 3.
                });
                assert!(
                    centroid[0].abs() >= 0.049 || (centroid[1] - 0.6).abs() >= 0.049,
                    "an aperture was filled"
                );
                let vertices = positions.map(|p| p.map(|v| (v * 1_000_000.).round() as i64));
                for [a, b] in [
                    [vertices[0], vertices[1]],
                    [vertices[1], vertices[2]],
                    [vertices[2], vertices[0]],
                ] {
                    let key = if a < b { [a, b] } else { [b, a] };
                    let uses = edges.entry(key).or_insert(Vec::new());
                    uses.push([a, b]);
                }
            }
            for uses in edges.values() {
                assert_eq!(uses.len(), 2, "open or nonmanifold cloth wall");
                assert_eq!(
                    uses[0],
                    [uses[1][1], uses[1][0]],
                    "inconsistent cloth winding"
                );
            }
        }
    }
    Ok(())
}
