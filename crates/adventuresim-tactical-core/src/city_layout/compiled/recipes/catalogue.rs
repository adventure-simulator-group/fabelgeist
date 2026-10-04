//! Occupied upper-house variations are explicit architectural catalogue data.
//! A construction repair must not substitute another seed, roof or storey
//! dimension merely because an earlier candidate becomes buildable.
use super::*;

const TOWN_DWELLING_SEEDS: [u64; 3] = [
    6514374187028306242,
    244875683185724721,
    15522022947284468661,
];
const MERCHANT_DWELLING_SEEDS: [u64; 3] =
    [7989866213631017260, 269418199818528039, 6006670756388891727];

pub(super) fn seed(archetype: BuildingArchetype, usage: BuildingUse, choice: usize) -> u64 {
    match (archetype, usage) {
        (BuildingArchetype::TownHouse, BuildingUse::Dwelling) => TOWN_DWELLING_SEEDS[choice],
        (BuildingArchetype::FachwerkMerchantHouse, BuildingUse::Dwelling) => {
            MERCHANT_DWELLING_SEEDS[choice]
        }
        _ => CURATED_RECIPE_SEEDS[choice],
    }
}

pub(super) fn exact_upper_dwelling(
    archetype: BuildingArchetype,
    usage: Option<BuildingUse>,
) -> bool {
    usage == Some(BuildingUse::Dwelling)
        && matches!(
            archetype,
            BuildingArchetype::TownHouse | BuildingArchetype::FachwerkMerchantHouse
        )
}
