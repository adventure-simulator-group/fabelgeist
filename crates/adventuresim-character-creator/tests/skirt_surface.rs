//! Native skirt carrier and optional construction behavior.
use anyhow::Result;
use fabelgeist_armor::gpu::{
    SKIRT_FIT_WORDS, SKIRT_PROFILE_FIRST_WORD, SKIRT_SECTION_WORDS, record_skirt,
};
use fabelgeist_armor::{
    ArmorGpu, BuiltPart, Permille, SKIRT_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS, SkirtDesign,
    SkirtProfile,
};
use fabelgeist_gpu::prelude::BufferUpload;

fn generate(gpu: &ArmorGpu, design: &SkirtDesign) -> Result<BuiltPart> {
    generate_in_span(gpu, design, 0.45)
}

/// Native fitted frame fixture; height is the vertical half span in metres.
fn generate_in_span(gpu: &ArmorGpu, design: &SkirtDesign, half_height: f32) -> Result<BuiltPart> {
    let mut fit = vec![0f32; SKIRT_FIT_WORDS as usize];
    fit[..15].copy_from_slice(&[
        0.,
        0.,
        0.,
        1.,
        0.,
        0.,
        0.,
        1.,
        0.,
        0.,
        0.,
        1.,
        0.2,
        half_height,
        0.2,
    ]);
    for row in 0..=SKIRT_AXIAL_SEGMENTS {
        let first = SKIRT_PROFILE_FIRST_WORD as usize + row * SKIRT_SECTION_WORDS as usize;
        fit[first..first + SKIRT_RING_SEGMENTS].fill(0.2);
    }
    let frame = gpu.upload(BufferUpload::from_elements(&fit))?;
    let mut batch = gpu.batch("skirt carrier integration");
    let mut part = record_skirt(gpu, &mut batch, design, &frame, None)?;
    part.record_shells(gpu, &mut batch)?;
    batch.submit();
    Ok(part.read(gpu)?)
}

fn hem_radius(part: &BuiltPart) -> f32 {
    part.positions
        .iter()
        .filter(|p| p[1] < -0.44)
        .map(|p| p[0].hypot(p[2]))
        .fold(0.0, f32::max)
}

#[test]
fn straight_and_bell_carriers_keep_correspondence_and_distinct_silhouettes() -> Result<()> {
    let gpu = ArmorGpu::open()?;
    let straight = generate(
        &gpu,
        &SkirtDesign {
            profile: SkirtProfile::Straight,
            ..Default::default()
        },
    )?;
    let bell = generate(
        &gpu,
        &SkirtDesign {
            profile: SkirtProfile::Bell {
                width: Permille(2200),
                depth: Permille(2200),
                roundness: Permille(1000),
            },
            ..Default::default()
        },
    )?;
    assert_eq!(straight.indices, bell.indices);
    assert_eq!(straight.positions.len(), bell.positions.len());
    assert!(hem_radius(&bell) > hem_radius(&straight) * 1.9);
    for part in [&straight, &bell] {
        assert!(part.positions.iter().flatten().all(|v| v.is_finite()));
        assert!(part.normals.iter().flatten().all(|v| v.is_finite()));
        assert!(
            part.indices
                .iter()
                .all(|&i| (i as usize) < part.positions.len())
        );
        let outward = part
            .positions
            .iter()
            .zip(&part.normals)
            .filter(|(p, _)| p[1].abs() < 0.3)
            .any(|(p, n)| p[0] * n[0] + p[2] * n[2] > 0.1);
        assert!(
            outward,
            "carrier outer surface has no outward radial normals"
        );
    }
    Ok(())
}

#[test]
fn folds_and_supports_are_independent_closed_constructions() -> Result<()> {
    use fabelgeist_armor::{ArmorComponentRole, PleatStyle, SkirtHoops, SkirtPleats, SkirtSupport};
    let gpu = ArmorGpu::open()?;
    let plain = generate(&gpu, &SkirtDesign::default())?;
    for style in [PleatStyle::Rounded, PleatStyle::Accordion] {
        let folded = generate(
            &gpu,
            &SkirtDesign {
                pleats: Some(SkirtPleats {
                    style,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )?;
        assert_eq!(folded.indices, plain.indices);
        assert_ne!(folded.positions, plain.positions);
    }
    let design = SkirtDesign {
        pleats: Some(SkirtPleats::default()),
        support: Some(SkirtSupport {
            hoops: Some(SkirtHoops::default()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let supported = generate(&gpu, &design)?;
    assert_eq!(
        supported
            .components
            .iter()
            .map(|c| c.role)
            .collect::<Vec<_>>(),
        [
            ArmorComponentRole::OuterFabric,
            ArmorComponentRole::Boning,
            ArmorComponentRole::Hoops
        ]
    );
    assert!(supported.positions.iter().flatten().all(|v| v.is_finite()));
    assert!(supported.normals.iter().flatten().all(|v| v.is_finite()));
    let mut edges = std::collections::BTreeMap::<[u32; 2], Vec<[u32; 2]>>::new();
    for t in supported.indices.as_chunks::<3>().0 {
        for [a, b] in [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]] {
            edges.entry([a.min(b), a.max(b)]).or_default().push([a, b]);
        }
    }
    assert!(
        edges
            .values()
            .all(|uses| uses.len() == 2 && uses[0] == [uses[1][1], uses[1][0]])
    );
    let invalid = SkirtDesign {
        support: Some(SkirtSupport {
            stays: None,
            hoops: None,
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(invalid.validate().is_err());
    let tiny = generate_in_span(
        &gpu,
        &SkirtDesign {
            support: Some(SkirtSupport {
                stays: None,
                hoops: Some(SkirtHoops {
                    width: fabelgeist_armor::Millimeters(20),
                    start: Permille(100),
                    end: Permille(980),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            profile: SkirtProfile::Straight,
            ..Default::default()
        },
        0.005,
    )?;
    let hoops = tiny
        .components
        .iter()
        .find(|c| c.role == ArmorComponentRole::Hoops)
        .ok_or_else(|| anyhow::anyhow!("missing hoops"))?;
    assert!(
        tiny.positions[hoops.vertices.clone()]
            .iter()
            .all(|p| p[1].abs() <= 0.00501)
    );
    Ok(())
}
