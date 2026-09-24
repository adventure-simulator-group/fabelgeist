//! Deterministic resident planning before any persistent character facts exist.
use super::*;
use adventuresim_core::reputation::effective_population;
use adventuresim_world_schema::settlement_buildings::{
    BuildingDistrict, BuildingUse, SettlementBuildingDemand,
};

#[derive(Clone)]
pub(super) struct ResidentDraft {
    pub(super) character_id: u64,
    pub(super) seed: String,
    pub(super) location: String,
    pub(super) service: Option<String>,
    pub(super) profession: String,
    pub(super) role: String,
    pub(super) is_default: bool,
    pub(super) business_id: Option<BusinessId>,
    pub(super) sex: Sex,
    pub(super) presentation: Presentation,
    pub(super) exact_age: Option<u16>,
}

#[derive(Clone, Copy)]
enum DefaultPresence {
    Default,
    Additional,
}

impl DefaultPresence {
    const fn is_default(self) -> bool {
        matches!(self, Self::Default)
    }
}

#[derive(Clone, Copy)]
enum StrategicServiceEndpoint {
    None,
    Service(&'static str),
}

impl StrategicServiceEndpoint {
    const fn value(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Service(value) => Some(value),
        }
    }
}

impl ResidentDraft {
    pub(super) fn character_id(&self) -> u64 {
        self.character_id
    }

    fn new(
        seed: String,
        location: impl Into<String>,
        service: StrategicServiceEndpoint,
        profession: impl Into<String>,
        role: impl Into<String>,
        presence: DefaultPresence,
        business_id: Option<BusinessId>,
    ) -> Self {
        let sex = if resident_random(&seed, ResidentEntropyStream::Sex).boolean() {
            Sex::Female
        } else {
            Sex::Male
        };
        Self {
            character_id: resident_character_id(&seed),
            seed,
            location: location.into(),
            service: service.value().map(str::to_owned),
            profession: profession.into(),
            role: role.into(),
            is_default: presence.is_default(),
            business_id,
            sex,
            presentation: match sex {
                Sex::Female => Presentation::Woman,
                Sex::Male => Presentation::Man,
            },
            exact_age: None,
        }
    }

    fn resident(
        seed: String,
        location: impl Into<String>,
        profession: impl Into<String>,
        role: impl Into<String>,
        presence: DefaultPresence,
    ) -> Self {
        Self::new(
            seed,
            location,
            StrategicServiceEndpoint::None,
            profession,
            role,
            presence,
            None,
        )
    }

    fn service_provider(
        seed: String,
        location: impl Into<String>,
        service: &'static str,
        profession: impl Into<String>,
        role: impl Into<String>,
        presence: DefaultPresence,
    ) -> Self {
        Self::new(
            seed,
            location,
            StrategicServiceEndpoint::Service(service),
            profession,
            role,
            presence,
            None,
        )
    }

    fn business_operator(
        seed: String,
        location: impl Into<String>,
        service: StrategicServiceEndpoint,
        profession: impl Into<String>,
        role: impl Into<String>,
        presence: DefaultPresence,
        business_id: BusinessId,
    ) -> Self {
        Self::new(
            seed,
            location,
            service,
            profession,
            role,
            presence,
            Some(business_id),
        )
    }
}

pub(super) fn canonical_service(
    usage: BuildingUse,
    ordinal: u32,
) -> Option<(&'static str, &'static str)> {
    if ordinal != 0 {
        return None;
    }
    Some(match usage {
        BuildingUse::GeneralShop => ("merchants", "market"),
        BuildingUse::Weaponsmith => ("weapons", "forge"),
        BuildingUse::Armorer => ("armor", "armoury"),
        BuildingUse::Tailor => ("clothing", "tailor"),
        BuildingUse::Herbalist => ("herbalist", "herbalist"),
        BuildingUse::Inn => ("inn", "inn"),
        BuildingUse::Bookshop => ("books", "bookstore"),
        _ => return None,
    })
}

fn business_location(usage: BuildingUse, ordinal: u32) -> &'static str {
    canonical_service(usage, ordinal)
        .map(|(_, location)| location)
        .unwrap_or(match usage.definition().district {
            BuildingDistrict::Market => "market",
            BuildingDistrict::Neighbourhood => "residences",
            BuildingDistrict::Craft | BuildingDistrict::Edge => "overview",
        })
}

fn business_seed(business_id: &BusinessId) -> Result<String, String> {
    serde_json::to_string(business_id)
        .map(|key| format!("resident:business:{key}"))
        .map_err(|error| format!("Could not serialize business identity: {error}"))
}

fn business_profession(usage: BuildingUse) -> &'static str {
    match usage {
        BuildingUse::GeneralShop => "merchant",
        BuildingUse::Weaponsmith => "weaponsmith",
        BuildingUse::Armorer => "armourer",
        BuildingUse::Tailor => "tailor",
        BuildingUse::Herbalist => "herbalist",
        BuildingUse::Inn => "innkeeper",
        BuildingUse::Bookshop => "bookseller",
        _ => usage.definition().label,
    }
}

