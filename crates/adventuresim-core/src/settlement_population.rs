//! Canonical deterministic weighted relations for persistent settlement residents.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod age;
pub use age::AgeBand;
mod demographics;
pub use demographics::settlement_building_seed;
mod error;
pub use error::PopulationError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceBridge {
    NearbyHome,
    HouseholdErrand,
    RetainerErrand,
}

impl PresenceBridge {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::NearbyHome => "nearby_home",
            Self::HouseholdErrand => "household_errand",
            Self::RetainerErrand => "retainer_errand",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profession {
    Artisan,
    Householder,
    Laborer,
    Retainer,
    ServiceProvider,
}

impl Profession {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Artisan => "artisan",
            Self::Householder => "householder",
            Self::Laborer => "laborer",
            Self::Retainer => "retainer",
            Self::ServiceProvider => "serviceprovider",
        }
    }

    pub const fn stable_variant_id(self) -> &'static str {
        match self {
            Self::Artisan => "Artisan",
            Self::Householder => "Householder",
            Self::Laborer => "Laborer",
            Self::Retainer => "Retainer",
            Self::ServiceProvider => "ServiceProvider",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Schedule {
    Provider,
    Day,
    Evening,
    Early,
}

impl Schedule {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Provider => "provider",
            Self::Day => "day",
            Self::Evening => "evening",
            Self::Early => "early",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationContext {
    Overview,
    Market,
    Forge,
    Armoury,
    Tailor,
    Herbalist,
    Inn,
    Church,
    Residences,
    Keep,
    Organization,
    AdultVenue,
}

impl LocationContext {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Market => "market",
            Self::Forge => "forge",
            Self::Armoury => "armoury",
            Self::Tailor => "tailor",
            Self::Herbalist => "herbalist",
            Self::Inn => "inn",
            Self::Church => "church",
            Self::Residences => "residences",
            Self::Keep => "keep",
            Self::Organization => "organization",
            Self::AdultVenue => "adultvenue",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationInput {
    pub seed: fabelgeist_determinism::SeedKey,
    pub location: LocationContext,
    pub role: PopulationRole,
    pub local_role: String,
    /// Final age selected by an owning population plan. `None` lets this
    /// relation choose the age before any age-dependent profile facts.
    pub age: Option<AgeBand>,
    pub available_bridges: BTreeSet<PresenceBridge>,
}

mod role;
pub use role::PopulationRole;
mod bridge;
pub use bridge::BridgeRequirement;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationCandidate<T> {
    pub value: T,
    pub plausibility: u32,
    pub curation: u32,
    pub bridge: BridgeRequirement,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationDecision {
    pub relation: String,
    pub context: String,
    pub decision: String,
    pub plausibility: u32,
    pub curation: u32,
    pub bridge: Option<PresenceBridge>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedPopulationProfile {
    pub age: AgeBand,
    pub profession: Profession,
    pub schedule: Schedule,
    pub height: String,
    pub build: String,
    pub hair: String,
    pub household_kind: String,
    pub decisions: Vec<RelationDecision>,
}

/// Stable identity seed for a persistent source coordinate.
pub fn stable_hash(value: &str) -> u64 {
    fabelgeist_determinism::Seed::derive(
        value.as_bytes(),
        fabelgeist_determinism::StreamId::new("population.source-identity"),
        &[],
    )
    .to_u64()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopulationRelation {
    AgeAtLocation,
    ProfessionAtLocation,
    ScheduleAtLocation,
    Height,
    BuildForProfession,
    HairForAge,
    HouseholdForAgeProfession,
}

impl PopulationRelation {
    const fn stable_id(self) -> &'static str {
        match self {
            Self::AgeAtLocation => "age_at_location",
            Self::ProfessionAtLocation => "profession_at_location",
            Self::ScheduleAtLocation => "schedule_at_location",
            Self::Height => "height",
            Self::BuildForProfession => "build_for_profession",
            Self::HairForAge => "hair_for_age",
            Self::HouseholdForAgeProfession => "household_for_age_profession",
        }
    }
}

trait StableDecision: Copy {
    fn stable_decision(self) -> &'static str;
}

impl StableDecision for AgeBand {
    fn stable_decision(self) -> &'static str {
        self.stable_id()
    }
}

impl StableDecision for Profession {
    fn stable_decision(self) -> &'static str {
        self.stable_id()
    }
}

impl StableDecision for Schedule {
    fn stable_decision(self) -> &'static str {
        self.stable_id()
    }
}

impl StableDecision for &'static str {
    fn stable_decision(self) -> &'static str {
        self
    }
}

