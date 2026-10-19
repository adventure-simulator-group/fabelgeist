#![cfg(feature = "prepared")]

//! Behavioral rendering-asset pipeline checks; no simulation or timing runs.

use std::num::NonZeroU16;

use adventuresim_environment::prepared::*;
use adventuresim_environment::render::ForestPart;
use fabelgeist_math::units::{Metres, Ratio};

mod support;

#[test]
fn declared_grid_counts_fail_before_reading_or_allocating_array_payload() {
    use fabelgeist_botany::viewer::sim::{BranchInstance, LeafInstance, PlantInstance};
    let library = support::padded_fixture()
        .bake(VariantBudget::AllPlants)
        .unwrap();
    let (bytes, _) = encode_library(&library).unwrap();
    let count =
        |offset: usize| u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
    let mut first_grid = 16 + count(8);
    for record_size in [
        size_of::<PlantInstance>(),
        size_of::<BranchInstance>(),
        size_of::<LeafInstance>(),
    ] {
        first_grid += 8 + count(first_grid) * record_size;
    }
    let first_indices =
        first_grid + 8 + library.grids[0].levels[0].vertices.len() * size_of::<u32>();
    for offset in [first_grid, first_indices] {
        let mut malformed = bytes[..offset + 8].to_vec();
        malformed[offset..offset + 8].copy_from_slice(&u64::from(u32::MAX).to_le_bytes());
        assert!(matches!(
            decode_library(&malformed),
            Err(PreparedError::MeshCardinality)
        ));
    }
}

#[test]
fn bounded_sharing_preserves_sparse_and_dense_crowns_deterministically() {
    let mut unique = support::fixture();
    for plant in &mut unique.snapshot.plant_bounds {
        plant.species = 0;
    }
    let mut leaves = Vec::new();
    for mut leaf in unique.snapshot.leaves.drain(..) {
        leaf.species = 0;
        let root = unique.roots[leaf.plant as usize]
            .position
            .map(Metres::metres);
        leaf.center[0] = root[0];
        leaf.center[2] = root[2];
        let count = if leaf.plant < 3 { 1 } else { 8 };
        leaves.extend(std::iter::repeat_n(leaf, count));
    }
    unique.snapshot.leaves = leaves;
    let budget = VariantBudget::PerSpecies(NonZeroU16::new(2).unwrap());
    let library = unique.bake(budget).unwrap();
    let (bytes, id) = encode_library(&library).unwrap();
    let (repeated, repeated_id) = encode_library(&unique.bake(budget).unwrap()).unwrap();
    assert_eq!(bytes, repeated);
    assert_eq!(id, repeated_id);
    let repeated_unique = decode_unique(&encode_unique(&unique).unwrap()).unwrap();
    let population = unique.population(&library, id).unwrap();
    let repeated_population = repeated_unique.population(&library, id).unwrap();
    assert_eq!(
        encode_population(&population, &library, id).unwrap(),
        encode_population(&repeated_population, &library, id).unwrap()
    );
    for (index, placement) in population.placements.iter().enumerate() {
        let model = &library.models[placement.variant.index()];
        assert_eq!(model.leaves.count, if index < 3 { 1 } else { 8 });
        if model.source == placement.id {
            assert_eq!(placement.pose.yaw().radians(), 0.0);
        }
    }
    assert!(
        population
            .placements
            .iter()
            .any(|p| p.pose.yaw().radians() != 0.0)
    );
}

