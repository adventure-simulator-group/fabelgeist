//! Establishment signs with period tool pictograms. Names belong to placed lots, not shared building recipes.
use adventuresim_world_schema::{person_names::NamePool, settlement_buildings::BuildingUse};
use bevy::math::{Quat, Vec2, Vec3};
use clap::ValueEnum;
use fabelgeist_determinism::mix64;
use serde::{Deserialize, Serialize};

mod emblems;
pub use emblems::TradeEmblem;
mod mounting;
pub use mounting::{MOUNTING_PLATE_THICKNESS_METRES, SignMounting};
mod site;
pub use site::SignSite;
#[cfg(feature = "sign-render")]
mod lettering;
#[cfg(feature = "sign-render")]
mod rendering;
#[cfg(feature = "sign-render")]
pub use lettering::SignTexture;
#[cfg(feature = "sign-render")]
pub use rendering::{ShopSignRenderCache, SignDetail, SignRenderAssets, SignRenderPart};

const NAME_STREAM: u64 = 0x7369_676e_6e61_6d65;
const STYLE_STREAM: u64 = 0x7369_676e_7374_796c;
pub const SIGN_PEDESTRIAN_CLEARANCE_METRES: f32 = 2.3;
pub const SIGN_MAX_PROJECTION_METRES: f32 = 1.5;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct EstablishmentId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ShopName {
    pub proprietor: String,
    pub trade: String,
}

impl ShopName {
    /// A stable business brand, without asserting that a generated resident owns it.
    pub fn for_establishment(id: EstablishmentId, usage: BuildingUse) -> Option<Self> {
        let trade = shop_trade(usage)?;
        let entropy = mix64(id.0 ^ NAME_STREAM);
        let names = if entropy & 1 == 0 {
            NamePool::Female
        } else {
            NamePool::Male
        };
        let given = names.choose(entropy >> 1);
        let surname = NamePool::Surname.choose(mix64(entropy));
        Some(Self {
            proprietor: format!("{given} {surname}’s"),
            trade: trade.to_owned(),
        })
    }

    pub fn text(&self) -> String {
        format!("{} {}", self.proprietor, self.trade)
    }
}

/// Only businesses with a public shopfront receive proprietor signs.
pub const fn shop_trade(usage: BuildingUse) -> Option<&'static str> {
    use BuildingUse::*;
    Some(match usage {
        Inn => "Tavern",
        GeneralShop => "General Store",
        Smithy => "Smithy",
        Weaponsmith => "Weaponsmith",
        Armorer => "Armourer",
        Tailor => "Tailor",
        Herbalist => "Herbalist",
        Bookshop => "Bookshop",
        Bakehouse => "Bakery",
        Brewery => "Brewery",
        Butcher => "Butcher",
        Stable => "Stables",
        Cooper => "Cooper",
        Carpenter => "Carpenter",
        Wheelwright => "Wheelwright",
        Cobbler => "Cobbler",
        Weaver => "Weaver",
        Dyer => "Dyer",
        Ropemaker => "Ropemaker",
        Chandler => "Chandler",
        Potter => "Potter",
        Fishmonger => "Fishmonger",
        Bathhouse => "Bathhouse",
        Apothecary => "Apothecary",
        PrintingHouse => "Print Shop",
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignMount {
    Wall,
    Projecting,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignFont {
    #[default]
    GrenzeGotisch,
    UnifrakturCook,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignFinish {
    PalePaint,
    DarkWood,
}

#[derive(Clone, Debug, bevy::prelude::Component, Serialize, Deserialize)]
#[component(immutable)]
pub struct ShopSign {
    pub emblem: Option<TradeEmblem>,
    pub name: ShopName,
    pub mount: SignMount,
    pub font: SignFont,
    pub finish: SignFinish,
}

impl ShopSign {
    pub fn for_establishment(id: EstablishmentId, usage: BuildingUse) -> Option<Self> {
        let style = mix64(id.0 ^ STYLE_STREAM);
        Some(Self {
            emblem: TradeEmblem::for_use(usage),
            name: ShopName::for_establishment(id, usage)?,
            mount: if style & 1 == 0 {
                SignMount::Wall
            } else {
                SignMount::Projecting
            },
            font: SignFont::GrenzeGotisch,
            finish: if style & 2 == 0 {
                SignFinish::PalePaint
            } else {
                SignFinish::DarkWood
            },
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SignBoard {
    pub centre: Vec3,
    pub rotation: Quat,
    pub size: Vec2,
}

#[cfg(test)]
mod tests;
