//! Canonical settlement economy inference shared by imports, services and city demand.
use crate::*;

/// Versioned, immutable settlement economy computed at world-build time.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum ProsperityTier {
    Subsistence,
    Modest,
    Comfortable,
    Prosperous,
    Wealthy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum SettlementService {
    GeneralStore,
    Inn,
    GeneralBlacksmith,
    Market,
    Weaponsmith,
    Armorer,
    Tailor,
    Herbalist,
    Temple,
    Bookstore,
}

/// A public settlement service that can authorize a rest or downtime action.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum SettlementActionService {
    Inn,
    Temple,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum StockCategory {
    Grain,
    Dairy,
    Meat,
    Fish,
    Cloth,
    Hides,
    Timber,
    Fuel,
    Stone,
    Pottery,
    Salt,
    Metalwares,
    Weapons,
    Armor,
    Herbs,
    GeneralGoods,
    Books,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum ProfileFactProvenance {
    ImportedEvidence,
    DerivedFromCanonicalEvidence,
    DeterministicGapFill,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub struct SettlementStock {
    pub category: StockCategory,
    /// Stable 1..=5 relative abundance, not mutable shop quantity.
    pub abundance: u8,
    pub provenance: ProfileFactProvenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub struct SettlementEconomyProfile {
    pub rules_version: u32,
    pub prosperity_score: u16,
    pub prosperity_tier: ProsperityTier,
    pub services: Vec<SettlementService>,
    pub specializations: Vec<StockCategory>,
    pub stock: Vec<SettlementStock>,
}

impl SettlementEconomyProfile {
    pub const MAX_SERVICES: usize = 10;
    pub const MAX_SPECIALIZATIONS: usize = 8;
    pub const MAX_STOCK: usize = 17;

    pub fn stage_placeholder() -> Self {
        Self {
            rules_version: CURRENT_INFERENCE_RULES_VERSION,
            prosperity_score: 0,
            prosperity_tier: ProsperityTier::Subsistence,
            services: vec![SettlementService::Inn],
            specializations: vec![],
            stock: vec![SettlementStock {
                category: StockCategory::GeneralGoods,
                abundance: 1,
                provenance: ProfileFactProvenance::DeterministicGapFill,
            }],
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.rules_version != CURRENT_INFERENCE_RULES_VERSION || self.prosperity_score > 1_000 {
            return Err("settlement economy has unsupported rules or prosperity".into());
        }
        if self.services.is_empty()
            || self.services.len() > Self::MAX_SERVICES
            || self.specializations.len() > Self::MAX_SPECIALIZATIONS
            || self.stock.is_empty()
            || self.stock.len() > Self::MAX_STOCK
            || self.services.windows(2).any(|v| v[0] >= v[1])
            || self.specializations.windows(2).any(|v| v[0] >= v[1])
            || self
                .stock
                .windows(2)
                .any(|v| v[0].category >= v[1].category)
            || self.stock.iter().any(|v| !(1..=5).contains(&v.abundance))
        {
            return Err("settlement economy collections are not bounded canonical sets".into());
        }
        Ok(())
    }

    pub fn has_service(&self, service: SettlementService) -> bool {
        self.services.binary_search(&service).is_ok()
    }
}

/// Single canonical economy projection used by import and database
/// finalization. `documented_town` is direct Viabundus evidence; route count is
/// taken from the finalized graph.
pub fn infer_settlement_economy(
    population_level: i32,
    population: u32,
    routes: u16,
    documented_town: bool,
    industries: &InferredIndustryProfile,
) -> Result<SettlementEconomyProfile, String> {
    let industrial = industries
        .outputs()
        .iter()
        .map(|e| match e {
            IndustryEvidence::Derived(v) => match v.scale() {
                ProductionScale::Marginal => 12,
                ProductionScale::Local => 28,
                ProductionScale::Regional => 48,
            },
            IndustryEvidence::Fallback(_) => 5,
        })
        .sum::<u16>()
        .min(260);
    let population_points = ((population.max(1) as f64).log10() * 95.0) as u16;
    let score = (u16::try_from(population_level.max(1)).unwrap_or(1) * 85)
        .saturating_add(population_points)
        .saturating_add(industrial)
        .saturating_add(routes.min(8) * 18)
        .saturating_add(u16::from(documented_town) * 55)
        .min(1_000);
    let tier = match score {
        0..=249 => ProsperityTier::Subsistence,
        250..=419 => ProsperityTier::Modest,
        420..=599 => ProsperityTier::Comfortable,
        600..=779 => ProsperityTier::Prosperous,
        _ => ProsperityTier::Wealthy,
    };
    let services = infer_services(population_level, population, score, industries);
    let stock = infer_stock(population_level, score, industries, &services);
    let mut specializations = stock
        .values()
        .filter(|value| value.abundance >= 4)
        .map(|value| (value.category, value.abundance))
        .collect::<Vec<_>>();
    specializations.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    specializations.truncate(SettlementEconomyProfile::MAX_SPECIALIZATIONS);
    specializations.sort_by_key(|(category, _)| *category);
    let profile = SettlementEconomyProfile {
        rules_version: CURRENT_INFERENCE_RULES_VERSION,
        prosperity_score: score,
        prosperity_tier: tier,
        services: services.into_iter().collect(),
        specializations: specializations
            .into_iter()
            .map(|(category, _)| category)
            .collect(),
        stock: stock.into_values().collect(),
    };
    profile.validate()?;
    Ok(profile)
}

fn forest_or_peat(e: &IndustryEvidence) -> bool {
    matches!(
        e,
        IndustryEvidence::Derived(DerivedIndustry::Forestry(_) | DerivedIndustry::PeatCutting(_))
    )
}

fn infer_services(
    population_level: i32,
    population: u32,
    score: u16,
    industries: &InferredIndustryProfile,
) -> std::collections::BTreeSet<SettlementService> {
    use std::collections::BTreeSet;
    let mut services = BTreeSet::from([SettlementService::Inn]);
    if population_level <= 1 && population < 250 {
        services.insert(SettlementService::GeneralStore);
    } else {
        services.extend([
            SettlementService::GeneralStore,
            SettlementService::Market,
            SettlementService::Temple,
        ]);
        if population_level <= 2 || score < 470 {
            services.insert(SettlementService::GeneralBlacksmith);
        } else {
            services.extend([SettlementService::Weaponsmith, SettlementService::Armorer]);
        }
        if population_level >= 3 || score >= 500 {
            services.insert(SettlementService::Tailor);
        }
        if population_level >= 3 || industries.outputs().iter().any(forest_or_peat) {
            services.insert(SettlementService::Herbalist);
        }
        if population_level >= 4 {
            services.insert(SettlementService::Bookstore);
        }
    }
    services
}

fn infer_stock(
    population_level: i32,
    score: u16,
    industries: &InferredIndustryProfile,
    services: &std::collections::BTreeSet<SettlementService>,
) -> std::collections::BTreeMap<StockCategory, SettlementStock> {
    use std::collections::BTreeMap;
    let mut stock = BTreeMap::<StockCategory, SettlementStock>::new();
    let mut add = |category, abundance, provenance| {
        stock
            .entry(category)
            .and_modify(|v| v.abundance = v.abundance.max(abundance))
            .or_insert(SettlementStock {
                category,
                abundance,
                provenance,
            });
    };
    for evidence in industries.outputs() {
        let IndustryEvidence::Derived(industry) = evidence else {
            continue;
        };
        let abundance = match industry.scale() {
            ProductionScale::Marginal => 2,
            ProductionScale::Local => 4,
            ProductionScale::Regional => 5,
        };
        let category = industry_stock_category(industry);
        add(
            category,
            abundance,
            ProfileFactProvenance::DerivedFromCanonicalEvidence,
        );
    }
    let gap = ProfileFactProvenance::DeterministicGapFill;
    add(
        StockCategory::GeneralGoods,
        if population_level <= 1 { 3 } else { 2 },
        gap,
    );
    if population_level <= 1 {
        add(StockCategory::Grain, 1, gap);
        add(StockCategory::Meat, 1, gap);
    }
    if industries.outputs().iter().any(|e|matches!(e,IndustryEvidence::Derived(DerivedIndustry::Agriculture(v)) if matches!(v.commodity,AgriculturalCommodity::Dairy|AgriculturalCommodity::Hides))){add(StockCategory::Meat,3,ProfileFactProvenance::DerivedFromCanonicalEvidence);}
    if services.contains(&SettlementService::GeneralBlacksmith)
        || services.contains(&SettlementService::Weaponsmith)
    {
        add(
            StockCategory::Metalwares,
            if score >= 600 { 4 } else { 2 },
            gap,
        );
    }
    if services.contains(&SettlementService::GeneralBlacksmith) {
        add(StockCategory::Weapons, 1, gap);
        add(StockCategory::Armor, 1, gap);
    }
    if services.contains(&SettlementService::Weaponsmith) {
        add(
            StockCategory::Weapons,
            if score >= 700 { 4 } else { 2 },
            gap,
        );
    }
    if services.contains(&SettlementService::Armorer) {
        add(StockCategory::Armor, if score >= 700 { 4 } else { 2 }, gap);
    }
    if services.contains(&SettlementService::Herbalist) {
        add(
            StockCategory::Herbs,
            2 + u8::from(industries.outputs().iter().any(forest_or_peat)),
            gap,
        );
    }
    if services.contains(&SettlementService::Bookstore) {
        add(
            StockCategory::Books,
            if population_level >= 5 { 4 } else { 2 },
            gap,
        );
    }
    stock
}

fn industry_stock_category(industry: &DerivedIndustry) -> StockCategory {
    match industry {
        DerivedIndustry::Agriculture(v) => match v.commodity {
            AgriculturalCommodity::Grain => StockCategory::Grain,
            AgriculturalCommodity::Flax | AgriculturalCommodity::Wool => StockCategory::Cloth,
            AgriculturalCommodity::Dairy => StockCategory::Dairy,
            AgriculturalCommodity::Hides => StockCategory::Hides,
        },
        DerivedIndustry::Fishing(_) => StockCategory::Fish,
        DerivedIndustry::Quarrying(_) => StockCategory::Stone,
        DerivedIndustry::Mining(_) => StockCategory::Fuel,
        DerivedIndustry::Pottery(_) => StockCategory::Pottery,
        DerivedIndustry::PeatCutting(_) | DerivedIndustry::CharcoalBurning(_) => {
            StockCategory::Fuel
        }
        DerivedIndustry::Forestry(v) => match v.commodity {
            ForestCommodity::Fuelwood => StockCategory::Fuel,
            _ => StockCategory::Timber,
        },
        DerivedIndustry::Saltmaking(_) => StockCategory::Salt,
        DerivedIndustry::Construction(v) => match v.commodity {
            ConstructionCommodity::Timber => StockCategory::Timber,
            ConstructionCommodity::Brick | ConstructionCommodity::RoofTile => {
                StockCategory::Pottery
            }
            _ => StockCategory::Stone,
        },
    }
}
