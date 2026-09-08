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
                .any(|b| b.usage == world::settlement_buildings::BuildingUse::Weaponsmith)
        );
        assert!(
            plan.buildings
                .iter()
                .any(|b| b.usage == world::settlement_buildings::BuildingUse::TimberYard)
        );
        assert!(
            !plan
                .buildings
                .iter()
                .any(|b| b.usage == world::settlement_buildings::BuildingUse::Armorer)
        );
    }
}
