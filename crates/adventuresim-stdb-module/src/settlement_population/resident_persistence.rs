//! Persist finalized resident drafts without revising their demographics.
use super::*;

const CLERIC: &str = "cleric";
const REEVE: &str = "reeve";
const HERBALIST_SERVICE: &str = "herbalist";
const RELIGION_SERVICE: &str = "religion";

struct PreparedResident {
    input: GenerationInput,
    profile: population::GeneratedPopulationProfile,
    age_band: NpcAgeBand,
    stable_seed: u64,
    household: String,
}

impl PreparedResident {
    fn new(draft: &ResidentDraft) -> Result<Self, String> {
        let provider = draft.business_id.is_some() || draft.service.is_some();
        let input = GenerationInput {
            seed: draft.seed.clone(),
            location: location_context(&draft.location)?,
            is_service_provider: provider,
            service_id: draft.service.clone(),
            profession_override: provider.then(|| draft.profession.clone()),
            local_role: draft.role.clone(),
            age: draft.exact_age.map(|age| match age {
                0..=12 => AgeBand::Child,
                13..=17 => AgeBand::Adolescent,
                18..=59 => AgeBand::Adult,
                _ => AgeBand::Elder,
            }),
            available_bridges: BTreeSet::from(RESIDENT_BRIDGES),
        };
        let profile = population::generate(&input)?;
        let age_band = age(profile.age);
        let household = profile.household_kind.to_owned();
        Ok(Self {
            input,
            profile,
            age_band,
            stable_seed: resident_random(&draft.seed, ResidentEntropyStream::Identity).next_u64(),
            household,
        })
    }

    fn exact_age(&self, draft: &ResidentDraft) -> u16 {
        draft.exact_age.unwrap_or(match self.age_band {
            NpcAgeBand::Child => 8,
            NpcAgeBand::Adolescent => 15,
            NpcAgeBand::Adult => 30,
            NpcAgeBand::Elder => 68,
        })
    }

    fn is_provider(&self) -> bool {
        self.input.is_service_provider
    }
}

fn ensure_roles(
    ctx: &ReducerContext,
    settlement_id: &str,
    character_id: u64,
    profession: &str,
) -> Result<(), String> {
    let settlement = ctx
        .db
        .settlement()
        .id()
        .find(settlement_id.to_owned())
        .ok_or("Settlement population references an unknown settlement")?;
    let urban = matches!(
        settlement.category,
        crate::strategic::SettlementCategory::Town
            | crate::strategic::SettlementCategory::City
            | crate::strategic::SettlementCategory::Capital
    );
    crate::social_roles::ensure_character_social_roles(ctx, character_id, settlement_id, urban)?;
    if profession == CLERIC
        && let Some(organization_id) =
            crate::social_roles::religious_organization_for(&settlement.religion_id)
    {
        let role = adventuresim_core::organization::organization(organization_id)
            .and_then(|definition| definition.entry_role_ids.first())
            .ok_or("Religious organization has no entry role")?;
        crate::social_roles::ensure_character_professional_role(
            ctx,
            character_id,
            organization_id,
            role,
        )?;
    }
    Ok(())
}

fn reuse_existing(
    ctx: &ReducerContext,
    settlement_id: &str,
    draft: &ResidentDraft,
) -> Result<bool, String> {
    let Some(existing) = ctx
        .db
        .settlement_resident_profile()
        .character_id()
        .find(draft.character_id())
    else {
        return Ok(false);
    };
    if existing.home_settlement_id != settlement_id {
        return Err("Settlement resident identity crosses a settlement boundary".into());
    }
    ensure_roles(
        ctx,
        settlement_id,
        draft.character_id(),
        &existing.profession,
    )?;
    ensure_business_operator_row(ctx, draft.business_id.as_ref(), draft.character_id())?;
    Ok(true)
}

fn insert_identity(
    ctx: &ReducerContext,
    settlement_id: &str,
    draft: &ResidentDraft,
    prepared: &PreparedResident,
) -> Result<(), String> {
    let personality = crate::personality::personality_from_stable_seed_with_demographics(
        draft.character_id(),
        prepared.stable_seed,
        draft.sex,
        draft.presentation,
    );
    let life = NpcLifeFacts {
        age_years: prepared.exact_age(draft),
        organization_id: None,
        literacy: None,
    };
    insert_persistent_npc_character(
        ctx,
        "Pending resident name".into(),
        draft.character_id(),
        settlement_id,
        prepared.stable_seed,
        None,
        &life,
        &personality,
    )?;
    crate::character::assign_generated_name_demographics(
        ctx,
        crate::character::CharacterId::new(draft.character_id()),
        draft.sex,
        life.age_years,
    )?;
    crate::character::assign_generated_historical_name(
        ctx,
        crate::character::CharacterId::new(draft.character_id()),
        crate::character::NameSeed::new(prepared.stable_seed),
        adventuresim_world_schema::person_names::NameBirthYear::new(
            adventuresim_core::strategic_time::birth_year_from_age(0, life.age_years),
        ),
        None,
    )?;
    ctx.db.npc_policy().insert(NpcPolicy {
        character_id: draft.character_id(),
        home_settlement_id: settlement_id.into(),
        policy_seed: prepared.stable_seed,
    });
    Ok(())
}

