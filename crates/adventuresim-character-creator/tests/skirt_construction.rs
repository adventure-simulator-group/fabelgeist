//! Joined textile topology survives device evaluation and shell thickening.

use anyhow::Result;
use fabelgeist_armor::gpu::{
    ClothRingVertex, SKIRT_ATTACHMENT_POINT_WORDS, SKIRT_FIT_WORDS, SKIRT_PROFILE_FIRST_WORD,
    SKIRT_SECTION_WORDS, SkirtAttachment, SkirtSeamLayout, record_skirt,
};
use fabelgeist_armor::{
    ArmorGpu, BuiltPart, SKIRT_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS, SkirtDesign,
};
use fabelgeist_gpu::prelude::BufferUpload;
use std::collections::{BTreeMap, BTreeSet};

fn joined_fixture(
    gpu: &ArmorGpu,
    body_width: f32,
    front_drop: f32,
    hanging_intervals: usize,
    seam_layout: SkirtSeamLayout,
    finishing: bool,
) -> Result<BuiltPart> {
    // Native fitted-frame fixture: metres, Y up, square cloned torso above a
    // circular skirt. The waist has fewer vertices than the skirt's ring.
    let mut fit = vec![
        0f32;
        SKIRT_FIT_WORDS as usize
            + (8 + SKIRT_RING_SEGMENTS) * SKIRT_ATTACHMENT_POINT_WORDS as usize
    ];
    fit[..15].copy_from_slice(&[
        0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1., 0.2, 0.45, 0.2,
    ]);
    for row in 0..=SKIRT_AXIAL_SEGMENTS {
        let first = SKIRT_PROFILE_FIRST_WORD as usize + row * SKIRT_SECTION_WORDS as usize;
        fit[first..first + SKIRT_RING_SEGMENTS].fill(0.2);
    }
    let mut indices = Vec::new();
    let mut waist = Vec::new();
    for column in 0..4 {
        let next = (column + 1) % 4;
        indices.extend([column, next, next + 4, column, next + 4, column + 4]);
        waist.push(ClothRingVertex {
            vertex: column,
            angle: std::f32::consts::TAU * column as f32 / 4.0,
        });
    }
    for point in 0..8 + SKIRT_RING_SEGMENTS {
        let (angle, mut height) = if point < 8 {
            (
                std::f32::consts::TAU * (point % 4) as f32 / 4.0,
                if point < 4 { 0.45 } else { 0.75 },
            )
        } else {
            (
                std::f32::consts::TAU * (point - 8) as f32 / SKIRT_RING_SEGMENTS as f32,
                0.45,
            )
        };
        if !(4..8).contains(&point) {
            height -= front_drop * angle.cos().max(0.0);
        }
        let first = SKIRT_FIT_WORDS as usize + point * SKIRT_ATTACHMENT_POINT_WORDS as usize;
        fit[first..first + 6].copy_from_slice(&[
            angle.sin() * 0.16 * body_width,
            height,
            angle.cos() * 0.16,
            angle.sin(),
            0.0,
            angle.cos(),
        ]);
    }
    let frame = gpu.upload(BufferUpload::from_elements(&fit))?;
    let layers = if finishing {
        vec![fabelgeist_armor::gpu::AttachedClothLayer {
            first_point: 0,
            layer: fabelgeist_armor::gpu::BodyClothLayer {
                point_count: 8,
                indices: indices.clone(),
                chart: None,
                role: fabelgeist_armor::ArmorComponentRole::EdgeBinding,
                offset_metres: 0.003,
                thickness_metres: 0.001,
            },
        }]
    } else {
        Vec::new()
    };
    let attachment = SkirtAttachment {
        dressing_opening: None,
        seam_layout,
        ring_segments: SkirtDesign::default().mesh_ring_segments(),
        hanging_rows: fabelgeist_armor::gpu::SkirtHangingRows::uniform(1, hanging_intervals)?,
        first_row: 1,
        point_count: 8,
        indices,
        waist,
        layers,
    };
    let mut design = SkirtDesign {
        bodice: Some(fabelgeist_armor::DressBodice {
            transition: fabelgeist_armor::Permille(20),
            ..Default::default()
        }),
        ..Default::default()
    };
    if seam_layout == SkirtSeamLayout::StraightFall {
        design.profile = fabelgeist_armor::SkirtProfile::Straight;
    }
    let mut batch = gpu.batch("continuous dress integration");
    let mut part = record_skirt(gpu, &mut batch, &design, &frame, Some(&attachment))?;
    part.record_shells(gpu, &mut batch)?;
    batch.submit();
    Ok(part.read(gpu)?)
}