#[test]
fn frozen_unique_round_trip_preserves_rendered_input() {
    let unique = support::fixture();
    let bytes = encode_unique(&unique).unwrap();
    let decoded = decode_unique(&bytes).unwrap();
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&unique.snapshot.branches),
        bytemuck::cast_slice::<_, u8>(&decoded.snapshot.branches)
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&unique.snapshot.leaves),
        bytemuck::cast_slice::<_, u8>(&decoded.snapshot.leaves)
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&unique.snapshot.plant_bounds),
        bytemuck::cast_slice::<_, u8>(&decoded.snapshot.plant_bounds)
    );
    assert_eq!(unique.snapshot.bounds, decoded.snapshot.bounds);
    assert_eq!(unique.detail, decoded.detail);
    assert_eq!(
        unique.roots.iter().map(|p| p.id).collect::<Vec<_>>(),
        decoded.roots.iter().map(|p| p.id).collect::<Vec<_>>()
    );
    let terrain = decoded.snapshot.terrain.as_ref().unwrap();
    assert_eq!(
        terrain.indices,
        unique.snapshot.terrain.as_ref().unwrap().indices
    );
    assert_eq!(terrain.vertices[0].position, [-20.0, 0.0, -20.0]);
}

#[test]
fn bounded_bank_reuses_records_without_changing_population_identity() {
    let unique = support::fixture();
    let library = unique
        .bake(VariantBudget::PerSpecies(NonZeroU16::new(1).unwrap()))
        .unwrap();
    assert_eq!(library.models.len(), 3);
    assert_eq!(library.branches.len(), 3);
    let (bytes, id) = encode_library(&library).unwrap();
    let (loaded, loaded_id) = decode_library(&bytes).unwrap();
    assert_eq!(id, loaded_id);
    let population = unique.population(&loaded, id).unwrap();
    let encoded = encode_population(&population, &loaded, id).unwrap();
    let decoded = decode_population(&encoded, &loaded, id).unwrap();
    assert_eq!(decoded.placements.len(), 6);
    assert_eq!(decoded.frame.source.branches, 6);
    assert_eq!(decoded.placements[0].variant, decoded.placements[3].variant);
    assert_ne!(decoded.placements[0].id, decoded.placements[3].id);
    assert_eq!(
        decoded.placements[3].pose.origin().map(Metres::metres),
        [8.0, 0.0, 0.0]
    );
}

#[test]
fn every_plant_oracle_keeps_its_own_model_at_all_three_levels() {
    let unique = support::fixture();
    let library = unique.bake(VariantBudget::AllPlants).unwrap();
    let (_, id) = encode_library(&library).unwrap();
    let population = unique.population(&library, id).unwrap();
    for placement in &population.placements {
        let model = &library.models[placement.variant.index()];
        assert_eq!(model.source, placement.id);
        assert_eq!(placement.pose.horizontal(), Ratio::from_ratio(1.0));
        assert_eq!(placement.pose.vertical(), Ratio::from_ratio(1.0));
        for (part, span) in [
            (ForestPart::Branches, model.branches),
            (ForestPart::Leaves, model.leaves),
        ] {
            let class = MeshClass::for_records(part, span.count).unwrap().unwrap();
            let grid = library
                .grids
                .iter()
                .find(|grid| grid.class == class)
                .unwrap();
            for level in &grid.levels {
                assert!(!level.indices.is_empty());
            }
        }
        let branch = library.branches[model.branches.start as usize];
        let world = placement
            .pose
            .transform([branch.a[0], branch.a[1], branch.a[2]].map(Metres::from_metres));
        assert_eq!(world, placement.pose.origin());
    }
}

#[test]
fn malformed_or_mismatched_payloads_fail_before_rendering() {
    let unique = support::fixture();
    let bytes = encode_unique(&unique).unwrap();
    for length in [0, 8, 15, bytes.len() - 1] {
        assert!(decode_unique(&bytes[..length]).is_err());
    }
    let library = unique.bake(VariantBudget::AllPlants).unwrap();
    let (bytes, id) = encode_library(&library).unwrap();
    assert!(decode_library(&bytes[..bytes.len() - 1]).is_err());
    let population = unique.population(&library, id).unwrap();
    let population_bytes = encode_population(&population, &library, id).unwrap();
    let mut other = support::fixture().bake(VariantBudget::AllPlants).unwrap();
    other.detail.full_pixels += 1.0;
    let (_, other_id) = encode_library(&other).unwrap();
    assert!(decode_population(&population_bytes, &other, other_id).is_err());
    let mut invalid = support::fixture();
    invalid.snapshot.branches[0].plant = u32::MAX;
    assert!(encode_unique(&invalid).is_err());
}

