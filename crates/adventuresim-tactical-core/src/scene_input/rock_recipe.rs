//! Geological rock recipe sampling for a stable obstacle seed.
use super::*;

pub(super) fn rock_recipe(seed: u64) -> RockRecipe {
    let archetype = match streams::ROCK_ARCHETYPE.rng(seed, &[]).index(3) {
        0 => RockArchetype::Rounded,
        1 => RockArchetype::Angular,
        _ => RockArchetype::Slab,
    };
    let lithology = match streams::ROCK_LITHOLOGY.rng(seed, &[]).index(3) {
        0 => RockLithology::Granite,
        1 => RockLithology::Limestone,
        _ => RockLithology::Sandstone,
    };
    let base_dimensions = match archetype {
        RockArchetype::Rounded => [128_u16, 104, 120],
        RockArchetype::Angular => [136, 112, 124],
        RockArchetype::Slab => [142, 72, 132],
    };
    let dimensions_cm = core::array::from_fn(|axis| {
        let offset = streams::ROCK_DIMENSION.rng(seed, &[axis as u64]).index(17) as i16 - 8;
        base_dimensions[axis].saturating_add_signed(offset)
    });
    RockRecipe {
        seed,
        archetype,
        lithology,
        dimensions_cm,
        collision_radius_cm: (ROCK_RADIUS_METRES * 100.0) as u16,
    }
}