fn conversation_id(draft: &ResidentDraft, provider: bool) -> &'static str {
    if !provider {
        "local-resident"
    } else if draft.service.as_deref() == Some(HERBALIST_SERVICE) {
        "herbalist-examination"
    } else if draft.service.as_deref() == Some(RELIGION_SERVICE) {
        "religion-service"
    } else {
        "service-professions"
    }
}

fn insert_profile(
    ctx: &ReducerContext,
    settlement_id: &str,
    draft: &ResidentDraft,
    prepared: &PreparedResident,
) -> SettlementResidentProfile {
    let profession = if prepared.is_provider() {
        draft.profession.as_str()
    } else {
        super::profession(prepared.profile.profession)
    };
    let local_role = if !prepared.is_provider()
        && prepared.profile.profession == Profession::Retainer
        && draft.role != REEVE
    {
        "lord's household retainer"
    } else {
        &draft.role
    };
    ctx.db
        .settlement_resident_profile()
        .insert(SettlementResidentProfile {
            character_id: draft.character_id(),
            projection_id: draft.character_id(),
            home_settlement_id: settlement_id.into(),
            height: prepared.profile.height.clone(),
            build: prepared.profile.build.clone(),
            hair: prepared.profile.hair.clone(),
            facial_hair: if draft.sex == Sex::Male
                && resident_random(&draft.seed, ResidentEntropyStream::FacialHair).index(3) == 0
                && !matches!(prepared.age_band, NpcAgeBand::Child)
            {
                "a neatly kept beard".into()
            } else {
                "none visible".into()
            },
            complexion: ["fair", "ruddy", "weathered", "olive"]
                [resident_random(&draft.seed, ResidentEntropyStream::Complexion).index(4)]
            .into(),
            visible_features: [
                "a small scar at one brow",
                "freckles",
                "work-worn hands",
                "no especially notable marks",
            ][resident_random(&draft.seed, ResidentEntropyStream::VisibleFeature).index(4)]
            .into(),
            clothing: if prepared.is_provider() {
                "clean working clothes appropriate to the trade".into()
            } else {
                "practical local woolens".into()
            },
            profession: profession.into(),
            household_kind: prepared.household.clone(),
            local_role: local_role.into(),
            service_id: draft.service.clone().unwrap_or_default(),
            organization_id: String::new(),
            conversation_id: conversation_id(draft, prepared.is_provider()).into(),
        })
}

fn insert_presence(
    ctx: &ReducerContext,
    settlement_id: &str,
    draft: &ResidentDraft,
    schedule: Schedule,
) {
    let (start_minute, end_minute) = match schedule {
        Schedule::Day => (360, 1200),
        Schedule::Evening => (720, 1380),
        Schedule::Early => (240, 960),
        Schedule::Provider => (0, MINUTES_PER_DAY as u16),
    };
    ctx.db
        .settlement_resident_presence()
        .insert(SettlementResidentPresence {
            character_id: draft.character_id(),
            settlement_id: settlement_id.into(),
            location_id: draft.location.clone(),
            start_minute,
            end_minute,
            is_default: draft.is_default,
            context_suppressed: false,
            health_suppressed: false,
        });
}

pub(super) fn insert_resident_draft(
    ctx: &ReducerContext,
    settlement_id: &str,
    draft: ResidentDraft,
) -> Result<(), String> {
    if reuse_existing(ctx, settlement_id, &draft)? {
        return Ok(());
    }
    let prepared = PreparedResident::new(&draft)?;
    insert_identity(ctx, settlement_id, &draft, &prepared)?;
    let resident = insert_profile(ctx, settlement_id, &draft, &prepared);
    ensure_roles(
        ctx,
        settlement_id,
        resident.character_id,
        &resident.profession,
    )?;
    insert_presence(ctx, settlement_id, &draft, prepared.profile.schedule);
    ensure_business_operator_row(ctx, draft.business_id.as_ref(), draft.character_id())?;
    let explanation = PersistedGenerationExplanation {
        input: prepared.input,
        profile: prepared.profile,
    };
    let relations_json = serde_json::to_string(&explanation)
        .map_err(|error| format!("Could not serialize population explanation: {error}"))?;
    ctx.db
        .settlement_resident_seed_explanation()
        .insert(SettlementResidentSeedExplanation {
            character_id: draft.character_id(),
            seed: draft.seed,
            relations_json,
        });
    Ok(())
}