#[test]
fn translated_rotated_stretched_placements_survive_asset_loading() {
    let unique = support::fixture();
    let library = unique.bake(VariantBudget::AllPlants).unwrap();
    let (_, id) = encode_library(&library).unwrap();
    let mut population = unique.population(&library, id).unwrap();
    let pose = TreePose::new(
        [12.0, 2.0, -4.0].map(Metres::from_metres),
        Ratio::from_ratio(0.6),
        Ratio::from_ratio(1.7),
        TreeYaw::from_radians(std::f32::consts::FRAC_PI_2).unwrap(),
    )
    .unwrap();
    population.placements[0].pose = pose;
    let bytes = encode_population(&population, &library, id).unwrap();
    let decoded = decode_population(&bytes, &library, id).unwrap();
    let point = decoded.placements[0]
        .pose
        .transform([1.0, 2.0, 3.0].map(Metres::from_metres))
        .map(Metres::metres);
    for (actual, expected) in point.into_iter().zip([13.8, 5.4, -4.6]) {
        assert!((actual - expected).abs() < 1e-5);
    }
    assert!(
        TreePose::new(
            [Metres::from_metres(0.0); 3],
            Ratio::from_ratio(0.0),
            Ratio::from_ratio(1.0),
            TreeYaw::from_radians(0.0).unwrap()
        )
        .is_err()
    );
}

#[test]
fn corrupt_mesh_addresses_and_metadata_are_rejected() {
    let mut library = support::fixture().bake(VariantBudget::AllPlants).unwrap();
    library.grids[0].levels[0].vertices[0] = 999;
    assert!(encode_library(&library).is_err());
    let mut library = support::padded_fixture()
        .bake(VariantBudget::AllPlants)
        .unwrap();
    library.grids[0].levels[0].indices[0] = u32::MAX;
    assert!(encode_library(&library).is_err());
    let mut library = support::fixture().bake(VariantBudget::AllPlants).unwrap();
    library.grids.pop();
    assert!(encode_library(&library).is_err());
    assert!(RecordCapacity::try_from(0).is_err());
    assert!(RecordCapacity::try_from(3).is_err());
    assert!(RecordCapacity::covering(u32::MAX).is_err());
    let unique = support::fixture();
    let library = unique.bake(VariantBudget::AllPlants).unwrap();
    let (_, id) = encode_library(&library).unwrap();
    let mut population = unique.population(&library, id).unwrap();
    population.frame.detail.leaf_blades += 1;
    assert!(encode_population(&population, &library, id).is_err());
}

#[test]
fn species_budget_preserves_bare_and_empty_geometry_states() {
    let mut unique = support::fixture();
    unique.snapshot.leaves.retain(|leaf| leaf.plant != 0);
    assert!(
        unique
            .bake(VariantBudget::PerSpecies(NonZeroU16::new(1).unwrap()))
            .is_err()
    );
    let library = unique
        .bake(VariantBudget::PerSpecies(NonZeroU16::new(2).unwrap()))
        .unwrap();
    let (_, id) = encode_library(&library).unwrap();
    let population = unique.population(&library, id).unwrap();
    let model = &library.models[population.placements[0].variant.index()];
    assert_eq!(model.leaves.count, 0);
    assert!(model.branches.count > 0);
    assert!(
        library
            .models
            .iter()
            .filter(|m| m.species == model.species)
            .count()
            <= 2
    );
}

