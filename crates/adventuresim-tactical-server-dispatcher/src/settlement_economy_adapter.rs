//! Project the generated SDK wire types into the canonical shared economy types.
use adventuresim_stdb_client as sdk;
use adventuresim_world_schema as world;

pub fn economy_profile(source: &sdk::SettlementEconomyProfile) -> world::SettlementEconomyProfile {
    world::SettlementEconomyProfile {
        rules_version: source.rules_version,
        prosperity_score: source.prosperity_score,
        prosperity_tier: match source.prosperity_tier {
            sdk::ProsperityTier::Subsistence => world::ProsperityTier::Subsistence,
            sdk::ProsperityTier::Modest => world::ProsperityTier::Modest,
            sdk::ProsperityTier::Comfortable => world::ProsperityTier::Comfortable,
            sdk::ProsperityTier::Prosperous => world::ProsperityTier::Prosperous,
            sdk::ProsperityTier::Wealthy => world::ProsperityTier::Wealthy,
        },
        services: source.services.iter().map(service).collect(),
        specializations: source.specializations.iter().map(stock_category).collect(),
        stock: source
            .stock
            .iter()
            .map(|stock| world::SettlementStock {
                category: stock_category(&stock.category),
                abundance: stock.abundance,
                provenance: match stock.provenance {
                    sdk::ProfileFactProvenance::ImportedEvidence => {
                        world::ProfileFactProvenance::ImportedEvidence
                    }
                    sdk::ProfileFactProvenance::DerivedFromCanonicalEvidence => {
                        world::ProfileFactProvenance::DerivedFromCanonicalEvidence
                    }
                    sdk::ProfileFactProvenance::DeterministicGapFill => {
                        world::ProfileFactProvenance::DeterministicGapFill
                    }
                },
            })
            .collect(),
    }
}

pub fn building_use(source: &sdk::BuildingUse) -> world::settlement_buildings::BuildingUse {
    use sdk::BuildingUse as S;
    use world::settlement_buildings::BuildingUse as W;
    match source {
        S::Dwelling => W::Dwelling,
        S::ParishChurch => W::ParishChurch,
        S::Inn => W::Inn,
        S::GeneralShop => W::GeneralShop,
        S::MarketHall => W::MarketHall,
        S::Smithy => W::Smithy,
        S::Weaponsmith => W::Weaponsmith,
        S::Armorer => W::Armorer,
        S::Tailor => W::Tailor,
        S::Herbalist => W::Herbalist,
        S::Bookshop => W::Bookshop,
        S::Bakehouse => W::Bakehouse,
        S::Brewery => W::Brewery,
        S::Malthouse => W::Malthouse,
        S::Butcher => W::Butcher,
        S::Stable => W::Stable,
        S::Barn => W::Barn,
        S::Granary => W::Granary,
        S::HorseMill => W::HorseMill,
        S::Cooper => W::Cooper,
        S::Carpenter => W::Carpenter,
        S::Wheelwright => W::Wheelwright,
        S::Cobbler => W::Cobbler,
        S::Weaver => W::Weaver,
        S::Tannery => W::Tannery,
        S::Dyer => W::Dyer,
        S::Ropemaker => W::Ropemaker,
        S::Chandler => W::Chandler,
        S::Potter => W::Potter,
        S::Stonecutter => W::Stonecutter,
        S::TimberYard => W::TimberYard,
        S::Warehouse => W::Warehouse,
        S::WoadStore => W::WoadStore,
        S::Fishmonger => W::Fishmonger,
        S::TownHall => W::TownHall,
        S::WeighHouse => W::WeighHouse,
        S::Guildhall => W::Guildhall,
        S::Hospital => W::Hospital,
        S::Bathhouse => W::Bathhouse,
        S::School => W::School,
        S::Apothecary => W::Apothecary,
        S::PrintingHouse => W::PrintingHouse,
        S::Guardhouse => W::Guardhouse,
        S::Prison => W::Prison,
        S::Rectory => W::Rectory,
        S::WaterMill => W::WaterMill,
        S::FullingMill => W::FullingMill,
        S::PaperMill => W::PaperMill,
        S::Sawmill => W::Sawmill,
        S::Windmill => W::Windmill,
        S::Cathedral => W::Cathedral,
        S::Monastery => W::Monastery,
        S::University => W::University,
        S::Mint => W::Mint,
        S::CustomsHouse => W::CustomsHouse,
        S::Synagogue => W::Synagogue,
        S::Chapel => W::Chapel,
        S::Manor => W::Manor,
        S::Castle => W::Castle,
        S::Arsenal => W::Arsenal,
        S::SaltWorks => W::SaltWorks,
        S::Smelter => W::Smelter,
        S::AssayHouse => W::AssayHouse,
        S::Brickworks => W::Brickworks,
        S::Glassworks => W::Glassworks,
        S::ExecutionerHouse => W::ExecutionerHouse,
    }
}

