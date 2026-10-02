use super::impostor::{BEECH_TREE_BAKE_STYLE, OAK_TREE_BAKE_STYLE, TreeBakeStyle};
use super::{
    COMMON_BEECH_PARAMETERS, OAK_GNARLING_SHOWCASE, OakGnarlingParameters, TreeBranchSegment,
    TreeLeaf, procedural_oak_leaves, procedural_oak_skeleton_with_gnarling,
    procedural_tree_skeleton, procedural_woody_plant_leaves, procedural_woody_plant_skeleton,
};
use crate::presentation::{SceneEnvironment, unit_hash};
use bevy::prelude::*;
use fabelgeist_determinism::splitmix64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TreePresentationSpecies {
    EnglishOak,
    CommonBeech,
}

impl TreePresentationSpecies {
    pub(in crate::presentation) fn name(self) -> &'static str {
        match self {
            Self::EnglishOak => "English oak",
            Self::CommonBeech => "common beech",
        }
    }

    pub(super) fn cache_salt(self) -> u64 {
        match self {
            Self::EnglishOak => 0,
            Self::CommonBeech => 0xbeec_5eed_0000_0001,
        }
    }

    pub(super) fn bake_style(self) -> TreeBakeStyle {
        match self {
            Self::EnglishOak => OAK_TREE_BAKE_STYLE,
            Self::CommonBeech => BEECH_TREE_BAKE_STYLE,
        }
    }
}

pub(crate) fn tree_species_for_site(
    position: Vec3,
    environment: &SceneEnvironment,
) -> TreePresentationSpecies {
    let canopy = crate::presentation::procedural::bps(environment.canopy_bps);
    let moisture = crate::presentation::procedural::bps(environment.weather.ground_moisture_bps);
    let wetland = crate::presentation::procedural::bps(environment.wetland_bps);
    let cultivation = crate::presentation::procedural::bps(environment.cultivation_bps);
    // Beech is concentrated in mesic, closed-canopy communities. Using a
    // 30-metre community key produces stands instead of tree-by-tree confetti
    // and places it where the existing canopy mask already strongly suppresses
    // grass. Species remains deterministic presentation data until the compact
    // server tree recipe grows an explicit species field.
    let probability =
        (canopy * 0.62 + moisture * 0.26 - wetland * 0.38 - cultivation * 0.18 - 0.12)
            .clamp(0.0, 0.68);
    let community_x = (position.x / 30.0).floor() as i32;
    let community_z = (position.z / 30.0).floor() as i32;
    let community = ((community_x as u32 as u64) << 32) | community_z as u32 as u64;
    let hash = splitmix64(oak_site_key(environment) ^ community ^ 0xbeec_7a1d);
    if unit_hash(hash) < probability {
        TreePresentationSpecies::CommonBeech
    } else {
        TreePresentationSpecies::EnglishOak
    }
}

pub(in crate::presentation) fn canopy_competition(canopy_bps: u16) -> f32 {
    let normalized = crate::presentation::procedural::bps(canopy_bps);
    normalized * normalized * (3.0 - 2.0 * normalized)
}

pub(super) fn oak_site_key(environment: &SceneEnvironment) -> u64 {
    let location = u64::from(environment.latitude_microdegrees as u32) << 32
        | u64::from(environment.longitude_microdegrees as u32);
    let terrain = u64::from(environment.hilly_bps)
        | u64::from(environment.wetland_bps) << 14
        | u64::from(environment.cultivation_bps) << 28
        | u64::from(environment.canopy_bps) << 42;
    splitmix64(
        location ^ terrain ^ (environment.absolute_elevation_metres as i64 as u64).rotate_left(9),
    )
}