#[test]
fn odd_fronds_have_bounds_for_every_stored_lod() {
    let mut unique = support::fixture();
    unique.detail.leaf_blades = 3;
    unique.snapshot.leaves[2].axis = [0.0, -1.0, 0.0];
    unique.snapshot.leaves[2].seed = 0.0;
    let library = unique.bake(VariantBudget::AllPlants).unwrap();
    let model = &library.models[2];
    let leaf = &library.leaves[model.leaves.start as usize];
    let extreme_y = leaf.center[1] - 12.8 * leaf.size;
    assert!(model.bounds.low[1].metres() <= extreme_y);
}

#[test]
fn export_small_rendering_oracles_when_requested() {
    let Some(directory) = std::env::var_os("FOLIAGE_FIXTURE_OUTPUT") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&directory).unwrap();
    let unique = support::fixture();
    let library = unique.bake(VariantBudget::AllPlants).unwrap();
    std::fs::write(
        directory.join("unique.forest"),
        encode_unique(&unique).unwrap(),
    )
    .unwrap();
    let (bytes, id) = encode_library(&library).unwrap();
    std::fs::write(directory.join("library.forest"), bytes).unwrap();
    let mut population = unique.population(&library, id).unwrap();
    std::fs::write(
        directory.join("population.forest"),
        encode_population(&population, &library, id).unwrap(),
    )
    .unwrap();
    population.placements[0].pose = TreePose::new(
        [0.0, 0.0, -5.0].map(Metres::from_metres),
        Ratio::from_ratio(0.6),
        Ratio::from_ratio(1.7),
        TreeYaw::from_radians(0.8).unwrap(),
    )
    .unwrap();
    std::fs::write(
        directory.join("stretched.forest"),
        encode_population(&population, &library, id).unwrap(),
    )
    .unwrap();
    let mut bare = support::fixture();
    bare.snapshot.leaves.clear();
    write_fixture(bare, &directory.join("bare"));
    let mut empty = support::fixture();
    empty.snapshot.branches.clear();
    empty.snapshot.leaves.clear();
    write_fixture(empty, &directory.join("empty"));
    let mut hidden = support::fixture();
    hidden.detail.leaves = false;
    write_fixture(hidden, &directory.join("hidden"));
    write_fixture(support::padded_fixture(), &directory.join("padded"));
}

#[test]
fn capacity_grids_share_topology_while_placements_keep_exact_botanical_spans() {
    let unique = support::padded_fixture();
    let library = unique.bake(VariantBudget::AllPlants).unwrap();
    assert!(library.grids.len() < library.models.len() * 2);
    let (bytes, id) = encode_library(&library).unwrap();
    let (loaded, loaded_id) = decode_library(&bytes).unwrap();
    assert_eq!(id, loaded_id);
    let population = unique.population(&loaded, id).unwrap();
    for placement in &population.placements {
        let model = &loaded.models[placement.variant.index()];
        assert_eq!(model.source, placement.id);
        assert!([3, 5].contains(&model.branches.count));
        assert!([5, 7].contains(&model.leaves.count));
        for (part, span) in [
            (ForestPart::Branches, model.branches),
            (ForestPart::Leaves, model.leaves),
        ] {
            let class = MeshClass::for_records(part, span.count).unwrap().unwrap();
            assert!(class.capacity.records() >= span.count);
            assert!(class.capacity.records() < span.count * 2);
            assert!(loaded.grids.iter().any(|grid| grid.class == class));
        }
    }
}

fn write_fixture(unique: PreparedUnique, directory: &std::path::Path) {
    std::fs::create_dir_all(directory).unwrap();
    std::fs::write(
        directory.join("unique.forest"),
        encode_unique(&unique).unwrap(),
    )
    .unwrap();
    let library = unique.bake(VariantBudget::AllPlants).unwrap();
    let (bytes, id) = encode_library(&library).unwrap();
    std::fs::write(directory.join("library.forest"), bytes).unwrap();
    let population = unique.population(&library, id).unwrap();
    std::fs::write(
        directory.join("population.forest"),
        encode_population(&population, &library, id).unwrap(),
    )
    .unwrap();
}
