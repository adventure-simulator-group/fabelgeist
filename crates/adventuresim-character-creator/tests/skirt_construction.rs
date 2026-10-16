//! Joined textile topology survives device evaluation and shell thickening.

use anyhow::Result;
use fabelgeist_armor::gpu::{
    SKIRT_ATTACHMENT_POINT_WORDS, SKIRT_FIT_WORDS, SKIRT_PROFILE_FIRST_WORD, SKIRT_SECTION_WORDS,
    SkirtAttachment, SkirtWaistVertex, record_skirt,
};
use fabelgeist_armor::{
    ArmorGpu, BuiltPart, SKIRT_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS, SkirtDesign,
};
use fabelgeist_gpu::prelude::BufferUpload;
use std::collections::{BTreeMap, BTreeSet};

fn joined_fixture(gpu: &ArmorGpu, body_width: f32, front_drop: f32) -> Result<BuiltPart> {
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
        waist.push(SkirtWaistVertex {
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
    let attachment = SkirtAttachment {
        first_row: 1,
        point_count: 8,
        indices,
        waist,
    };
    let design = SkirtDesign {
        bodice: Some(fabelgeist_armor::DressBodice {
            transition: fabelgeist_armor::Permille(20),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut batch = gpu.batch("continuous dress integration");
    let mut part = record_skirt(gpu, &mut batch, &design, &frame, Some(&attachment))?;
    part.record_shells(gpu, &mut batch)?;
    batch.submit();
    Ok(part.read(gpu)?)
}

fn assert_closed_connected(part: &BuiltPart) {
    let mut edges = BTreeMap::<[u32; 2], Vec<[u32; 2]>>::new();
    let mut adjacent = vec![Vec::new(); part.positions.len()];
    for triangle in part.indices.as_chunks::<3>().0 {
        for [a, b] in [
            [triangle[0], triangle[1]],
            [triangle[1], triangle[2]],
            [triangle[2], triangle[0]],
        ] {
            edges.entry([a.min(b), a.max(b)]).or_default().push([a, b]);
            adjacent[a as usize].push(b);
            adjacent[b as usize].push(a);
        }
    }
    for uses in edges.values() {
        assert_eq!(uses.len(), 2, "a seam edge is open or nonmanifold");
        assert_eq!(uses[0], [uses[1][1], uses[1][0]], "a seam reverses winding");
    }
    let mut reached = BTreeSet::new();
    let mut pending = vec![0u32];
    while let Some(vertex) = pending.pop() {
        if reached.insert(vertex) {
            pending.extend(&adjacent[vertex as usize]);
        }
    }
    assert_eq!(
        reached.len(),
        part.positions.len(),
        "body and skirt are separate shells"
    );
}

#[test]
fn cloned_body_and_skirt_are_one_closed_surface_with_fixed_refit_topology() -> Result<()> {
    let gpu = ArmorGpu::open()?;
    let reference = joined_fixture(&gpu, 1.0, 0.0)?;
    for (width, drop) in [(1.2, 0.0), (1.0, 0.2), (1.2, 0.2)] {
        let part = joined_fixture(&gpu, width, drop)?;
        assert_eq!(reference.indices, part.indices);
        assert_eq!(reference.positions.len(), part.positions.len());
        assert_ne!(reference.positions, part.positions);
        assert!(part.positions.iter().flatten().all(|v| v.is_finite()));
        assert!(part.normals.iter().flatten().all(|v| v.is_finite()));
        assert_closed_connected(&part);
        let hem = SKIRT_AXIAL_SEGMENTS - 1;
        let ring = &part.positions[hem * SKIRT_RING_SEGMENTS..(hem + 1) * SKIRT_RING_SEGMENTS];
        assert!(ring.iter().all(|p| (p[1] + 0.45).abs() < 1e-6));
    }
    Ok(())
}
