//! Shared textile treatment works on both a limb and a two-leg skirt carrier.

use anyhow::Result;
use fabelgeist_armor::gpu::{
    SKIRT_FIT_WORDS, SKIRT_PROFILE_FIRST_WORD, SKIRT_SECTION_WORDS, record_puff_and_slash,
    record_skirt,
};
use fabelgeist_armor::{
    ArmorComponentRole, ArmorGpu, BuiltPart, PuffAndSlashDesign, PuffAndSlashPattern,
    SKIRT_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS, SkirtDesign,
};
use fabelgeist_gpu::prelude::BufferUpload;

fn finished(gpu: &ArmorGpu, pattern: &PuffAndSlashPattern) -> Result<[BuiltPart; 2]> {
    // Native GPU fit table: metre coordinates in an upright Y frame.
    let mut words = vec![0.0f32; SKIRT_FIT_WORDS as usize];
    words[..15].copy_from_slice(&[
        0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1., 0.2, 0.45, 0.2,
    ]);
    for row in 0..=SKIRT_AXIAL_SEGMENTS {
        let first = SKIRT_PROFILE_FIRST_WORD as usize + row * SKIRT_SECTION_WORDS as usize;
        words[first..first + SKIRT_RING_SEGMENTS].fill(0.2);
    }
    let fit = gpu.upload(BufferUpload::from_elements(&words))?;
    let mut batch = gpu.batch("shared textile integration");
    let mut skirt = record_skirt(
        gpu,
        &mut batch,
        &SkirtDesign {
            puff_and_slash: Some(pattern.clone()),
            ..Default::default()
        },
        &fit,
        None,
    )?;
    skirt.record_shells(gpu, &mut batch)?;
    batch.submit();
    let skirt = skirt.read(gpu)?;
    let mut batch = gpu.batch("limb textile regression");
    let mut limb = record_puff_and_slash(
        gpu,
        &mut batch,
        &PuffAndSlashDesign {
            pattern: pattern.clone(),
            ..Default::default()
        },
        &fit,
    )?;
    limb.record_shells(gpu, &mut batch)?;
    batch.submit();
    Ok([skirt, limb.read(gpu)?])
}

fn assert_closed(part: &BuiltPart) {
    assert!(
        part.positions
            .iter()
            .chain(&part.normals)
            .flatten()
            .all(|v| v.is_finite())
    );
    assert!(
        part.indices
            .iter()
            .all(|&i| (i as usize) < part.positions.len())
    );
    assert_eq!(
        part.components.iter().map(|c| c.role).collect::<Vec<_>>(),
        [
            ArmorComponentRole::Undercloth,
            ArmorComponentRole::OuterFabric
        ]
    );
    let mut edges = std::collections::BTreeMap::<[u32; 2], Vec<[u32; 2]>>::new();
    for t in part.indices.as_chunks::<3>().0 {
        for [a, b] in [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]] {
            edges.entry([a.min(b), a.max(b)]).or_default().push([a, b]);
        }
    }
    assert!(
        edges
            .values()
            .all(|uses| uses.len() == 2 && uses[0] == [uses[1][1], uses[1][0]])
    );
}

#[test]
fn open_and_closed_treatments_keep_valid_shells_on_both_carriers() -> Result<()> {
    let gpu = ArmorGpu::open()?;
    for (puff_count, slash_count) in [(1, 0), (3, 8), (8, 16)] {
        let pattern = PuffAndSlashPattern {
            puff_count,
            slash_count,
            ..Default::default()
        };
        let [skirt, limb] = finished(&gpu, &pattern)?;
        assert_closed(&skirt);
        assert_closed(&limb);
        // A shared treatment has the same panel connectivity in either chart,
        // although its underlying lining and fitted shape are different.
        let outer = |part: &BuiltPart| {
            let component = &part.components[1];
            part.indices[component.indices.clone()]
                .iter()
                .map(|index| index - component.vertices.start as u32)
                .collect::<Vec<_>>()
        };
        assert_eq!(outer(&skirt), outer(&limb));
        let repeated = finished(&gpu, &pattern)?;
        assert_eq!(skirt.indices, repeated[0].indices);
        assert_eq!(skirt.positions, repeated[0].positions);
    }
    let invalid = SkirtDesign {
        puff_and_slash: Some(PuffAndSlashPattern {
            slash_count: 2,
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(invalid.validate().is_err());
    Ok(())
}

#[test]
fn authored_museum_treatments_use_the_shared_pattern_schema() -> Result<()> {
    fn visit(value: &serde_json::Value) -> Result<usize> {
        match value {
            serde_json::Value::Object(fields) => {
                let mut count = 0;
                if let Some(puff) = fields.get("PuffAndSlash") {
                    serde_json::from_value::<PuffAndSlashDesign>(puff.clone())?.validate()?;
                    count += 1;
                }
                for child in fields.values() {
                    count += visit(child)?;
                }
                Ok(count)
            }
            serde_json::Value::Array(children) => children
                .iter()
                .try_fold(0, |count, child| Ok(count + visit(child)?)),
            _ => Ok(0),
        }
    }
    let museum = serde_json::from_str(include_str!(
        "../../fabelgeist-armor/review/museum/landsknecht/armor.json"
    ))?;
    assert_eq!(visit(&museum)?, 5);
    Ok(())
}