fn service(source: &sdk::SettlementService) -> world::SettlementService {
    match source {
        sdk::SettlementService::GeneralStore => world::SettlementService::GeneralStore,
        sdk::SettlementService::Inn => world::SettlementService::Inn,
        sdk::SettlementService::GeneralBlacksmith => world::SettlementService::GeneralBlacksmith,
        sdk::SettlementService::Market => world::SettlementService::Market,
        sdk::SettlementService::Weaponsmith => world::SettlementService::Weaponsmith,
        sdk::SettlementService::Armorer => world::SettlementService::Armorer,
        sdk::SettlementService::Tailor => world::SettlementService::Tailor,
        sdk::SettlementService::Herbalist => world::SettlementService::Herbalist,
        sdk::SettlementService::Temple => world::SettlementService::Temple,
        sdk::SettlementService::Bookstore => world::SettlementService::Bookstore,
    }
}

fn stock_category(source: &sdk::StockCategory) -> world::StockCategory {
    match source {
        sdk::StockCategory::Grain => world::StockCategory::Grain,
        sdk::StockCategory::Dairy => world::StockCategory::Dairy,
        sdk::StockCategory::Meat => world::StockCategory::Meat,
        sdk::StockCategory::Fish => world::StockCategory::Fish,
        sdk::StockCategory::Cloth => world::StockCategory::Cloth,
        sdk::StockCategory::Hides => world::StockCategory::Hides,
        sdk::StockCategory::Timber => world::StockCategory::Timber,
        sdk::StockCategory::Fuel => world::StockCategory::Fuel,
        sdk::StockCategory::Stone => world::StockCategory::Stone,
        sdk::StockCategory::Pottery => world::StockCategory::Pottery,
        sdk::StockCategory::Salt => world::StockCategory::Salt,
        sdk::StockCategory::Metalwares => world::StockCategory::Metalwares,
        sdk::StockCategory::Weapons => world::StockCategory::Weapons,
        sdk::StockCategory::Armor => world::StockCategory::Armor,
        sdk::StockCategory::Herbs => world::StockCategory::Herbs,
        sdk::StockCategory::GeneralGoods => world::StockCategory::GeneralGoods,
        sdk::StockCategory::Books => world::StockCategory::Books,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_preserves_canonical_service_availability_and_specializations() {
        let source = sdk::SettlementEconomyProfile {
            rules_version: world::CURRENT_INFERENCE_RULES_VERSION,
            prosperity_score: 650,
            prosperity_tier: sdk::ProsperityTier::Prosperous,
            services: vec![
                sdk::SettlementService::Inn,
                sdk::SettlementService::Weaponsmith,
            ],
            specializations: vec![sdk::StockCategory::Timber],
            stock: vec![sdk::SettlementStock {
                category: sdk::StockCategory::Timber,
                abundance: 4,
                provenance: sdk::ProfileFactProvenance::DerivedFromCanonicalEvidence,
            }],
        };
        let economy = economy_profile(&source);
        economy.validate().unwrap();
        let plan = world::settlement_buildings::SettlementBuildingDemand::new(42, 6_500, &economy);
        assert!(
            plan.buildings
                .iter()
                .any(|b| b.usage() == world::settlement_buildings::BuildingUse::Weaponsmith)
        );
        assert!(
            plan.buildings
                .iter()
                .any(|b| b.usage() == world::settlement_buildings::BuildingUse::TimberYard)
        );
        assert!(
            !plan
                .buildings
                .iter()
                .any(|b| b.usage() == world::settlement_buildings::BuildingUse::Armorer)
        );
    }
}
