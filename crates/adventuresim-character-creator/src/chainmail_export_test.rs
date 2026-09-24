use super::*;
use adventuresim_character_creator::{
    garment::{DrapeStage, DrapedGarment, pattern::shapes, transfer_skin},
    inventory::Article,
};

#[test]
#[ignore = "requires MHR_ASSETS and the measured drape acceptance output"]
fn draped_chainmail_exports() -> Result<()> {
    let assets = std::env::var_os("MHR_ASSETS").context("set MHR_ASSETS")?;
    let model = load_body_model(std::path::Path::new(&assets), 1, false, &Device::default())?;
    let catalog = EquipmentCatalog(ItemCatalog::new(vec![], CatalogDesigns::authored())?);
    let mut recipe = CharacterRecipe {
        inventory: Default::default(),
        ..CharacterRecipe::default()
    };
    let selection = GarmentSelection::chainmail();
    let id = recipe.inventory.add(Article::Draped(selection.clone()));
    recipe
        .inventory
        .wear(id, &catalog)
        .map_err(|conflict| anyhow::anyhow!("{conflict:?}"))?;
    let generated = generate_character(&model, &recipe)?;
    let source: serde_json::Value = serde_json::from_slice(&std::fs::read(
        std::env::temp_dir().join(format!("fabelgeist-drape-{}.json", shapes::SHIRT.name)),
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
        form: input.selection.form(),
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
        .join("../../target/chainmail/chainmail.glb");
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
        "final body/self contacts: {:?}",
        garment.contact_issues(&body)
    );
    std::fs::write(
        path.with_extension("diagnostic.json"),
        serde_json::to_vec(&serde_json::json!({
            "body": generated.positions, "body_faces": model.mhr.character.mesh.faces,
            "garment": garment.positions, "garment_faces": garment.faces,
            "normals": garment.normals, "uv": garment.texcoords,
        }))?,
    )?;
    (garment.indices, garment.weights) = transfer_skin(&input, &garment.positions)?;
    export_character(&path, &model, &recipe, &catalog, Some(&[garment]))?;
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
    println!("verified chainmail: {}", path.display());
    Ok(())
}