fn population_relation_seed(
    seed: &fabelgeist_determinism::SeedKey,
    relation: PopulationRelation,
) -> fabelgeist_determinism::Seed {
    seed.derive(
        fabelgeist_determinism::StreamId::new("population.relation"),
        &[relation.stable_id().as_bytes()],
    )
}

fn choose<T: StableDecision>(
    seed: &fabelgeist_determinism::SeedKey,
    relation: PopulationRelation,
    context: &str,
    available_bridges: &BTreeSet<PresenceBridge>,
    candidates: &[RelationCandidate<T>],
) -> Result<(T, RelationDecision), PopulationError> {
    let relation_id = relation.stable_id();
    let mut valid: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|candidate| {
            candidate.plausibility > 0
                && candidate.curation > 0
                && candidate.bridge.is_satisfied_by(available_bridges)
        })
        .collect();
    valid.sort_by_key(|candidate| candidate.value.stable_decision());
    if valid
        .windows(2)
        .any(|pair| pair[0].value.stable_decision() == pair[1].value.stable_decision())
    {
        return Err(PopulationError::DuplicateCandidate { relation });
    }
    let weights: Vec<_> = valid
        .iter()
        .map(|candidate| u64::from(candidate.plausibility) * u64::from(candidate.curation))
        .collect();
    let selected = population_relation_seed(seed, relation)
        .rng()
        .weighted_index(&weights)
        .map_err(|source| PopulationError::Sampling {
            relation,
            context: context.into(),
            source,
        })?;
    let candidate = valid[selected];
    Ok((
        candidate.value,
        RelationDecision {
            relation: relation_id.into(),
            context: context.into(),
            decision: candidate.value.stable_decision().into(),
            plausibility: candidate.plausibility,
            curation: candidate.curation,
            bridge: candidate.bridge.required_bridge(),
        },
    ))
}

fn candidate<T>(value: T, plausibility: u32) -> RelationCandidate<T> {
    RelationCandidate {
        value,
        plausibility,
        curation: 10,
        bridge: BridgeRequirement::Unrestricted,
    }
}
fn bridged<T>(value: T, plausibility: u32, bridge: PresenceBridge) -> RelationCandidate<T> {
    RelationCandidate {
        value,
        plausibility,
        curation: 10,
        bridge: BridgeRequirement::Required(bridge),
    }
}

