//! Native skirt carrier behavior. Run after the benchmark build hold is lifted.
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
    let mut fit = vec![0f32; SKIRT_FIT_WORDS as usize];
    fit[..15].copy_from_slice(&[
        0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1., 0.2, 0.45, 0.2,
    ]);
    for row in 0..=SKIRT_AXIAL_SEGMENTS {
        let first = SKIRT_PROFILE_FIRST_WORD as usize + row * SKIRT_SECTION_WORDS as usize;
        fit[first..first + SKIRT_RING_SEGMENTS].fill(0.2);
    }
    let frame = gpu.upload(BufferUpload::from_elements(&fit))?;
    let mut batch = gpu.batch("skirt carrier integration");
    let mut part = record_skirt(gpu, &mut batch, design, &frame)?;
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
