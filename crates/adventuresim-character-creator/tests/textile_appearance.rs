//! Fabric charts, shading seams and color layers survive native generation.
use anyhow::Result;
use fabelgeist_armor::gpu::{
    SKIRT_FIT_WORDS, SKIRT_PROFILE_FIRST_WORD, SKIRT_SECTION_WORDS, record_puff_and_slash,
    record_skirt,
};
use fabelgeist_armor::{
    ArmorGpu, ArmorMorph, BuiltPart, GeneratedArmor, Permille, PuffAndSlashDesign,
    SKIRT_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS, SkirtDesign, TextileAppearance, TextileAxis,
    TextileColor, TextileFill, TextileFinish, TextileLayer, TextileLayers, TextileRegion,
    TextileSpan,
};
use fabelgeist_gpu::prelude::BufferUpload;

fn generated(gpu: &ArmorGpu, placed: bool, limb: bool) -> Result<BuiltPart> {
    // Native metre frame fixture: translated and rotated 90 degrees about Z.
    let frame = if placed {
        [
            0.7, 1.2, -0.4, 0., 1., 0., -1., 0., 0., 0., 0., 1., 0.2, 0.45, 0.2,
        ]
    } else {
        [
            0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1., 0.2, 0.45, 0.2,
        ]
    };
    let mut words = vec![0f32; SKIRT_FIT_WORDS as usize];
    words[..15].copy_from_slice(&frame);
    for row in 0..=SKIRT_AXIAL_SEGMENTS {
        let first = SKIRT_PROFILE_FIRST_WORD as usize + row * SKIRT_SECTION_WORDS as usize;
        words[first..first + SKIRT_RING_SEGMENTS].fill(0.2);
    }
    let fit = gpu.upload(BufferUpload::from_elements(&words))?;
    let mut batch = gpu.batch("fabric frame integration");
    let mut part = if limb {
        record_puff_and_slash(gpu, &mut batch, &PuffAndSlashDesign::default(), &fit)?
    } else {
        record_skirt(gpu, &mut batch, &SkirtDesign::default(), &fit, None)?
    };
    part.record_shells(gpu, &mut batch)?;
    batch.submit();
    Ok(part.read(gpu)?)
}

#[test]
fn curved_ornament_parameters_change_the_shared_baked_surface() -> Result<()> {
    use fabelgeist_armor::{TextileMotif, TextileRepeatCount};
    let mut fabric = TextileAppearance::solid(TextileColor([35, 12, 18]), TextileFinish::Silk);
    fabric.layers.0.push(TextileLayer {
        region: TextileRegion::Whole,
        color: TextileColor([145, 110, 55]),
        fill: TextileFill::Motifs {
            motif: TextileMotif::Scrollwork {
                branches: TextileRepeatCount(2),
                curl: Permille(250),
            },
            across: TextileRepeatCount(8),
            along: TextileRepeatCount(4),
            scale: Permille(1000),
            rotation: Permille(125),
            stagger: Permille(500),
        },
    });
    let before = adventuresim_character_creator::textile_surface::textures(&fabric)?.unwrap();
    if let TextileFill::Motifs { motif, .. } = &mut fabric.layers.0[0].fill {
        *motif = TextileMotif::Scrollwork {
            branches: TextileRepeatCount(2),
            curl: Permille(1000),
        };
    }
    let after = adventuresim_character_creator::textile_surface::textures(&fabric)?.unwrap();
    let before = image::load_from_memory(&before.base_color_png)?.to_rgba8();
    let after = image::load_from_memory(&after.base_color_png)?.to_rgba8();
    assert_eq!(before.dimensions(), after.dimensions());
    assert_ne!(before.as_raw(), after.as_raw());
    assert!(before.pixels().any(|pixel| pixel.0[..3] == fabric.color.0));
    assert!(
        before
            .pixels()
            .any(|pixel| pixel.0[..3] == fabric.layers.0[0].color.0)
    );
    Ok(())
}

#[test]
fn placement_does_not_change_skirt_or_sleeve_fabric_coordinates() -> Result<()> {
    let gpu = ArmorGpu::open()?;
    for limb in [false, true] {
        let base = generated(&gpu, false, limb)?;
        let placed = generated(&gpu, true, limb)?;
        assert_eq!(base.indices, placed.indices);
        let base_uv = base.texcoords.as_ref().unwrap();
        let placed_uv = placed.texcoords.as_ref().unwrap();
        assert_eq!(base_uv.len(), base.positions.len());
        assert_eq!(base_uv.len(), placed_uv.len());
        for (a, b) in base_uv.iter().zip(placed_uv) {
            assert!(
                a.iter()
                    .chain(b)
                    .all(|v| v.is_finite() && (0.0..=1.00001).contains(v))
            );
            let around = (a[0] - b[0]).abs();
            assert!(around.min((1.0 - around).abs()) < 0.00001);
            assert!((a[1] - b[1]).abs() < 0.00001);
        }
        assert!(base_uv.iter().any(|uv| uv[1] < 0.00001));
        assert!(base_uv.iter().any(|uv| uv[1] > 0.99999));
    }
    Ok(())
}

