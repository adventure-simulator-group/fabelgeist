use super::*;
use adventuresim_character_creator::{
    garment::{DrapeStage, SettledGarment, drape, pattern::shapes},
    inventory::Article,
};

/// Bodies the saved garment is worn on, as the studio rolls them.
const WEARER_SEEDS: [u64; 3] = [7, 1545, 90210];
const RANDOM_IDENTITY_LIMIT: f32 = 1.35;
/// How much further from a new wearer than from its own body a fitted garment
/// may sit on average, in metres: it keeps the fit it settled with.
const MAXIMUM_ADDED_DISTANCE_M: f32 = 0.01;

/// Whether a drape or fit reported cloth passing through the wearer.
fn crosses(warnings: &[String]) -> bool {
    warnings
        .iter()
        .any(|warning| warning.contains("intersect the wearer"))
}

fn wearer(seed: u64) -> CharacterRecipe {
    let mut recipe = CharacterRecipe {
        inventory: Default::default(),
        ..CharacterRecipe::default()
    };
    recipe.proportions =
        adventuresim_core::character_proportions::CharacterProportions::from_character_id(seed);
    let mut rng = StdRng::seed_from_u64(seed);
    for value in &mut recipe.identity {
        *value = rng.random_range(-RANDOM_IDENTITY_LIMIT..=RANDOM_IDENTITY_LIMIT);
    }
    recipe
}

fn mean_distance(garment: &[[f32; 3]], body: &fabelgeist_bvh::TriangleBvh) -> f32 {
    garment
        .iter()
        .map(|p| {
            body.closest_point(fabelgeist_math::Vec3::from_array(*p), f32::MAX)
                .expect("the body has a surface")
                .2
        })
        .sum::<f32>()
        / garment.len() as f32
}

/// A garment drapes without crossing its wearer, and once settled on the
/// canonical body is worn on other bodies without simulating: it keeps its
/// fit on them, stays clear of them, and exports.
#[test]
#[ignore = "requires MHR_ASSETS and a compute-capable GPU"]
fn a_settled_garment_fits_other_bodies_without_draping() -> Result<()> {
    let assets = std::env::var_os("MHR_ASSETS").context("set MHR_ASSETS")?;
    let model = load_body_model(std::path::Path::new(&assets), 1, false, &Device::default())?;
    let catalog = EquipmentCatalog(ItemCatalog::new(vec![], CatalogDesigns::authored())?);
    let shape = std::env::var("SETTLED_TEST_SHAPE").unwrap_or_else(|_| shapes::TUNIC.name.into());
    let shape = shapes::SHAPES
        .iter()
        .find(|candidate| candidate.name == shape)
        .context("unknown SETTLED_TEST_SHAPE")?;
    let selection = GarmentSelection::from_shape(shape);

    let canonical = generate_character(&model, &wearer_free())?;
    let input = drape_preview::input(&model, &canonical, selection.clone());
    let started = std::time::Instant::now();
    let outcome = drape(
        input.clone(),
        None,
        &std::sync::atomic::AtomicBool::new(false),
        |_| {},
    );
    println!("drape warnings: {:?}", outcome.warnings);
    assert!(!crosses(&outcome.warnings), "the drape crosses the wearer");
    let settled = outcome.result?;
    println!(
        "draped {} in {:.1} s",
        shape.name,
        started.elapsed().as_secs_f32()
    );
    let settled_mean = mean_distance(
        &settled.positions,
        &fabelgeist_bvh::TriangleBvh::new(
            canonical
                .positions
                .iter()
                .copied()
                .map(fabelgeist_math::Vec3::from_array)
                .collect(),
            model.mhr.character.mesh.faces.clone(),
        ),
    );
    println!("settled {settled_mean:.4} m from its body on average");
    let saved = SettledGarment::capture(&input, &settled)?;
    let json = serde_json::to_vec_pretty(&saved)?;
    println!(
        "saved {} vertices in {} KiB",
        saved.drape.vertex_count(),
        json.len() / 1024
    );
    let saved: SettledGarment = serde_json::from_slice(&json)?;

    for seed in WEARER_SEEDS {
        let recipe = wearer(seed);
        let body = generate_character(&model, &recipe)?;
        let input = drape_preview::input(&model, &body, saved.selection.clone());
        let started = std::time::Instant::now();
        let outcome = saved.drape.wear(&input);
        let garment = outcome.result?;
        let seconds = started.elapsed().as_secs_f32();
        assert_eq!(garment.stage, DrapeStage::Worn);
        let wearer = fabelgeist_bvh::TriangleBvh::new(
            body.positions
                .iter()
                .copied()
                .map(fabelgeist_math::Vec3::from_array)
                .collect(),
            model.mhr.character.mesh.faces.clone(),
        );
        let mean = mean_distance(&garment.positions, &wearer);
        println!(
            "body {seed}: fitted in {seconds:.2} s, mean distance {mean:.4} m, \
             warnings: {:?}",
            outcome.warnings
        );
        assert!(
            mean < settled_mean + MAXIMUM_ADDED_DISTANCE_M,
            "body {seed}: the garment left the wearer ({mean} m)"
        );
        garment
            .validate_body_clearance(&wearer)
            .with_context(|| format!("body {seed}"))?;
        assert!(
            !crosses(&outcome.warnings),
            "body {seed}: the garment crosses the wearer"
        );
        std::fs::write(
            std::env::temp_dir().join(format!("fabelgeist-settled-{}-{seed}.json", shape.name)),
            serde_json::to_vec(&serde_json::json!({
                "body": body.positions,
                "body_faces": model.mhr.character.mesh.faces,
                "garment": garment.positions,
                "garment_faces": garment.faces,
            }))?,
        )?;
    }

    // Worn from the inventory, it exports without being draped.
    let mut recipe = wearer(WEARER_SEEDS[0]);
    let id = recipe.inventory.add(Article::Settled(saved));
    recipe
        .inventory
        .wear(id, &catalog)
        .map_err(|conflict| anyhow::anyhow!("{conflict:?}"))?;
    let path = std::env::temp_dir().join(format!("fabelgeist-settled-{}.glb", shape.name));
    for warning in export_character(&path, &model, &recipe, &catalog, None)? {
        println!("export warning: {warning}");
    }
    let parsed = gltf::Gltf::from_slice(&std::fs::read(&path)?)?;
    assert_eq!(parsed.meshes().count(), 2);
    println!("exported {}", path.display());
    Ok(())
}

/// The canonical body, wearing nothing.
fn wearer_free() -> CharacterRecipe {
    CharacterRecipe {
        inventory: Default::default(),
        ..CharacterRecipe::default()
    }
}