fn business_role(usage: BuildingUse) -> String {
    match usage {
        BuildingUse::GeneralShop => "market steward".into(),
        BuildingUse::Weaponsmith => "master weaponsmith".into(),
        BuildingUse::Armorer => "master armourer".into(),
        BuildingUse::Tailor => "master tailor".into(),
        BuildingUse::Herbalist => "local healer".into(),
        BuildingUse::Inn => "innkeeper".into(),
        BuildingUse::Bookshop => "bookseller".into(),
        _ => format!("{} operator", usage.definition().label.to_lowercase()),
    }
}

pub(super) fn finalize_household_demographics(drafts: &mut [ResidentDraft]) -> Vec<Vec<u64>> {
    let (families, _) = drafts.as_chunks_mut::<4>();
    for family in families {
        for (draft, (sex, presentation, age)) in family.iter_mut().zip([
            (Sex::Male, Presentation::Man, 52),
            (Sex::Female, Presentation::Woman, 48),
            (Sex::Female, Presentation::Woman, 24),
            (Sex::Male, Presentation::Man, 21),
        ]) {
            draft.sex = sex;
            draft.presentation = presentation;
            draft.exact_age = Some(age);
        }
    }
    drafts
        .chunks(4)
        .map(|family| family.iter().map(ResidentDraft::character_id).collect())
        .collect()
}

