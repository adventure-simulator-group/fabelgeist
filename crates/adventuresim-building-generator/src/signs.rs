//! Establishment signs with period tool pictograms and operator-projected brands.
use adventuresim_world_schema::{
    person_names::RenderedPersonalName, settlement_buildings::BuildingUse,
};
use bevy::math::{Quat, Vec2, Vec3};
use clap::ValueEnum;
use fabelgeist_determinism::StreamId;
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

const SIGN_MOUNT: StreamId = StreamId::new("sign.mount");
const SIGN_FINISH: StreamId = StreamId::new("sign.finish");
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
    /// Brand a business from the caller-supplied validated operator projection.
    pub fn for_operator(operator: &RenderedPersonalName, usage: BuildingUse) -> Option<Self> {
        let trade = shop_trade(usage)?;
        Some(Self {
            proprietor: format!("{operator}’s"),
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
    pub fn for_operator(
        id: EstablishmentId,
        operator: &RenderedPersonalName,
        usage: BuildingUse,
    ) -> Option<Self> {
        Self::for_establishment(id, usage, ShopName::for_operator(operator, usage)?)
    }

    /// Build a sign from a caller-supplied business brand whose trade matches
    /// the establishment use.
    pub fn for_establishment(
        id: EstablishmentId,
        usage: BuildingUse,
        name: ShopName,
    ) -> Option<Self> {
        if name.trade != shop_trade(usage)? {
            return None;
        }
        Some(Self {
            emblem: TradeEmblem::for_use(usage),
            name,
            mount: if SIGN_MOUNT.rng(id.0, &[]).boolean() {
                SignMount::Wall
            } else {
                SignMount::Projecting
            },
            font: SignFont::GrenzeGotisch,
            finish: if SIGN_FINISH.rng(id.0, &[]).boolean() {
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
