//! Native file/JSON boundary joined to the admitted bank and actual source.

use super::*;
use adventuresim_environment::prepared::{
    ForestCounts, ForestFrame, LibraryId, RecordSpan, TreeBounds, TreeFocus, TreePlacement,
    TreePopulation, TreePose, TreeVisibility, TreeYaw, encode_population,
};
use adventuresim_tactical_core::prelude::SceneEnvironment;
use fabelgeist_botany::Biome;
use fabelgeist_math::units::{Metres, Ratio};

#[derive(Deserialize)]
struct Manifest {
    format: String,
    input_digest: String,
    scene_digest: String,
    environment: SceneEnvironment,
    library: LibraryId,
    models: usize,
    source_specimens: Vec<SourceBinding>,
    converted_models: Vec<ConvertedBinding>,
}

#[derive(Deserialize)]
struct SourceBinding {
    #[serde(deserialize_with = "decode_plant_id")]
    source_id: PlantId,
    species: TreePresentationSpecies,
    authored_seed: u64,
    source_leaf_cards: usize,
    centerline_runs: Vec<Vec<serde::de::IgnoredAny>>,
    leaf_groups: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
struct ConvertedBinding {
    #[serde(deserialize_with = "decode_plant_id")]
    source_id: PlantId,
    species: u16,
    bounds: TreeBounds,
    branches: RecordSpan,
    clusters: RecordSpan,
}

pub(super) fn load(options: Request, setup: &SceneSetupData) -> Result<Review> {
    let manifest_bytes = fs::read(options.directory.join("source.json"))?;
    let library_bytes = fs::read(options.directory.join("library.forest"))?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
    ensure!(
        manifest.format == TREE_BANK_SOURCE_FORMAT,
        "unknown source manifest format"
    );
    let bank = Arc::new(TreeBank::from_bytes(&library_bytes)?);
    ensure!(
        bank.id() == manifest.library,
        "manifest/bank content identity mismatch"
    );
    ensure!(
        manifest.input_digest == setup.input.digest()?,
        "source input mismatch"
    );
    ensure!(
        manifest.scene_digest == setup.generated.digest,
        "generated scene mismatch"
    );
    ensure!(
        serde_json::to_value(&manifest.environment)? == serde_json::to_value(&setup.environment)?,
        "source environment mismatch"
    );
    ensure!(
        manifest.models == bank.library().models.len(),
        "model count mismatch"
    );
    let id = options.source;
    let source = TreeSourceGeometry::find(id, &setup.environment)?;
    let binding = exactly_one(
        manifest
            .source_specimens
            .iter()
            .filter(|row| row.source_id == id),
    )?;
    ensure!(
        binding.species == source.species && binding.authored_seed == source.seed.to_u64(),
        "source recipe mismatch"
    );
    ensure!(
        binding.source_leaf_cards == source.cards
            && binding.centerline_runs.iter().map(Vec::len).sum::<usize>() == source.branches,
        "source count mismatch"
    );
    let converted = exactly_one(
        manifest
            .converted_models
            .iter()
            .filter(|row| row.source_id == id),
    )?;
    let mut matches = bank
        .library()
        .variant_indices()
        .zip(bank.library().models.iter())
        .filter(|(_, model)| model.source == id);
    let (variant, model) = matches.next().context("source missing from bank")?;
    ensure!(matches.next().is_none(), "ambiguous bank source");
    ensure!(
        converted.species == model.species.word()
            && converted.branches.start == model.branches.start
            && converted.branches.count == model.branches.count
            && converted.clusters.start == model.leaves.start
            && converted.clusters.count == model.leaves.count
            && serde_json::to_value(converted.bounds)? == serde_json::to_value(model.bounds)?,
        "converted model binding mismatch"
    );
    ensure!(
        model.branches.count as usize == source.branches
            && model.leaves.count as usize == binding.leaf_groups.len(),
        "converted counts mismatch"
    );
    let actual_species = bank
        .library()
        .species
        .get(model.species.index())
        .context("bank species metadata missing")?;
    let expected_species = &source.species_record;
    ensure!(
        actual_species.name == expected_species.name
            && actual_species.bark == expected_species.bark
            && actual_species.leaf == expected_species.leaf
            && actual_species.foliage == expected_species.foliage
            && actual_species.plants == expected_species.plants,
        "bank species does not match the production source"
    );
    let ground = setup
        .generated
        .terrain
        .height_at(Vec2::ZERO)
        .context("review root lacks production terrain support")?;
    ensure!(ground.is_finite(), "nonfinite review ground");
    let root = Vec3::new(0.0, ground, 0.0);
    let cameras = cameras::Cameras::from_source(&source, root, &setup.generated.terrain)?;
    let placement = TreePlacement {
        id,
        variant,
        pose: TreePose::new(
            root.to_array().map(Metres::from_metres),
            Ratio::from_ratio(1.0),
            Ratio::from_ratio(1.0),
            TreeYaw::from_radians(0.0)?,
        )?,
        visibility: TreeVisibility::unbounded(),
    };
    let frame = ForestFrame {
        label: "Production source/bank isolated review".into(),
        detail: bank.library().detail,
        time: 0.0,
        source: ForestCounts {
            plants: 1,
            branches: model.branches.count as usize,
            leaves: model.leaves.count as usize,
        },
        bounds: [
            (root + Vec3::from_array(model.bounds.low.map(Metres::metres))).to_array(),
            (root + Vec3::from_array(model.bounds.high.map(Metres::metres))).to_array(),
        ],
        biome: Biome::TemperateSeasonalForest,
        terrain: None,
        structures: None,
        focus: Some(TreeFocus {
            root: root.to_array().map(Metres::from_metres),
            height: source.maximum[1],
        }),
    };
    let population = TreePopulation {
        library: bank.id(),
        placements: vec![placement],
        frame,
    };
    let bytes = encode_population(&population, bank.library(), bank.id())?;
    let forest = Arc::new(SharedForest::from_population_bytes(
        Arc::clone(&bank),
        &bytes,
    )?);
    let metadata = serde_json::json!({
        "role": options.role, "source_id": id.word(), "species": source.species, "seed": source.seed.to_u64(),
        "bank": bank.id(), "population": forest.id(), "root_metres": root,
        "source_mesh_minimum": source.minimum, "source_mesh_maximum": source.maximum,
        "suppressed_generated_obstacles": setup.generated.obstacles.len(),
        "source_branches": source.branches, "source_cards": source.cards, "converted_clusters": model.leaves.count,
        "source_manifest_blake3": blake3::hash(&manifest_bytes).to_hex().to_string(),
        "library_blake3": blake3::hash(&library_bytes).to_hex().to_string(),
        "cameras": cameras, "fixed_bank_lod": "full", "bank_ground": "none; actual tactical terrain retained",
        "source_materials": "production full wood, cambered leaves and buds", "time_phase": super::super::CAPTURE_CLOCK_PHASE_SECONDS,
        "scope": "isolated static content, unrelated obstacles/scatter hidden, no arrival or motion claim"
    });
    Ok(Review {
        role: options.role,
        source: Some(source),
        forest,
        cameras,
        root,
        metadata,
    })
}

fn exactly_one<'a, T>(mut rows: impl Iterator<Item = &'a T>) -> Result<&'a T> {
    let row = rows.next().context("missing source binding")?;
    ensure!(rows.next().is_none(), "ambiguous source binding");
    Ok(row)
}

fn decode_plant_id<'de, D: serde::Deserializer<'de>>(
    decoder: D,
) -> std::result::Result<PlantId, D::Error> {
    u64::deserialize(decoder).map(PlantId::from_word)
}