fn business_drafts(
    settlement_id: &str,
    demand: SettlementBuildingDemand,
) -> Result<(Vec<ResidentDraft>, Vec<Vec<u64>>), String> {
    let mut drafts = demand
        .buildings
        .into_iter()
        .filter_map(|demand| demand.business_key())
        .map(|key| {
            let business_id = BusinessId::new(settlement_id, key);
            let (service, location, presence) = canonical_service(key.usage, key.ordinal).map_or(
                (
                    StrategicServiceEndpoint::None,
                    business_location(key.usage, key.ordinal),
                    DefaultPresence::Additional,
                ),
                |(service, location)| {
                    (
                        StrategicServiceEndpoint::Service(service),
                        location,
                        DefaultPresence::Default,
                    )
                },
            );
            Ok(ResidentDraft::business_operator(
                business_seed(&business_id)?,
                location,
                service,
                business_profession(key.usage),
                business_role(key.usage),
                presence,
                business_id,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut households = Vec::new();
    let mut start = 0;
    while start < drafts.len() {
        let usage = drafts[start].business_id.as_ref().unwrap().key.usage;
        let end = drafts[start..]
            .iter()
            .position(|draft| draft.business_id.as_ref().unwrap().key.usage != usage)
            .map_or(drafts.len(), |offset| start + offset);
        households.extend(finalize_household_demographics(&mut drafts[start..end]));
        start = end;
    }
    Ok((drafts, households))
}

fn ambient_drafts(settlement_id: &str, urban: bool) -> Vec<ResidentDraft> {
    let mut drafts = [
        "market",
        "forge",
        "armoury",
        "tailor",
        "herbalist",
        "inn",
        "bookstore",
    ]
    .into_iter()
    .map(|location| {
        ResidentDraft::resident(
            resident_seed(settlement_id, location, 1),
            location,
            "local resident",
            "customer or visitor",
            DefaultPresence::Additional,
        )
    })
    .collect::<Vec<_>>();
    drafts.extend([
        ResidentDraft::service_provider(
            resident_seed(settlement_id, "church", 0),
            "church",
            "religion",
            "cleric",
            "parish priest",
            DefaultPresence::Default,
        ),
        ResidentDraft::resident(
            resident_seed(settlement_id, "church", 1),
            "church",
            "local resident",
            "customer or visitor",
            DefaultPresence::Additional,
        ),
    ]);
    for (ordinal, profession, role) in [
        (0, "laborer", "neighbor"),
        (1, "householder", "household representative"),
        (2, "artisan", "local resident"),
    ] {
        drafts.push(ResidentDraft::resident(
            resident_seed(settlement_id, "overview", ordinal),
            "overview",
            profession,
            role,
            if ordinal == 0 {
                DefaultPresence::Default
            } else {
                DefaultPresence::Additional
            },
        ));
    }
    for (ordinal, profession, role, presence) in [
        (0, "householder", "resident", DefaultPresence::Default),
        (
            1,
            "domestic worker",
            "neighbor",
            DefaultPresence::Additional,
        ),
    ] {
        drafts.push(ResidentDraft::resident(
            resident_seed(settlement_id, "residences", ordinal),
            "residences",
            profession,
            role,
            presence,
        ));
    }
    if urban {
        for (ordinal, profession, role, presence) in [
            (0, "retainer", "reeve", DefaultPresence::Default),
            (1, "servant", "keep servant", DefaultPresence::Additional),
        ] {
            drafts.push(ResidentDraft::resident(
                resident_seed(settlement_id, "keep", ordinal),
                "keep",
                profession,
                role,
                presence,
            ));
        }
    }
    drafts
}

fn organization_drafts(
    settlement_id: &str,
    economy: &adventuresim_world_schema::SettlementEconomyProfile,
) -> Vec<ResidentDraft> {
    adventuresim_core::organization::organizations_for_chapter(settlement_id)
        .map(|organization| {
            let chapter = organization.chapter(settlement_id).unwrap();
            let location = adventuresim_core::organization::chapter_effective_location_id(
                organization,
                chapter,
                economy,
            );
            let mut draft = ResidentDraft::service_provider(
                organization_representative_seed(settlement_id, &organization.id),
                location,
                "organization",
                &chapter.representative_profession,
                &chapter.representative_title,
                if location == chapter.location_id {
                    DefaultPresence::Default
                } else {
                    DefaultPresence::Additional
                },
            );
            draft.character_id = adventuresim_core::organization::organization_representative_id(
                settlement_id,
                &organization.id,
            );
            draft
        })
        .collect()
}

fn validate(drafts: &[ResidentDraft]) -> Result<(), String> {
    let mut character_ids = BTreeSet::new();
    let mut business_ids = BTreeSet::new();
    for draft in drafts {
        if !character_ids.insert(draft.character_id()) {
            return Err("Settlement resident drafts contain a duplicate character identity".into());
        }
        if let Some(id) = &draft.business_id
            && !business_ids.insert(id)
        {
            return Err("Settlement resident drafts contain a duplicate business identity".into());
        }
    }
    Ok(())
}

pub(super) fn settlement_resident_drafts(
    ctx: &ReducerContext,
    settlement_id: &str,
) -> Result<(Vec<ResidentDraft>, Vec<Vec<u64>>), String> {
    let settlement = ctx
        .db
        .settlement()
        .id()
        .find(settlement_id.to_owned())
        .ok_or("Settlement population references an unknown settlement")?;
    let demand = SettlementBuildingDemand::new(
        population::settlement_building_seed(settlement_id),
        effective_population(settlement.population_level, settlement.population_estimate),
        &settlement.economy,
    );
    let (mut drafts, mut households) = business_drafts(settlement_id, demand)?;
    let mut ambient = ambient_drafts(
        settlement_id,
        matches!(
            settlement.category,
            crate::strategic::SettlementCategory::Town
                | crate::strategic::SettlementCategory::City
                | crate::strategic::SettlementCategory::Capital
        ),
    );
    households.extend(finalize_household_demographics(&mut ambient));
    drafts.extend(ambient);
    let organizations = organization_drafts(settlement_id, &settlement.economy);
    households.extend(organizations.iter().map(|draft| vec![draft.character_id()]));
    drafts.extend(organizations);
    validate(&drafts)?;
    Ok((drafts, households))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_family_demographics_are_finalized_before_unrelated_drafts_are_added() {
        let make = |ordinal| {
            ResidentDraft::resident(
                resident_seed("family-test", "overview", ordinal),
                "overview",
                "resident",
                "resident",
                DefaultPresence::Additional,
            )
        };
        let mut drafts = (0..4).map(make).collect::<Vec<_>>();
        let households = finalize_household_demographics(&mut drafts);
        let finalized = drafts
            .iter()
            .map(|draft| {
                (
                    draft.character_id(),
                    draft.sex,
                    draft.presentation,
                    draft.exact_age,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            households,
            vec![finalized.iter().map(|row| row.0).collect::<Vec<_>>()]
        );
        assert_eq!(
            finalized
                .iter()
                .map(|row| (row.1, row.2, row.3))
                .collect::<Vec<_>>(),
            vec![
                (Sex::Male, Presentation::Man, Some(52)),
                (Sex::Female, Presentation::Woman, Some(48)),
                (Sex::Female, Presentation::Woman, Some(24)),
                (Sex::Male, Presentation::Man, Some(21)),
            ]
        );

        drafts.push(make(4));
        finalize_household_demographics(&mut drafts);
        assert_eq!(
            drafts[..4]
                .iter()
                .map(|draft| (
                    draft.character_id(),
                    draft.sex,
                    draft.presentation,
                    draft.exact_age
                ))
                .collect::<Vec<_>>(),
            finalized
        );
        assert!(drafts[4].exact_age.is_none());
    }

    #[test]
    fn only_ordinal_zero_businesses_own_legacy_service_endpoints() {
        for usage in [
            BuildingUse::GeneralShop,
            BuildingUse::Weaponsmith,
            BuildingUse::Armorer,
            BuildingUse::Tailor,
            BuildingUse::Herbalist,
            BuildingUse::Inn,
            BuildingUse::Bookshop,
        ] {
            assert!(canonical_service(usage, 0).is_some());
            assert!(canonical_service(usage, 1).is_none());
        }
        assert!(canonical_service(BuildingUse::ParishChurch, 0).is_none());
    }
}