pub(super) fn oak_gnarling_for_site(
    mut recipe: OakGnarlingParameters,
    environment: &SceneEnvironment,
    tree_seed: u64,
) -> OakGnarlingParameters {
    let canopy = crate::presentation::procedural::bps(environment.canopy_bps);
    let open_exposure = 1.0 - canopy;
    let slope = crate::presentation::procedural::bps(environment.hilly_bps);
    let wetland = crate::presentation::procedural::bps(environment.wetland_bps);
    let cultivation = crate::presentation::procedural::bps(environment.cultivation_bps);
    let elevation =
        ((f32::from(environment.absolute_elevation_metres) - 40.0) / 900.0).clamp(0.0, 1.0);
    let susceptibility = 0.72 + unit_hash(splitmix64(tree_seed ^ 0x5355_5343)) * 0.28;
    let wind_exposure =
        (open_exposure * 0.46 + slope * 0.34 + elevation * 0.2).clamp(0.0, 1.0) * susceptibility;
    let age_and_wounds = unit_hash(splitmix64(tree_seed ^ 0x4147_4557));
    let location = u64::from(environment.latitude_microdegrees as u32) << 32
        | u64::from(environment.longitude_microdegrees as u32);
    recipe.stress_azimuth_radians =
        unit_hash(splitmix64(location ^ 0x5749_4e44)) * core::f32::consts::TAU;
    let add = |value: f32, stress: f32| (value + stress).clamp(0.0, 1.0);
    recipe.root_spread = add(
        recipe.root_spread,
        slope * 0.34 + wetland * 0.24 + wind_exposure * 0.2,
    );
    recipe.root_meander = add(recipe.root_meander, slope * 0.28 + wetland * 0.18);
    recipe.root_exposure = add(recipe.root_exposure, slope * 0.5 + open_exposure * 0.12);
    recipe.root_forking = add(recipe.root_forking, slope * 0.2 + age_and_wounds * 0.12);
    recipe.trunk_lean = add(recipe.trunk_lean, wind_exposure * 0.62 + wetland * 0.18);
    recipe.trunk_sweep = add(recipe.trunk_sweep, wind_exposure * 0.7);
    recipe.trunk_twist = add(
        recipe.trunk_twist,
        wind_exposure * 0.24 + age_and_wounds * 0.16,
    );
    recipe.trunk_crooks = add(
        recipe.trunk_crooks,
        cultivation * 0.3 + age_and_wounds * 0.16,
    );
    recipe.taper_irregularity = add(
        recipe.taper_irregularity,
        wetland * 0.18 + cultivation * 0.22 + age_and_wounds * 0.14,
    );
    recipe.knot_frequency = add(
        recipe.knot_frequency,
        cultivation * 0.38 + age_and_wounds * 0.24,
    );
    recipe.knot_scale = add(recipe.knot_scale, cultivation * 0.24 + age_and_wounds * 0.2);
    recipe.burl_scale = add(recipe.burl_scale, wetland * 0.3 + age_and_wounds * 0.16);
    recipe.scaffold_droop = add(
        recipe.scaffold_droop,
        age_and_wounds * 0.18 + wetland * 0.12,
    );
    recipe.scaffold_sweep = add(recipe.scaffold_sweep, wind_exposure * 0.76);
    recipe.scaffold_contortion = add(
        recipe.scaffold_contortion,
        wind_exposure * 0.32 + age_and_wounds * 0.18,
    );
    recipe.crown_asymmetry = add(recipe.crown_asymmetry, wind_exposure * 0.82);
    recipe
}

pub(super) fn vista_tree_source(
    variant_seed: u64,
    competition: f32,
    species: TreePresentationSpecies,
) -> (Vec<TreeBranchSegment>, Vec<TreeLeaf>) {
    match species {
        TreePresentationSpecies::EnglishOak => {
            let branches = procedural_tree_skeleton(variant_seed, competition);
            let leaves = procedural_oak_leaves(variant_seed, &branches, competition);
            (branches, leaves)
        }
        TreePresentationSpecies::CommonBeech => {
            let branches =
                procedural_woody_plant_skeleton(variant_seed, competition, COMMON_BEECH_PARAMETERS);
            let leaves = procedural_woody_plant_leaves(
                variant_seed,
                &branches,
                competition,
                COMMON_BEECH_PARAMETERS,
            );
            (branches, leaves)
        }
    }
}

pub(super) fn playable_tree_source(
    species: TreePresentationSpecies,
    variant_seed: u64,
    variant_index: usize,
    competition: f32,
    environment: &SceneEnvironment,
) -> (Vec<TreeBranchSegment>, Vec<TreeLeaf>) {
    match species {
        TreePresentationSpecies::EnglishOak => {
            let gnarling = oak_gnarling_for_site(
                OAK_GNARLING_SHOWCASE[variant_index],
                environment,
                variant_seed,
            );
            let branches =
                procedural_oak_skeleton_with_gnarling(variant_seed, competition, gnarling);
            let leaves = procedural_oak_leaves(variant_seed, &branches, competition);
            (branches, leaves)
        }
        TreePresentationSpecies::CommonBeech => {
            let branches =
                procedural_woody_plant_skeleton(variant_seed, competition, COMMON_BEECH_PARAMETERS);
            let leaves = procedural_woody_plant_leaves(
                variant_seed,
                &branches,
                competition,
                COMMON_BEECH_PARAMETERS,
            );
            (branches, leaves)
        }
    }
}

pub(in crate::presentation) fn vista_tree_species(
    environment: Option<&SceneEnvironment>,
    position: Vec2,
) -> TreePresentationSpecies {
    environment.map_or(TreePresentationSpecies::EnglishOak, |environment| {
        tree_species_for_site(Vec3::new(position.x, 0.0, position.y), environment)
    })
}
