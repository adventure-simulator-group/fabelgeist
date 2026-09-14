use super::*;
use adventuresim_character_creator::garment::{DrapeStage, DrapedGarment, transfer_skin};

#[test]
#[ignore = "requires MHR_ASSETS and the measured drape acceptance output"]
fn draped_chainmail_exports_under_plate() -> Result<()> {
    let assets = std::env::var_os("MHR_ASSETS").context("set MHR_ASSETS")?;
    let model = load_body_model(std::path::Path::new(&assets), 1, false, &Device::default())?;
    let mut recipe = CharacterRecipe::default();
    recipe.clothing.clear();
    recipe.armor = Some(fabelgeist_armor::Armor::default());
    let selection = GarmentSelection::chainmail();
    recipe.garments = vec![selection.clone()];
    let generated = generate_character(&model, &recipe)?;
    let source: serde_json::Value = serde_json::from_slice(&std::fs::read(
        std::env::temp_dir().join(format!("fabelgeist-drape-{:?}.json", selection.preset)),
    )?)?;
    let source_body: Vec<[f32; 3]> = serde_json::from_value(source["body"].clone())?;
    anyhow::ensure!(
        source_body == generated.positions,
        "cached drape belongs to a different body"
    );
    let positions: Vec<[f32; 3]> = serde_json::from_value(source["garment"].clone())?;
    let input = drape_preview::input(&model, &generated, selection);
    let (indices, weights) = transfer_skin(&input, &positions)?;
    let mut garment = DrapedGarment {
        preset: input.selection.preset,
        name: "Chainmail shirt".into(),
        fabric: FabricPreset::Chainmail,
        positions,
        indices,
        weights,
        stage: DrapeStage::Settling { step: 180, of: 180 },
        normals: serde_json::from_value(source["normals"].clone())?,
        faces: serde_json::from_value(source["garment_faces"].clone())?,
        texcoords: serde_json::from_value(source["uv"].clone())?,
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chainmail/chainmail-under-armor.glb");
    let body = fabelgeist_bvh::TriangleBvh::new(
        generated
            .positions
            .iter()
            .copied()
            .map(fabelgeist_math::Vec3::from_array)
            .collect(),
        model.mhr.character.mesh.faces.clone(),
    );
    println!(
        "before final reconciliation: {:?}",
        garment.validate_armor(recipe.armor.as_ref().unwrap())
    );
    let reconciled = garment.finish_armor(
        recipe.armor.as_ref().unwrap(),
        &body,
        0.0035,
        &input.selection.drape.armor_fit,
    );
    println!(
        "final body/self contacts: {:?}",
        garment.validate_contacts(&body)
    );
    std::fs::write(
        path.with_extension("diagnostic.json"),
        serde_json::to_vec(&serde_json::json!({
            "body": generated.positions, "body_faces": model.mhr.character.mesh.faces,
            "garment": garment.positions, "garment_faces": garment.faces,
            "normals": garment.normals, "uv": garment.texcoords,
            "plates": fabelgeist_armor::build(recipe.armor.as_ref().unwrap()).map_err(anyhow::Error::msg)?
                .into_iter().map(|p| serde_json::json!({"name":p.name,"positions":p.mesh.positions,"faces":p.mesh.faces})).collect::<Vec<_>>(),
        }))?,
    )?;
    reconciled?;
    (garment.indices, garment.weights) = transfer_skin(&input, &garment.positions)?;
    export_character(
        &path,
        &model,
        &recipe,
        &EquipmentCatalog(vec![], Default::default()),
        &BracerDesign::default(),
        &BreastplateDesign::default(),
        Some(&[garment]),
    )?;
    let bytes = std::fs::read(&path)?;
    let glb = gltf::Gltf::from_slice(&bytes)?;
    let mail = glb
        .materials()
        .find(|m| m.name() == Some("Chainmail shirt"))
        .context("export omitted chainmail")?;
    assert_eq!(mail.alpha_mode(), gltf::material::AlphaMode::Mask);
    assert_eq!(mail.pbr_metallic_roughness().metallic_factor(), 1.0);
    assert!(mail.normal_texture().is_some());
    assert!(mail.pbr_metallic_roughness().base_color_texture().is_some());
    assert!(glb.materials().any(|m| m.name() == Some("Breastplate")));
    println!("verified chainmail beneath plate: {}", path.display());
    Ok(())
}