pub fn generate(input: &GenerationInput) -> Result<GeneratedPopulationProfile, PopulationError> {
    let context = input.location.stable_id();
    let adult_only = input.role.is_provider() || input.location == LocationContext::AdultVenue;
    let age_candidates = [
        candidate(AgeBand::Adult, if adult_only { 85 } else { 62 }),
        candidate(AgeBand::Elder, if adult_only { 15 } else { 18 }),
        bridged(
            AgeBand::Adolescent,
            if input.role.is_provider() {
                0
            } else if input.location == LocationContext::AdultVenue {
                3
            } else {
                15
            },
            PresenceBridge::HouseholdErrand,
        ),
        bridged(
            AgeBand::Child,
            if input.role.is_provider() {
                0
            } else if input.location == LocationContext::AdultVenue {
                1
            } else {
                5
            },
            PresenceBridge::NearbyHome,
        ),
    ];
    let profession_candidates = [
        candidate(
            Profession::Artisan,
            if input.role.is_provider() {
                0
            } else if matches!(
                input.location,
                LocationContext::Forge
                    | LocationContext::Armoury
                    | LocationContext::Tailor
                    | LocationContext::Market
            ) {
                55
            } else {
                18
            },
        ),
        candidate(
            Profession::Householder,
            if input.role.is_provider() {
                0
            } else if matches!(
                input.location,
                LocationContext::Overview | LocationContext::Residences
            ) {
                45
            } else {
                20
            },
        ),
        candidate(
            Profession::Laborer,
            if input.role.is_provider() { 0 } else { 30 },
        ),
        if input.role.is_provider() {
            candidate(Profession::Retainer, 0)
        } else if input.location == LocationContext::Keep {
            candidate(Profession::Retainer, 70)
        } else {
            bridged(Profession::Retainer, 2, PresenceBridge::RetainerErrand)
        },
        candidate(
            Profession::ServiceProvider,
            if input.role.is_provider() { 100 } else { 0 },
        ),
    ];
    let (age, age_decision) = demographics::choose_age(input, context, &age_candidates)?;
    let (profession, mut profession_decision) = choose(
        &input.seed,
        PopulationRelation::ProfessionAtLocation,
        context,
        &input.available_bridges,
        &profession_candidates,
    )?;
    input
        .role
        .apply_profession_explanation(&mut profession_decision, &input.local_role);
    let schedule_candidates = if input.role.is_provider() {
        [
            candidate(Schedule::Provider, 100),
            candidate(Schedule::Day, 0),
            candidate(Schedule::Evening, 0),
            candidate(Schedule::Early, 0),
        ]
    } else {
        [
            candidate(Schedule::Provider, 0),
            candidate(
                Schedule::Day,
                if input.location == LocationContext::Inn {
                    35
                } else {
                    75
                },
            ),
            candidate(
                Schedule::Evening,
                if input.location == LocationContext::Inn {
                    65
                } else {
                    15
                },
            ),
            candidate(
                Schedule::Early,
                if matches!(profession, Profession::Laborer | Profession::Artisan) {
                    35
                } else {
                    10
                },
            ),
        ]
    };
    let (schedule, schedule_decision) = choose(
        &input.seed,
        PopulationRelation::ScheduleAtLocation,
        context,
        &input.available_bridges,
        &schedule_candidates,
    )?;
    let (height, height_decision) = choose(
        &input.seed,
        PopulationRelation::Height,
        "demographic",
        &input.available_bridges,
        &[
            candidate("short", 25),
            candidate("average height", 55),
            candidate("tall", 20),
        ],
    )?;
    let (build, build_decision) = choose(
        &input.seed,
        PopulationRelation::BuildForProfession,
        profession.stable_variant_id(),
        &input.available_bridges,
        &[
            candidate("slender", 30),
            candidate(
                "sturdy",
                if matches!(
                    profession,
                    Profession::Artisan | Profession::Laborer | Profession::ServiceProvider
                ) {
                    60
                } else {
                    35
                },
            ),
            candidate("broad", 20),
        ],
    )?;
    let (hair, hair_decision) = choose(
        &input.seed,
        PopulationRelation::HairForAge,
        age.stable_variant_id(),
        &input.available_bridges,
        &[
            candidate("brown hair", 45),
            candidate("fair hair", 25),
            candidate("black hair", 15),
            candidate("red hair", 5),
            candidate("grey hair", if age == AgeBand::Elder { 60 } else { 5 }),
        ],
    )?;
    let (household_kind, household_decision) = choose(
        &input.seed,
        PopulationRelation::HouseholdForAgeProfession,
        &format!(
            "{}:{}",
            age.stable_variant_id(),
            profession.stable_variant_id()
        ),
        &input.available_bridges,
        &[
            candidate(
                "independent",
                if matches!(age, AgeBand::Adult | AgeBand::Elder) {
                    60
                } else {
                    0
                },
            ),
            candidate(
                "family",
                if matches!(age, AgeBand::Child | AgeBand::Adolescent) {
                    80
                } else {
                    35
                },
            ),
            candidate(
                "employer",
                if profession == Profession::Retainer || profession == Profession::ServiceProvider {
                    70
                } else {
                    5
                },
            ),
        ],
    )?;
    Ok(GeneratedPopulationProfile {
        age,
        profession,
        schedule,
        height: height.into(),
        build: build.into(),
        hair: hair.into(),
        household_kind: household_kind.into(),
        decisions: vec![
            age_decision,
            profession_decision,
            schedule_decision,
            height_decision,
            build_decision,
            hair_decision,
            household_decision,
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(seed: &str, location: LocationContext) -> GenerationInput {
        GenerationInput {
            seed: seed.into(),
            location,
            role: PopulationRole::Resident,
            local_role: "witness".into(),
            age: None,
            available_bridges: BTreeSet::from([
                PresenceBridge::NearbyHome,
                PresenceBridge::HouseholdErrand,
                PresenceBridge::RetainerErrand,
            ]),
        }
    }
    #[test]
    fn relation_failures_retain_their_classification_and_sampling_cause() {
        let root = "failure-case".into();
        let relation = PopulationRelation::Height;
        assert!(matches!(
            choose(
                &root,
                relation,
                "market",
                &BTreeSet::new(),
                &[candidate("short", 1), candidate("short", 1)]
            ),
            Err(PopulationError::DuplicateCandidate {
                relation: PopulationRelation::Height
            })
        ));
        let error = choose(
            &root,
            relation,
            "market",
            &BTreeSet::new(),
            &[bridged("short", 1, PresenceBridge::NearbyHome)],
        )
        .unwrap_err();
        assert!(matches!(
            error,
            PopulationError::Sampling {
                source: fabelgeist_determinism::SamplingError::EmptyCandidates,
                ..
            }
        ));
        assert!(
            std::error::Error::source(&error)
                .unwrap()
                .is::<fabelgeist_determinism::SamplingError>()
        );
    }
    #[test]
    fn provider_role_supplies_profession_and_adult_schedule_together() {
        let mut input = input("provider", LocationContext::Market);
        input.role = PopulationRole::ServiceProvider {
            profession: "armourer".into(),
            service_id: Some("forge".into()),
        };
        let profile = generate(&input).unwrap();
        assert_eq!(profile.profession, Profession::ServiceProvider);
        assert_eq!(profile.schedule, Schedule::Provider);
        assert!(matches!(profile.age, AgeBand::Adult | AgeBand::Elder));
        let explanation = profile
            .decisions
            .iter()
            .find(|decision| decision.relation == "profession_at_location")
            .unwrap();
        assert_eq!(explanation.decision, "armourer");
        assert_eq!(explanation.context, "service:forge;role:witness");
    }

    #[test]
    fn production_profile_is_deterministic_and_explanation_round_trips() {
        let value = generate(&input("same", LocationContext::Market)).unwrap();
        assert_eq!(
            value,
            generate(&input("same", LocationContext::Market)).unwrap()
        );
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(value, serde_json::from_str(&json).unwrap());
    }
    #[test]
    fn hard_zero_and_missing_required_bridges_are_impossible() {
        let mut i = input("x", LocationContext::AdultVenue);
        i.available_bridges.clear();
        for n in 0..500 {
            i.seed = format!("x-{n}").into();
            let p = generate(&i).unwrap();
            assert!(matches!(p.age, AgeBand::Adult | AgeBand::Elder));
            assert_ne!(p.profession, Profession::Retainer);
        }
    }
    #[test]
    fn adult_venue_young_people_are_rare_and_always_bridged() {
        let mut young = 0;
        for n in 0..4000 {
            let p = generate(&input(&format!("venue-{n}"), LocationContext::AdultVenue)).unwrap();
            if matches!(p.age, AgeBand::Child | AgeBand::Adolescent) {
                young += 1;
                let d = p
                    .decisions
                    .iter()
                    .find(|d| d.relation == "age_at_location")
                    .unwrap();
                assert!(matches!(
                    d.bridge,
                    Some(PresenceBridge::NearbyHome | PresenceBridge::HouseholdErrand)
                ));
            }
        }
        assert!(young > 0 && young < 400);
    }
    #[test]
    fn rare_retainer_outside_keep_preserves_bridge() {
        let mut found = None;
        for n in 0..20000 {
            let p = generate(&input(&format!("retainer-{n}"), LocationContext::Overview)).unwrap();
            if p.profession == Profession::Retainer {
                found = Some(p);
                break;
            }
        }
        let p = found.expect("deterministic corpus should include rare retainer");
        assert_eq!(
            p.decisions
                .iter()
                .find(|d| d.relation == "profession_at_location")
                .unwrap()
                .bridge,
            Some(PresenceBridge::RetainerErrand)
        );
    }

    #[test]
    fn population_relations_have_independent_streams() {
        assert_ne!(
            population_relation_seed(&"same".into(), PopulationRelation::AgeAtLocation),
            population_relation_seed(&"same".into(), PopulationRelation::ProfessionAtLocation)
        );
        assert_ne!(
            population_relation_seed(&"same".into(), PopulationRelation::AgeAtLocation),
            population_relation_seed(&"different".into(), PopulationRelation::AgeAtLocation)
        );
    }

    #[test]
    fn building_plan_seed_is_stable_per_settlement_identity() {
        assert_eq!(settlement_building_seed("same"), 16_612_061_259_017_072_879);
        assert_ne!(
            settlement_building_seed("same"),
            settlement_building_seed("different")
        );
    }

    #[test]
    fn planned_age_precedes_age_dependent_profile_generation() {
        let mut adult = input("planned-age", LocationContext::Overview);
        adult.age = Some(AgeBand::Adult);
        let mut elder = adult.clone();
        elder.age = Some(AgeBand::Elder);
        let adult = generate(&adult).unwrap();
        let elder = generate(&elder).unwrap();
        assert_eq!(adult.age, AgeBand::Adult);
        assert_eq!(elder.age, AgeBand::Elder);
        assert_eq!(adult.profession, elder.profession);
        assert_eq!(adult.schedule, elder.schedule);
        assert_eq!(adult.height, elder.height);
        assert_eq!(adult.build, elder.build);
        assert_eq!(adult.decisions[0].context, "household-plan");
        assert_eq!(elder.decisions[0].context, "household-plan");
        assert_eq!(adult.decisions[5].context, "Adult");
        assert_eq!(elder.decisions[5].context, "Elder");
    }
}
