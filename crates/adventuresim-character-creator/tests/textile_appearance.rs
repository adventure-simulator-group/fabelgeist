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
            space: fabelgeist_armor::TextilePatternSpace::Region,
            motif: TextileMotif::Scrollwork {
                branches: TextileRepeatCount(2),
                curl: Permille(250),
            },
            across: TextileRepeatCount(8),
            along: TextileRepeatCount(4),
            scale: Permille(1000),
            rotation: Permille(125),
            stagger: Permille(500),
            offset: Default::default(),
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
    if let TextileFill::Motifs { offset, .. } = &mut fabric.layers.0[0].fill {
        offset.across = Permille(500);
        offset.along = Permille(500);
    }
    let shifted = adventuresim_character_creator::textile_surface::textures(&fabric)?.unwrap();
    let shifted = image::load_from_memory(&shifted.base_color_png)?.to_rgba8();
    // Half a repeat at 8 x 4 repeats shifts this frozen image by 1/16 x 1/8.
    for row in (0..after.height()).step_by(17) {
        for column in (0..after.width()).step_by(17) {
            assert_eq!(
                shifted.get_pixel(column, row),
                after.get_pixel(
                    (column + after.width() / 16) % after.width(),
                    (row + after.height() / 8) % after.height()
                )
            );
        }
    }
    Ok(())
}

#[test]
fn continuous_fabric_keeps_its_repeat_scale_when_clipped_into_bands() -> Result<()> {
    use fabelgeist_armor::{TextileMotif, TextilePatternSpace, TextileRepeatCount};
    let mut whole = TextileAppearance::solid(TextileColor([35, 12, 18]), TextileFinish::Velvet);
    whole.layers.0.push(TextileLayer {
        region: TextileRegion::Whole,
        color: TextileColor([145, 110, 55]),
        fill: TextileFill::Motifs {
            space: TextilePatternSpace::Garment,
            motif: TextileMotif::Damask {
                leaf_pairs: TextileRepeatCount(2),
                spread: Permille(950),
            },
            across: TextileRepeatCount(8),
            along: TextileRepeatCount(3),
            scale: Permille(1000),
            rotation: Permille(0),
            stagger: Permille(500),
            offset: Default::default(),
        },
    });
    let mut clipped = whole.clone();
    clipped.layers.0.clear();
    for (start, end) in [(200, 400), (700, 900)] {
        let mut band = whole.layers.0[0].clone();
        band.region = TextileRegion::Band {
            axis: TextileAxis::Along,
            start: Permille(start),
            end: Permille(end),
        };
        clipped.layers.0.push(band);
    }
    let render = |fabric: &TextileAppearance| -> Result<image::RgbaImage> {
        let maps = adventuresim_character_creator::textile_surface::textures(fabric)?.unwrap();
        Ok(image::load_from_memory(&maps.base_color_png)?.to_rgba8())
    };
    let mut denser = whole.clone();
    if let TextileFill::Motifs {
        motif: TextileMotif::Damask { leaf_pairs, .. },
        ..
    } = &mut denser.layers.0[0].fill
    {
        *leaf_pairs = TextileRepeatCount(5);
    }
    let denser = render(&denser)?;
    let whole = render(&whole)?;
    assert_ne!(whole.as_raw(), denser.as_raw());
    let clipped = render(&clipped)?;
    let mut colored = 0;
    for row in (0..whole.height()).step_by(13) {
        let along = (row as f32 + 0.5) / whole.height() as f32;
        let in_band = (0.2..0.4).contains(&along) || (0.7..0.9).contains(&along);
        for column in (0..whole.width()).step_by(11) {
            let actual = clipped.get_pixel(column, row);
            if in_band {
                assert_eq!(actual, whole.get_pixel(column, row));
                colored += usize::from(actual.0[..3] == [145, 110, 55]);
            } else {
                assert_eq!(actual.0, [35, 12, 18, 255]);
            }
        }
    }
    assert!(colored > 100, "clipped bands must retain visible ornament");
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
        opacity: Permille(1000),
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

#[test]
fn sheer_cloth_keeps_opaque_ornament_in_shared_preview_and_portable_pixels() -> Result<()> {
    use bevy::prelude::{Alpha, AlphaMode, Assets, Image};
    let mut fabric = TextileAppearance::solid(TextileColor([240, 235, 220]), TextileFinish::Linen);
    fabric.opacity = Permille(200);
    fabric.layers.0.push(TextileLayer {
        region: TextileRegion::Band {
            axis: TextileAxis::Along,
            start: Permille(750),
            end: Permille::ONE,
        },
        fill: TextileFill::Solid,
        color: TextileColor([190, 150, 55]),
    });
    let maps = adventuresim_character_creator::textile_surface::textures(&fabric)?.unwrap();
    let pixels = image::load_from_memory(&maps.base_color_png)?.to_rgba8();
    assert_eq!(pixels.get_pixel(100, 100).0, [240, 235, 220, 51]);
    assert_eq!(
        pixels.get_pixel(100, pixels.height() - 100).0,
        [190, 150, 55, 255]
    );
    let material = adventuresim_character_creator::textile_surface::material(
        &fabric,
        &mut Assets::<Image>::default(),
    )?;
    assert_eq!(material.alpha_mode, AlphaMode::Blend);
    assert_eq!(material.base_color.alpha(), 1.);
    assert!(material.base_color_texture.is_some());
    assert!(material.cull_mode.is_some());
    #[cfg(feature = "offline-creator")]
    {
        let portable: adventuresim_character_creator::export::SurfaceTextures = maps.into();
        assert!(matches!(
            portable.alpha,
            adventuresim_character_creator::export::SurfaceAlpha::Blend
        ));
        assert!(matches!(
            portable.sides,
            adventuresim_character_creator::export::SurfaceSides::Front
        ));
    }
    fabric.layers.0.clear();
    assert!(adventuresim_character_creator::textile_surface::textures(&fabric)?.is_some());
    let material = adventuresim_character_creator::textile_surface::material(
        &fabric,
        &mut Assets::<Image>::default(),
    )?;
    assert!((material.base_color.alpha() - 0.2).abs() < 1e-6);
    fabric.opacity = Permille(1001);
    assert!(fabric.validate().is_err());
    Ok(())
}
