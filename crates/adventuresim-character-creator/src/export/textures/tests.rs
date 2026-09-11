use super::*;

const MAPS: SurfaceTextures = SurfaceTextures {
    base_color_png: include_bytes!(
        "../../../../../assets_src/equipment/materials/mail-base-color.png"
    ),
    normal_png: include_bytes!("../../../../../assets_src/equipment/materials/mail-normal.png"),
    occlusion_png: Some(include_bytes!(
        "../../../../../assets_src/equipment/materials/mail-occlusion.png"
    )),
    cutout: true,
};

fn export_equipment(path: &Path) -> Result<()> {
    let positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let normals = [[0.0, 0.0, 1.0]; 3];
    let joints = [[0; 8]; 3];
    let weights = [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 3];
    let shell = RiggedShell {
        name: "mail",
        hinge: None,
        positions: &positions,
        normals: &normals,
        faces: &[[0, 1, 2]],
        joint_indices: Some(&joints),
        joint_weights: Some(&weights),
        morph_targets: &[],
        base_color: [1.0; 4],
        metallic: 1.0,
        roughness: 0.3,
        texcoords: Some(&[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
        textures: Some(MAPS),
    };
    export_rigged_glb(
        GlbOutput::SharedTextures(path),
        "mail",
        1,
        1,
        &RiggedMesh {
            positions: &positions,
            normals: &normals,
            faces: &[],
            export_body: false,
            joint_indices: &joints,
            joint_weights: &weights,
            joint_names: &["root".to_owned()],
            joint_parents: &[-1],
            global_joint_states: &[[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]],
            joint_proportions: &[],
            morph_targets: &[],
        },
        &[shell],
        &[],
    )
}

#[test]
fn separate_equipment_glbs_share_exact_surface_files_and_material_roles() {
    let directory = std::env::temp_dir().join(format!("shared-mail-export-{}", std::process::id()));
    let paths = [
        directory.join("voiders.glb"),
        directory.join("standard.glb"),
    ];
    let mut shared_uris = None;
    for path in &paths {
        export_equipment(path).unwrap();
        let parsed = gltf::Gltf::from_slice(&fs::read(path).unwrap()).unwrap();
        let uris = parsed
            .images()
            .map(|image| {
                let gltf::image::Source::Uri { uri, mime_type } = image.source() else {
                    panic!("individual equipment must reference shared PNG files");
                };
                assert_eq!(mime_type, Some("image/png"));
                assert_eq!(Path::new(uri).components().count(), 1);
                uri.to_owned()
            })
            .collect::<Vec<_>>();
        assert_eq!(uris.len(), 3);
        for (uri, expected) in uris.iter().zip([
            MAPS.base_color_png,
            MAPS.normal_png,
            MAPS.occlusion_png.unwrap(),
        ]) {
            assert_eq!(fs::read(directory.join(uri)).unwrap(), expected);
        }
        if let Some(expected) = &shared_uris {
            assert_eq!(&uris, expected);
        } else {
            shared_uris = Some(uris);
        }
        let material = parsed.materials().next().unwrap();
        assert_eq!(
            material
                .pbr_metallic_roughness()
                .base_color_texture()
                .unwrap()
                .texture()
                .source()
                .index(),
            0
        );
        assert_eq!(
            material
                .normal_texture()
                .unwrap()
                .texture()
                .source()
                .index(),
            1
        );
        assert_eq!(
            material
                .occlusion_texture()
                .unwrap()
                .texture()
                .source()
                .index(),
            2
        );
        assert_eq!(material.alpha_mode(), gltf::material::AlphaMode::Mask);
        // Only mesh data remains in the GLB; texture bytes cannot be retained per asset.
        assert!(parsed.blob.as_ref().unwrap().len() < MAPS.base_color_png.len());
    }
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 5);
    let uris = shared_uris.unwrap();
    // A corrupted content-addressed file must fail export, not silently be reused.
    fs::write(directory.join(&uris[0]), b"damaged").unwrap();
    let error = export_equipment(&paths[0]).unwrap_err();
    assert!(error.to_string().contains("unexpected contents"));
    for path in paths
        .into_iter()
        .chain(uris.into_iter().map(|uri| directory.join(uri)))
    {
        fs::remove_file(path).unwrap();
    }
    fs::remove_dir(directory).unwrap();
}
