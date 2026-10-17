//! Native skirt carrier and optional construction behavior.
use anyhow::Result;
use fabelgeist_armor::gpu::{
    SKIRT_FIT_WORDS, SKIRT_PROFILE_FIRST_WORD, SKIRT_SECTION_WORDS, record_skirt,
};
use fabelgeist_armor::{
    ArmorGpu, BuiltPart, Permille, SKIRT_AXIAL_SEGMENTS, SKIRT_MESH_AXIAL_SEGMENTS,
    SKIRT_MESH_RING_SEGMENTS, SKIRT_RING_SEGMENTS, SkirtDesign, SkirtProfile,
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
fn tailored_folds_repeat_evenly_and_keep_the_hem_level() -> Result<()> {
    use fabelgeist_armor::{Millimeters, SkirtPleats, SkirtRepeatCount};
    let gpu = ArmorGpu::open()?;
    let design = SkirtDesign {
        fitted_fraction: Permille(20),
        pleats: Some(SkirtPleats {
            count: SkirtRepeatCount(12),
            depth: Millimeters(140),
            waist_count: SkirtRepeatCount(36),
            waist_depth: Millimeters(30),
            gather_retention: Permille(850),
            ..Default::default()
        }),
        ..Default::default()
    };
    let tailored = generate(&gpu, &design)?;
    let repeat_columns = SKIRT_MESH_RING_SEGMENTS / 12;
    for row in [1, 2, SKIRT_MESH_AXIAL_SEGMENTS] {
        let ring = &tailored.positions
            [row * SKIRT_MESH_RING_SEGMENTS..(row + 1) * SKIRT_MESH_RING_SEGMENTS];
        for column in 0..SKIRT_MESH_RING_SEGMENTS {
            let a = ring[column];
            let b = ring[(column + repeat_columns) % SKIRT_MESH_RING_SEGMENTS];
            assert!((a[0].hypot(a[2]) - b[0].hypot(b[2])).abs() < 2e-6);
            assert!((a[1] - b[1]).abs() < 1e-7);
        }
    }
    // Fine gathering changes surface normals instead of mesh connectivity or
    // positions, including the densest gathered half-width quiet-front recipe.
    let mut dense = design.clone();
    let pleats = dense.pleats.as_mut().unwrap();
    pleats.waist_count = SkirtRepeatCount(48);
    pleats.front_panel = Permille(500);
    pleats.rotation = Permille(250);
    let gathered = generate(&gpu, &dense)?;
    dense.pleats.as_mut().unwrap().waist_depth = Millimeters(0);
    let ungathered = generate(&gpu, &dense)?;
    assert_eq!(gathered.positions, ungathered.positions);
    assert_eq!(gathered.indices, ungathered.indices);
    for half_height in [0.005, 0.45] {
        let part = generate_in_span(&gpu, &dense, half_height)?;
        assert!(
            part.positions
                .iter()
                .chain(&part.normals)
                .flatten()
                .all(|v| v.is_finite())
        );
        for triangle in part.indices.as_chunks::<3>().0 {
            let [a, b, c] =
                triangle.map(|index| bevy::math::Vec3::from_array(part.positions[index as usize]));
            assert!((b - a).cross(c - a).length_squared() > 1e-18);
        }
    }
    Ok(())
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

#[test]
fn a_quiet_front_preserves_the_carrier_while_gathers_shape_the_rear() -> Result<()> {
    use fabelgeist_armor::{Millimeters, SkirtPleats, SkirtRepeatCount};
    let gpu = ArmorGpu::open()?;
    let plain = generate(&gpu, &SkirtDesign::default())?;
    for count in [4, 24] {
        let gathered = generate(
            &gpu,
            &SkirtDesign {
                pleats: Some(SkirtPleats {
                    count: SkirtRepeatCount(count),
                    depth: Millimeters(70),
                    waist_depth: Millimeters(30),
                    front_panel: Permille(500),
                    front_depth: Permille(0),
                    rotation: Permille(250),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )?;
        assert_eq!(gathered.indices, plain.indices);
        assert!(gathered.positions.iter().flatten().all(|v| v.is_finite()));
        assert!(gathered.normals.iter().flatten().all(|v| v.is_finite()));
        // Native lattice fixture: row descends, column zero faces +Z. Compare
        // an interior front point and its opposite, away from thickened rims.
        let front = SKIRT_MESH_AXIAL_SEGMENTS / 2 * SKIRT_MESH_RING_SEGMENTS;
        let rear = front + SKIRT_MESH_RING_SEGMENTS / 2;
        for axis in 0..3 {
            assert!((gathered.positions[front][axis] - plain.positions[front][axis]).abs() < 1e-6);
        }
        assert!(gathered.positions[rear][2] < plain.positions[rear][2] - 0.01);
    }
    Ok(())
}