fn skinned(mut part: BuiltPart) -> GeneratedArmor {
    let count = part.positions.len();
    for component in &mut part.components {
        component.textile = Some(TextileAppearance::solid(
            TextileColor([70, 15, 25]),
            TextileFinish::Velvet,
        ));
    }
    GeneratedArmor {
        components: part.components,
        design_hash: [0; 32],
        surface_domain: "fixture_body".into(),
        morphs: vec![ArmorMorph {
            name: "fixture_shape".into(),
            direct_positions: part
                .positions
                .iter()
                .map(|p| [p[0] + 0.1, p[1], p[2]])
                .collect(),
            position_deltas: vec![[0.1, 0., 0.]; count],
            normal_deltas: vec![[0., 0.02, 0.]; count],
        }],
        positions: part.positions,
        normals: part.normals,
        texcoords: part.texcoords.unwrap(),
        joint_indices: vec![[2, 5, 0, 0, 0, 0, 0, 0]; count],
        joint_weights: vec![[0.7, 0.3, 0., 0., 0., 0., 0., 0.]; count],
        indices: part.indices,
        faces: part.faces,
        trim: None,
        grids: part.grids,
    }
}

#[test]
fn rear_shading_seam_preserves_triangles_skin_and_morph_endpoints() -> Result<()> {
    let gpu = ArmorGpu::open()?;
    let source = skinned(generated(&gpu, false, false)?);
    let mut split = source.clone();
    split.split_textile_seams()?;
    assert!(split.positions.len() > source.positions.len());
    assert_eq!(split.indices.len(), source.indices.len());
    for (&before, &after) in source.indices.iter().zip(&split.indices) {
        let a = before as usize;
        let b = after as usize;
        assert_eq!(source.positions[a], split.positions[b]);
        assert_eq!(source.normals[a], split.normals[b]);
        assert_eq!(source.joint_indices[a], split.joint_indices[b]);
        assert_eq!(source.joint_weights[a], split.joint_weights[b]);
        assert_eq!(
            source.morphs[0].position_deltas[a],
            split.morphs[0].position_deltas[b]
        );
        assert_eq!(
            source.morphs[0].normal_deltas[a],
            split.morphs[0].normal_deltas[b]
        );
        assert_eq!(
            source.morphs[0].direct_positions[a],
            split.morphs[0].direct_positions[b]
        );
        assert_eq!(source.texcoords[a][1], split.texcoords[b][1]);
        let shift = split.texcoords[b][0] - source.texcoords[a][0];
        assert!(shift == 0.0 || (shift - 1.0).abs() < 0.000001);
    }
    for triangle in split.indices.as_chunks::<3>().0 {
        let u = triangle
            .iter()
            .map(|&i| split.texcoords[i as usize][0])
            .collect::<Vec<_>>();
        assert!(
            u.iter().copied().fold(f32::NEG_INFINITY, f32::max)
                - u.iter().copied().fold(f32::INFINITY, f32::min)
                < 0.5
        );
    }
    let mut invalid = source.clone();
    invalid.normals.pop();
    assert!(invalid.split_textile_seams().is_err());
    Ok(())
}

#[test]
fn bounded_layers_composite_into_the_same_preview_and_export_pixels() -> Result<()> {
    let fabric = TextileAppearance {
        color: TextileColor([20, 30, 40]),
        finish: TextileFinish::Linen,
        layers: TextileLayers(vec![
            TextileLayer {
                region: TextileRegion::Band {
                    axis: TextileAxis::Along,
                    start: Permille(750),
                    end: Permille(1000),
                },
                fill: TextileFill::Solid,
                color: TextileColor([90, 80, 70]),
            },
            TextileLayer {
                region: TextileRegion::Panel {
                    across: TextileSpan {
                        start: Permille(450),
                        end: Permille(550),
                    },
                    along: TextileSpan {
                        start: Permille(100),
                        end: Permille(950),
                    },
                },
                fill: TextileFill::Solid,
                color: TextileColor([150, 120, 50]),
            },
        ]),
    };
    let pixels = fabric.pixels()?;
    let maps = adventuresim_character_creator::textile_surface::textures(&fabric)?.unwrap();
    let png = image::load_from_memory(&maps.base_color_png)?.to_rgba8();
    assert_eq!(png.as_raw(), &pixels.color);
    let sample = |u: u32, v: u32| {
        png.get_pixel(png.width() * u / 100, png.height() * v / 100)
            .0
    };
    assert_eq!(sample(20, 20), [20, 30, 40, 255]);
    assert_eq!(sample(20, 90), [90, 80, 70, 255]);
    assert_eq!(sample(50, 90), [150, 120, 50, 255]);
    assert_eq!(sample(50, 98), [90, 80, 70, 255]);
    let mut invalid = fabric;
    invalid.layers.0[0].region = TextileRegion::Band {
        axis: TextileAxis::Across,
        start: Permille(500),
        end: Permille(500),
    };
    assert!(invalid.pixels().is_err());
    Ok(())
}