fn assert_closed_connected(part: &BuiltPart, component: &fabelgeist_armor::ArmorComponent) {
    let first = component.vertices.start;
    let count = component.vertices.len();
    let mut edges = BTreeMap::<[u32; 2], Vec<[u32; 2]>>::new();
    let mut adjacent = vec![Vec::new(); count];
    for triangle in part.indices[component.indices.clone()].as_chunks::<3>().0 {
        for [a, b] in [
            [triangle[0], triangle[1]],
            [triangle[1], triangle[2]],
            [triangle[2], triangle[0]],
        ] {
            edges.entry([a.min(b), a.max(b)]).or_default().push([a, b]);
            adjacent[a as usize - first].push(b);
            adjacent[b as usize - first].push(a);
        }
    }
    for uses in edges.values() {
        assert_eq!(uses.len(), 2, "a seam edge is open or nonmanifold");
        assert_eq!(uses[0], [uses[1][1], uses[1][0]], "a seam reverses winding");
    }
    let mut reached = BTreeSet::new();
    let mut pending = vec![first as u32];
    while let Some(vertex) = pending.pop() {
        if reached.insert(vertex) {
            pending.extend(&adjacent[vertex as usize - first]);
        }
    }
    assert_eq!(reached.len(), count, "body and skirt are separate shells");
}

#[test]
fn cloned_body_and_skirt_are_one_closed_surface_with_fixed_refit_topology() -> Result<()> {
    let gpu = ArmorGpu::open()?;
    for seam_layout in [SkirtSeamLayout::Transition, SkirtSeamLayout::StraightFall] {
        for hanging_intervals in [2, fabelgeist_armor::SKIRT_MESH_AXIAL_SEGMENTS] {
            let reference = joined_fixture(&gpu, 1., 0., hanging_intervals, seam_layout, false)?;
            for (width, drop) in [(1.2, 0.), (1., 0.2), (1.2, 0.2)] {
                let part =
                    joined_fixture(&gpu, width, drop, hanging_intervals, seam_layout, false)?;
                assert_eq!(reference.indices, part.indices);
                assert_eq!(reference.positions.len(), part.positions.len());
                assert_ne!(reference.positions, part.positions);
                assert!(part.positions.iter().flatten().all(|v| v.is_finite()));
                assert!(part.normals.iter().flatten().all(|v| v.is_finite()));
                for component in &part.components {
                    assert_closed_connected(&part, component);
                }
                let columns = match seam_layout {
                    SkirtSeamLayout::Transition => SkirtDesign::default().mesh_ring_segments(),
                    SkirtSeamLayout::StraightFall => 4,
                };
                let hem = part.positions[..(hanging_intervals + 1) * columns]
                    .iter()
                    .map(|point| point[1])
                    .fold(f32::INFINITY, f32::min);
                assert!((hem + 0.45).abs() < 1e-6);
            }
        }
    }

    Ok(())
}

#[test]
fn folded_finishing_sheet_tracks_the_attachment_as_a_separate_closed_component() -> Result<()> {
    let gpu = ArmorGpu::open()?;
    let base = joined_fixture(&gpu, 1., 0., 2, SkirtSeamLayout::StraightFall, true)?;
    let refit = joined_fixture(&gpu, 1.2, 0.2, 2, SkirtSeamLayout::StraightFall, true)?;
    assert_eq!(base.indices, refit.indices);
    assert_eq!(base.positions.len(), refit.positions.len());
    assert_ne!(base.positions, refit.positions);
    assert_eq!(refit.components.len(), 2);
    assert_eq!(
        refit.components[1].role,
        fabelgeist_armor::ArmorComponentRole::EdgeBinding
    );
    assert!(refit.positions.iter().flatten().all(|v| v.is_finite()));
    assert!(refit.normals.iter().flatten().all(|v| v.is_finite()));
    for component in &refit.components {
        assert_closed_connected(&refit, component);
    }
    Ok(())
}
