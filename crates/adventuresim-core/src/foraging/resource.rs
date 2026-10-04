//! Checked admission of serialized resource keys into the authored catalog.

use super::ForageSource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForageBiome {
    Plains,
    Forest,
    Hills,
    RiverWetGround,
    SeaCoast,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForageRarity {
    Common,
    Uncommon,
    Rare,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForageResource {
    pub item_id: &'static str,
    pub name: &'static str,
    pub rarity: ForageRarity,
    pub biomes: &'static [ForageBiome],
    pub yield_min: u16,
    pub yield_max: u16,
    pub source: ForageSource,
}

use ForageBiome::{Forest, Hills, Plains, RiverWetGround, SeaCoast};

pub const FORAGE_RESOURCES: &[ForageResource] = &[
    ForageResource {
        item_id: "wild_berries",
        name: "Wild berries",
        rarity: ForageRarity::Common,
        biomes: &[Plains, Forest, Hills],
        yield_min: 1,
        yield_max: 4,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "root_vegetables",
        name: "Wild roots",
        rarity: ForageRarity::Common,
        biomes: &[Plains, Forest, Hills, RiverWetGround],
        yield_min: 1,
        yield_max: 3,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "hazelnuts",
        name: "Hazelnuts",
        rarity: ForageRarity::Uncommon,
        biomes: &[Forest, Hills],
        yield_min: 1,
        yield_max: 3,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "wild_mushrooms",
        name: "Wild mushrooms",
        rarity: ForageRarity::Uncommon,
        biomes: &[Forest, RiverWetGround],
        yield_min: 1,
        yield_max: 3,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "garlic",
        name: "Wild garlic",
        rarity: ForageRarity::Uncommon,
        biomes: &[Forest, RiverWetGround],
        yield_min: 1,
        yield_max: 2,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "sage",
        name: "Sage",
        rarity: ForageRarity::Rare,
        biomes: &[Plains, Hills],
        yield_min: 1,
        yield_max: 2,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "willow_bark",
        name: "Willow bark",
        rarity: ForageRarity::Uncommon,
        biomes: &[Forest, RiverWetGround],
        yield_min: 1,
        yield_max: 2,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "poppy",
        name: "Poppy",
        rarity: ForageRarity::Uncommon,
        biomes: &[Plains, Hills],
        yield_min: 1,
        yield_max: 2,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "comfrey",
        name: "Comfrey",
        rarity: ForageRarity::Uncommon,
        biomes: &[Plains, RiverWetGround],
        yield_min: 1,
        yield_max: 2,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "watercress",
        name: "Watercress",
        rarity: ForageRarity::Common,
        biomes: &[RiverWetGround],
        yield_min: 1,
        yield_max: 4,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "seaweed",
        name: "Seaweed",
        rarity: ForageRarity::Common,
        biomes: &[SeaCoast],
        yield_min: 1,
        yield_max: 4,
        source: ForageSource::Plants,
    },
    ForageResource {
        item_id: "raw_venison",
        name: "Raw venison",
        rarity: ForageRarity::Uncommon,
        biomes: &[Forest, Hills, Plains],
        yield_min: 2,
        yield_max: 6,
        source: ForageSource::HighGame,
    },
    ForageResource {
        item_id: "raw_fowl",
        name: "Raw fowl",
        rarity: ForageRarity::Common,
        biomes: &[Plains, Forest, RiverWetGround],
        yield_min: 1,
        yield_max: 3,
        source: ForageSource::LowGame,
    },
    ForageResource {
        item_id: "raw_fish",
        name: "Raw fish",
        rarity: ForageRarity::Common,
        biomes: &[RiverWetGround, SeaCoast],
        yield_min: 1,
        yield_max: 4,
        source: ForageSource::Fish,
    },
    ForageResource {
        item_id: "raw_beast_meat",
        name: "Raw beast meat",
        rarity: ForageRarity::Uncommon,
        biomes: &[Plains, Forest, Hills],
        yield_min: 1,
        yield_max: 3,
        source: ForageSource::HarmfulBeasts,
    },
];

impl TryFrom<&str> for ForageResource {
    type Error = UnknownForageResource;

    fn try_from(encoded: &str) -> Result<Self, Self::Error> {
        for resource in FORAGE_RESOURCES {
            if resource.item_id == encoded {
                return Ok(*resource);
            }
        }
        Err(UnknownForageResource {
            encoded: encoded.into(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownForageResource {
    encoded: String,
}

impl std::fmt::Display for UnknownForageResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Unknown forage resource: {}", self.encoded)
    }
}

impl std::error::Error for UnknownForageResource {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_yield_ranges_always_produce_positive_units() {
        for resource in FORAGE_RESOURCES {
            assert!(resource.yield_min > 0, "{}", resource.item_id);
            assert!(
                resource.yield_min <= resource.yield_max,
                "{}",
                resource.item_id
            );
        }
    }

    #[test]
    fn resource_admission_keeps_catalog_identity_and_rejects_altered_keys() {
        for expected in FORAGE_RESOURCES {
            assert_eq!(ForageResource::try_from(expected.item_id), Ok(*expected));
        }
        for encoded in ["", "sage ", "Sage", "not_a_resource", "wild berries"] {
            let error = ForageResource::try_from(encoded).unwrap_err();
            assert_eq!(error.encoded, encoded);
            assert!(error.to_string().contains(encoded));
        }
    }
}
