use super::*;
use adventuresim_character_creator::{
    armor_recipes::ParametricDesign,
    device_fit::{self, FitBody, Fitted, Realization},
};
use fabelgeist_armor::{
    GarmentArmorDesign, GarmentArmorKind, GarmentPlateShape, Millimeters, Permille,
    WrappedTassetDesign,
};

#[test]
fn wrapped_waist_assembly_fits_completed_fauld_and_morph() -> Result<()> {
    let (body, morphs) = armor_fixture::load(&std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/animations/biped/unarmed/base.glb"
    ))?)?;
    let gpu = adventuresim_character_creator::armor_gpu()?;
    let topology = FitBody {
        faces: &body.faces,
        texcoords: &body.texcoords,
        joint_indices: &body.joint_indices,
        joint_weights: &body.joint_weights,
        joint_names: &body.joint_names,
    };
    let wearer = Realization {
        positions: &body.positions,
        normals: &body.normals,
        joints: &body.global_joint_states,
        device: &body.device,
    };
    let morph = &morphs[0];
    let targets = [(
        morph.name.as_str(),
        Realization {
            positions: &morph.positions,
            normals: &morph.normals,
            joints: &morph.global_joint_states,
            device: &morph.device,
        },
    )];
    let ParametricDesign::WaistAssembly(mut design) =
        adventuresim_character_creator::armor_recipes::recipe("tassets")
            .context("tasset recipe")?
    else {
        anyhow::bail!("expected waist assembly");
    };
    design.tassets.lame_count = 8;
    design.tassets.plate_shape = GarmentPlateShape::WrappedTassets(WrappedTassetDesign {
        upper_edge_slope: Permille(200),
        ..Default::default()
    });
    let design = ParametricDesign::WaistAssembly(design);
    let fitted = device_fit::fit_recipe(gpu, &topology, &wearer, &targets, &design, "worn", &[])?;
    assert_eq!(fitted.endpoints.len(), 1);
    let target = &fitted.endpoints[0];
    assert_eq!(fitted.base.indices, target.indices);
    assert_eq!(fitted.base.positions.len(), target.positions.len());
    for part in [&fitted.base, target] {
        assert!(part.positions.iter().flatten().all(|p| p.is_finite()));
        assert_eq!(part.components.len(), 2);
        let tasset = &part.components[1];
        assert!(
            part.positions[tasset.vertices.clone()]
                .iter()
                .any(|p| p[0] > 0.0)
        );
        assert!(
            part.positions[tasset.vertices.clone()]
                .iter()
                .any(|p| p[0] < 0.0)
        );
    }
    Ok(())
}

#[test]
fn wrapped_tassets_reject_nonpositive_final_section_radii() -> Result<()> {
    use fabelgeist_armor::gpu::wrapped_tassets::{self, COURSE_ROWS, HULL_START, HULL_WORDS};
    let gpu = adventuresim_character_creator::armor_gpu()?;
    for (slope, radii, valid) in [
        (0, [0.1, 0.105, 0.01], true),
        (400, [0.1, 0.105, 0.01], false),
        // The first top-row bracket has radius zero at axial 2, but the
        // final root near axial 1 remains valid and must not be rejected.
        (100, [0.125, 0.125, 0.09375], true),
    ] {
        let mut design = GarmentArmorDesign::new(GarmentArmorKind::Tassets);
        design.lame_count = 1;
        design.plate_shape = GarmentPlateShape::WrappedTassets(WrappedTassetDesign {
            upper_edge_slope: Permille(slope),
            section_break: 0,
            ..Default::default()
        });
        let inner_start = HULL_START + 2 * HULL_WORDS;
        let fits = [1.0, -1.0].map(|sign| {
            let mut words = vec![0.0; (inner_start + COURSE_ROWS + 1) as usize];
            for index in [3, 7, 11, 12, 13, 14] {
                words[index] = 1.0;
            }
            words[16..21].copy_from_slice(&[0.0, 1.0, -1.0, 2.0, 2.0]);
            words[31] = 1.0;
            for (index, radius) in radii.into_iter().enumerate() {
                words[32 + 4 * index..36 + 4 * index].copy_from_slice(&[
                    sign * 0.3,
                    0.0,
                    radius,
                    radius,
                ]);
            }
            words[45] = inner_start as f32;
            gpu.upload(&words)
        });
        let [left, right] = fits;
        let (left, right) = (left?, right?);
        let statuses = [gpu.upload(&[0_u32])?, gpu.upload(&[0_u32])?];
        let mut batch = gpu.batch("invalid final tasset section regression");
        let mut part = wrapped_tassets::record_wrapped_tassets(
            gpu,
            &mut batch,
            &design,
            [&left, &right],
            [&statuses[0], &statuses[1]],
        )?;
        part.record_shells(gpu, &mut batch)?;
        batch.submit();
        let result = part.read(gpu);
        if valid {
            assert!(result.is_ok());
        } else {
            assert!(
                result.is_err(),
                "invalid final radii must not become a clamped shell"
            );
        }
    }
    Ok(())
}

