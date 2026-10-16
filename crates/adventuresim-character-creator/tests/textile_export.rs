//! Portable cloth preserves preview opacity, ornament and face policy.
#![cfg(feature = "offline-creator")]

use adventuresim_character_creator::export::{
    GlbOutput, RiggedMesh, RiggedShell, export_rigged_glb,
};
use anyhow::{Context, Result};
use fabelgeist_armor::{
    Permille, TextileAppearance, TextileAxis, TextileColor, TextileFill, TextileFinish,
    TextileLayer, TextileRegion,
};
use fabelgeist_rig::RigJointName;

#[test]
fn portable_sheer_cloth_blends_once_and_retains_opaque_pattern_pixels() -> Result<()> {
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
    let maps = adventuresim_character_creator::textile_surface::textures(&fabric)?
        .context("missing sheer maps")?;
    // Native portable-file fixture: metre triangle and one identity rig joint.
    let points = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let normals = [[0., 0., 1.]; 3];
    let indices = [[0; 8]; 3];
    let weights = [[1., 0., 0., 0., 0., 0., 0., 0.]; 3];
    let surface = fabric.material();
    let shell = RiggedShell {
        textile_finish: Some(fabric.finish),
        plate_edges: &[],
        surface: None,
        textures: Some(maps.into()),
        texcoords: Some(&[[0., 0.], [1., 0.], [0., 1.]]),
        hinge: None,
        name: "sheer textile",
        positions: &points,
        normals: &normals,
        faces: &[[0, 1, 2]],
        joint_indices: Some(&indices),
        joint_weights: Some(&weights),
        morph_targets: &[],
        base_color: surface.base_color,
        metallic: surface.metallic,
        roughness: surface.roughness,
    };
    let path = std::env::temp_dir().join(format!(
        "fabelgeist-sheer-export-{}.glb",
        std::process::id()
    ));
    export_rigged_glb(
        GlbOutput::Standalone(&path),
        "textile",
        1,
        4,
        &RiggedMesh {
            joint_proportions: &[],
            morph_targets: &[],
            positions: &points,
            normals: &normals,
            faces: &[],
            export_body: false,
            joint_indices: &indices,
            joint_weights: &weights,
            joint_names: &[RigJointName::ROOT],
            joint_parents: &[-1],
            global_joint_states: &[[0., 0., 0., 0., 0., 0., 1., 1.]],
        },
        &[shell],
        &[],
    )?;
    let file = gltf::Gltf::open(&path)?;
    let material = file
        .document
        .materials()
        .next()
        .context("missing material")?;
    assert_eq!(material.alpha_mode(), gltf::material::AlphaMode::Blend);
    assert!(!material.double_sided());
    assert_eq!(
        material.pbr_metallic_roughness().base_color_factor(),
        [1.; 4]
    );
    let texture = material
        .pbr_metallic_roughness()
        .base_color_texture()
        .context("missing cloth texture")?;
    let gltf::image::Source::View { view, .. } = texture.texture().source().source() else {
        anyhow::bail!("cloth image was not embedded")
    };
    let blob = file.blob.as_ref().context("missing portable image bytes")?;
    let pixels =
        image::load_from_memory(&blob[view.offset()..view.offset() + view.length()])?.to_rgba8();
    assert_eq!(pixels.get_pixel(100, 100).0, [240, 235, 220, 51]);
    assert_eq!(
        pixels.get_pixel(100, pixels.height() - 100).0,
        [190, 150, 55, 255]
    );
    std::fs::remove_file(path)?;
    Ok(())
}