#[test]
fn wrapped_tassets_fit_final_height_with_rigid_separate_thigh_ownership() -> Result<()> {
    let (body, _) = armor_fixture::load(&std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/animations/biped/unarmed/base.glb"
    ))?)?;
    let gpu = adventuresim_character_creator::armor_gpu()?;
    let topology = FitBody {
        faces: &body.faces,
        texcoords: &body.texcoords,
        joint_indices: &body.joint_indices,
        joint_weights: &body.joint_weights,
        joint_names: &body.joint_names,
    };
    let wearer = Realization {
        positions: &body.positions,
        normals: &body.normals,
        joints: &body.global_joint_states,
        device: &body.device,
    };
    for (slope, gauge) in [(0, 1), (200, 1), (400, 1), (400, 3)] {
        let mut garment = GarmentArmorDesign::new(GarmentArmorKind::Tassets);
        garment.lame_count = 8;
        garment.wall_thickness = Millimeters(gauge);
        garment.plate_shape = GarmentPlateShape::WrappedTassets(WrappedTassetDesign {
            upper_edge_slope: Permille(slope),
            ..Default::default()
        });
        let design = ParametricDesign::Garment(garment);
        let piece = device_fit::fit_recipe(gpu, &topology, &wearer, &[], &design, "worn", &[])?;
        let armor = device_fit::assemble_recipe(
            &design,
            piece,
            &Fitted {
                placement: "worn",
                morphs: &[],
                domain: &body.domain,
                joint_names: &body.joint_names,
                joints: &body.global_joint_states,
            },
        )?;
        if let Ok(directory) = std::env::var("ARMOR_FIXTURE_OUTPUT") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory)?;
            std::fs::write(
                directory.join("body.json"),
                serde_json::to_vec(&serde_json::json!({
                    "positions":body.positions,"normals":body.normals,"faces":body.faces
                }))?,
            )?;
            std::fs::write(
                directory.join(format!("wrapped-{slope}--worn.json")),
                serde_json::to_vec(&serde_json::json!({
                    "id":format!("wrapped-{slope}"),"placement":"worn","positions":armor.positions,
                    "normals":armor.normals,"indices":armor.indices,"components":armor.components
                }))?,
            )?;
        }
        assert!(armor.morphs.is_empty());
        assert_eq!(armor.components.len(), 2);
        assert!(
            armor
                .positions
                .iter()
                .chain(&armor.normals)
                .flatten()
                .all(|v| v.is_finite())
        );
        for (side, name) in ["l_upleg", "r_upleg"].into_iter().enumerate() {
            let component = &armor.components[side];
            let owner = body.joint_names.iter().position(|n| n == name).unwrap() as u32;
            assert!(
                armor.joint_indices[component.vertices.clone()]
                    .iter()
                    .all(|j| *j == [owner; 8])
            );
            let sign = if side == 0 { 1.0 } else { -1.0 };
            assert!(
                armor.positions[component.vertices.clone()]
                    .iter()
                    .all(|p| sign * p[0] >= 0.009_99)
            );
        }
        for face in armor.indices.as_chunks::<3>().0 {
            let [a, b, c] = face.map(|i| bevy::math::Vec3::from(armor.positions[i as usize]));
            assert!((b - a).cross(c - a).length_squared() > 1e-18);
        }
    }
    Ok(())
}
