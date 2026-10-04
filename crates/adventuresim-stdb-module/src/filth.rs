//! Durable strategic filth and automatic washing.

mod soap_error;
use adventuresim_core::identity::InventoryItemId;
use adventuresim_core::physical_object::CarriedInventoryScope;
pub(crate) use soap_error::SoapConsumptionError;

mod exposure_windows;

use adventuresim_core::disease::DiseaseId;
use adventuresim_core::filth::{
    self, Deposit, DiseaseSnapshot, FilthOrigin, FilthSubstance, SoapSource, SoapStackId, WashStack,
};
use adventuresim_world_schema::calendar::{MINUTES_PER_DAY, StrategicMinute};
use spacetimedb::{ReducerContext, Table, table};

use crate::character::character;
use crate::{
    character_attributes, character_time, infection_episode, inventory_item, limb_injury,
    party_inventory_item, retained_projectile,
};

use adventuresim_core::item_references::SOFT_SOAP_ID;

/// Sanitized visible deposit. Disease identity is never copied into this table.
#[derive(Clone, Debug)]
#[table(accessor = character_filth, public)]
pub struct CharacterFilth {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub character_id: u64,
    pub substance: FilthSubstance,
    pub origin: FilthOrigin,
    pub amount: u16,
    pub deposited_at: StrategicMinute,
}

/// Exact source identity is private; public subscribers receive only origin.
#[derive(Clone, Debug)]
#[table(accessor = filth_provenance)]
pub struct FilthProvenance {
    #[primary_key]
    pub filth_id: u64,
    pub source_character_id: Option<u64>,
}

/// Private snapshot of source infection provenance at the instant of deposition.
#[derive(Clone, Debug)]
#[table(accessor = filth_disease_snapshot)]
pub struct FilthDiseaseSnapshot {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub filth_id: u64,
    pub disease_id: String,
    pub episode_id: u64,
}

/// Exact fractional travel-dirt accumulator. Eight dirt per 1,440 movement
/// minutes is represented as integer dirt-minute numerator for chunk invariance.
#[derive(Clone, Debug)]
#[table(accessor = travel_filth_progress)]
pub struct TravelFilthProgress {
    #[primary_key]
    pub character_id: u64,
    pub remainder_numerator: u16,
}

/// Private deterministic exposure cursor. Absolute-minute seeds plus this
/// cursor prevent split intervals from rerolling already evaluated blood.
#[derive(Clone, Debug)]
#[table(accessor = blood_exposure_checkpoint)]
pub struct BloodExposureCheckpoint {
    #[primary_key]
    pub id: String,
    pub character_id: u64,
    pub disease_id: String,
    pub evaluated_through: StrategicMinute,
}

fn predicted_wound_routes(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    allow_healing: bool,
) -> Result<Vec<filth::TimedCutRoute>, String> {
    let natural = if allow_healing {
        crate::time::health_recovered_per_day(crate::time::party_physiology_check(
            ctx,
            (character_id).into(),
        )?)
    } else {
        0.0
    };
    Ok(ctx
        .db
        .limb_injury()
        .character_id()
        .filter(u64::from(character_id))
        .filter(|injury| injury.cut_damage > 0.0)
        .map(|injury| {
            let state = if injury.stitched {
                filth::CutRouteState::Stitched
            } else if injury.bandaged {
                filth::CutRouteState::Bandaged
            } else {
                filth::CutRouteState::Open
            };
            let active_minutes = if allow_healing && injury.bandaged {
                let stitch_bonus = if injury.stitched {
                    injury.stitch_quality.max(0.0) * crate::surgery::STITCH_HEALING_BONUS_PER_LEVEL
                } else {
                    0.0
                };
                let projectile_term = if ctx
                    .db
                    .retained_projectile()
                    .character_id()
                    .filter(u64::from(character_id))
                    .any(|projectile| projectile.limb == injury.limb)
                {
                    crate::surgery::RETAINED_PROJECTILE_HEALING_MULTIPLIER
                } else {
                    1.0
                };
                let per_day = (natural + 0.01 + stitch_bonus) * projectile_term;
                Some(
                    ((injury.cut_damage / per_day) * MINUTES_PER_DAY as f32)
                        .ceil()
                        .max(1.0) as u64,
                )
            } else {
                None
            };
            filth::TimedCutRoute {
                state,
                active_minutes,
            }
        })
        .collect())
}

include!("filth/blood_attempts.rs");

pub fn next_travel_dirt_boundary(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) -> u64 {
    let remainder = ctx
        .db
        .travel_filth_progress()
        .character_id()
        .find(u64::from(character_id))
        .map_or(0, |row| u64::from(row.remainder_numerator));
    (MINUTES_PER_DAY - remainder).div_ceil(8).max(1)
}

pub fn record_travel_elapsed(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    minutes: u64,
    at: StrategicMinute,
) -> Result<u16, String> {
    let mut row = ctx
        .db
        .travel_filth_progress()
        .character_id()
        .find(u64::from(character_id))
        .unwrap_or(TravelFilthProgress {
            character_id: u64::from(character_id),
            remainder_numerator: 0,
        });
    let (dirt, remainder) = filth::travel_dirt_accrual(row.remainder_numerator, minutes);
    row.remainder_numerator = remainder;
    if ctx
        .db
        .travel_filth_progress()
        .character_id()
        .find(u64::from(character_id))
        .is_some()
    {
        ctx.db.travel_filth_progress().character_id().update(row);
    } else {
        ctx.db.travel_filth_progress().insert(row);
    }
    if dirt > 0 {
        deposit(
            ctx,
            (character_id).into(),
            FilthSubstance::Dirt,
            None,
            dirt,
            at,
        )?;
    }
    Ok(dirt)
}

fn total(ctx: &ReducerContext, character_id: adventuresim_core::identity::CharacterId) -> u16 {
    ctx.db
        .character_filth()
        .character_id()
        .filter(u64::from(character_id))
        .map(|d| d.amount)
        .fold(0u16, u16::saturating_add)
        .min(filth::MAX_FILTH)
}

pub fn dirt_total(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) -> u16 {
    ctx.db
        .character_filth()
        .character_id()
        .filter(u64::from(character_id))
        .filter(|d| d.substance == FilthSubstance::Dirt)
        .map(|d| d.amount)
        .fold(0u16, u16::saturating_add)
        .min(filth::MAX_FILTH)
}

/// Reusable strategic boundary: callers persist only final dirt/blood outcomes.
pub fn deposit(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    substance: FilthSubstance,
    source_character_id: Option<adventuresim_core::identity::CharacterId>,
    amount: u16,
    at: StrategicMinute,
) -> Result<Option<u64>, String> {
    let amount = filth::bounded_deposit_amount(total(ctx, (character_id).into()), amount);
    if amount == 0 {
        return Ok(None);
    }
    let row = ctx.db.character_filth().insert(CharacterFilth {
        id: 0,
        character_id: u64::from(character_id),
        substance,
        origin: match source_character_id {
            Some(source) if source == character_id => FilthOrigin::Own,
            Some(_) => FilthOrigin::Foreign,
            None => FilthOrigin::Unknown,
        },
        amount,
        deposited_at: at,
    });
    ctx.db.filth_provenance().insert(FilthProvenance {
        filth_id: row.id,
        source_character_id: source_character_id.map(u64::from),
    });
    if substance == FilthSubstance::Blood
        && let Some(source) = source_character_id
    {
        let immunity = ctx
            .db
            .character_attributes()
            .character_id()
            .find(u64::from(source))
            .map_or(3.0, |attributes| attributes.immunity);
        let mut seen = std::collections::BTreeSet::new();
        for episode in ctx
            .db
            .infection_episode()
            .character_id()
            .filter(u64::from(source))
        {
            let disease_id = episode
                .disease_id
                .parse::<DiseaseId>()
                .map_err(|error| error.to_string())?;
            if !adventuresim_core::disease::definition(disease_id)
                .supports(adventuresim_core::disease::TransmissionVector::Blood)
                || episode.contracted_at > at
                || matches!(
                    adventuresim_core::disease::evaluate(
                        adventuresim_core::disease::InfectionEpisode {
                            id: episode.id,
                            character_id: (source).into(),
                            disease_id,
                            contracted_at: episode.contracted_at,
                            ruleset_version: episode.ruleset_version,
                            phenotype_key_version: episode.phenotype_key_version,
                        },
                        at,
                        immunity,
                    )
                    .stage,
                    adventuresim_core::disease::DiseaseStage::Resolved
                )
                || !seen.insert((disease_id as u8, episode.id))
            {
                continue;
            }
            ctx.db
                .filth_disease_snapshot()
                .insert(FilthDiseaseSnapshot {
                    id: 0,
                    filth_id: row.id,
                    disease_id: disease_id.stable_id().into(),
                    episode_id: episode.id,
                });
        }
    }
    Ok(Some(row.id))
}

pub fn deposit_now(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    substance: FilthSubstance,
    source_character_id: Option<adventuresim_core::identity::CharacterId>,
    amount: u16,
) -> Result<Option<u64>, String> {
    let at = ctx
        .db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .map_or(StrategicMinute::ZERO, |t| t.minutes);
    deposit(
        ctx,
        (character_id).into(),
        substance,
        source_character_id,
        amount,
        at,
    )
}

pub(crate) fn deposits(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) -> Result<Vec<Deposit>, String> {
    ctx.db
        .character_filth()
        .character_id()
        .filter(u64::from(character_id))
        .map(|row| -> Result<Deposit, String> {
            let diseases = ctx
                .db
                .filth_disease_snapshot()
                .filth_id()
                .filter(row.id)
                .map(|s| {
                    s.disease_id
                        .parse::<DiseaseId>()
                        .map(|disease_id| DiseaseSnapshot {
                            disease_id,
                            episode_id: s.episode_id,
                        })
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string())?;
            let source_character_id = ctx
                .db
                .filth_provenance()
                .filth_id()
                .find(row.id)
                .ok_or("Filth provenance is missing")?
                .source_character_id;
            Ok(Deposit {
                id: row.id,
                character_id: (character_id).into(),
                substance: row.substance,
                source_character_id: (source_character_id)
                    .map(adventuresim_core::identity::CharacterId::from),
                amount: row.amount,
                deposited_at: row.deposited_at,
                diseases,
            })
        })
        .collect()
}

fn has_cut(ctx: &ReducerContext, character_id: adventuresim_core::identity::CharacterId) -> bool {
    ctx.db
        .limb_injury()
        .character_id()
        .filter(u64::from(character_id))
        .any(|i| i.cut_damage > 0.0)
}

pub const SOAP_FRACTION_PER_CLEANSING_POINT:
    adventuresim_core::inventory_measurement::ConsumableFractionMicros =
    adventuresim_core::inventory_measurement::ConsumableFractionMicros::whole_divided_by(
        filth::SOAP_CLEANSING_CAPACITY as u32,
    );

include!("filth/soap_consumption.rs");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WashSummary {
    pub total_units: u32,
    pub personal_units: u32,
    pub shared_units: u32,
    pub stacks: Vec<(SoapStackId, u32)>,
}

struct PlannedCharacterWash {
    character_id: adventuresim_core::identity::CharacterId,
    plan: filth::WashPlan,
}

fn take_units(pool: &mut [WashStack], mut wanted: u32) -> Vec<WashStack> {
    let mut assigned = Vec::new();
    pool.sort_by_key(|stack| stack.key);
    for stack in pool {
        if wanted == 0 {
            break;
        }
        let quantity = stack.quantity.min(wanted);
        stack.quantity -= quantity;
        wanted -= quantity;
        if quantity > 0 {
            assigned.push(WashStack {
                key: stack.key,
                quantity,
            });
        }
    }
    assigned
}

fn plan_party_wash(
    ctx: &ReducerContext,
    character_ids: &[adventuresim_core::identity::CharacterId],
) -> Result<(Vec<PlannedCharacterWash>, WashSummary), String> {
    struct Subject {
        id: adventuresim_core::identity::CharacterId,
        dirty: Vec<Deposit>,
        cut: bool,
        assigned: Vec<WashStack>,
        remaining: u32,
    }
    let mut subjects = Vec::new();
    let mut party_id = None;
    for id in character_ids.iter().copied() {
        let character = ctx
            .db
            .character()
            .id()
            .find(u64::from(id))
            .ok_or("Character not found")?;
        if !character.alive {
            continue;
        }
        party_id = party_id.or(character.party_id.clone());
        let dirty = deposits(ctx, (id).into())?;
        if dirty.is_empty() {
            continue;
        }
        let needed = dirty.iter().map(|d| u32::from(d.amount)).sum::<u32>();
        let mut personal = ctx
            .db
            .inventory_item()
            .character_and_item_id()
            .filter((u64::from(id), SOFT_SOAP_ID))
            .map(|stack| WashStack {
                key: SoapStackId {
                    source: SoapSource::Personal,
                    id: stack.id,
                },
                quantity: crate::inventory_amount::personal_fraction(ctx, stack.id)
                    .unwrap_or_default()
                    .get()
                    / SOAP_FRACTION_PER_CLEANSING_POINT.get(),
            })
            .collect::<Vec<_>>();
        let assigned = take_units(&mut personal, needed);
        let personal_used = assigned.iter().map(|stack| stack.quantity).sum::<u32>();
        subjects.push(Subject {
            id,
            cut: has_cut(ctx, (id).into()),
            dirty,
            assigned,
            remaining: needed.saturating_sub(personal_used),
        });
    }
    let mut shared = party_id.as_ref().map_or_else(Vec::new, |party_id| {
        ctx.db
            .party_inventory_item()
            .party_id()
            .filter(party_id)
            .filter(|stack| stack.item_id == SOFT_SOAP_ID)
            .map(|stack| WashStack {
                key: SoapStackId {
                    source: SoapSource::Party,
                    id: stack.id,
                },
                quantity: crate::inventory_amount::party_fraction(ctx, stack.id)
                    .unwrap_or_default()
                    .get()
                    / SOAP_FRACTION_PER_CLEANSING_POINT.get(),
            })
            .collect::<Vec<_>>()
    });
    let mut priorities = subjects
        .iter()
        .map(|subject| {
            let now = ctx
                .db
                .character_time()
                .character_id()
                .find(u64::from(subject.id))
                .map_or(StrategicMinute::ZERO, |t| t.minutes);
            let exposure = filth::blood_exposure(
                &subject.dirty,
                adventuresim_core::disease::DiseaseId::Plague,
                now,
                filth::timed_cut_exposure(
                    &predicted_wound_routes(ctx, (subject.id).into(), false)?,
                    0,
                ),
            );
            Ok::<_, String>(filth::wash_priority(
                (subject.id).into(),
                &subject.dirty,
                subject.cut,
                exposure,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    filth::sort_wash_priorities(&mut priorities);
    for priority in priorities {
        let subject = subjects
            .iter_mut()
            .find(|subject| {
                adventuresim_core::identity::CharacterId::from(subject.id) == priority.character_id
            })
            .unwrap();
        let assigned = take_units(&mut shared, subject.remaining);
        let used = assigned.iter().map(|stack| stack.quantity).sum::<u32>();
        subject.remaining -= used;
        subject.assigned.extend(assigned);
    }
    let mut summary = WashSummary::default();
    let planned = subjects
        .into_iter()
        .map(|subject| {
            let plan = filth::plan_wash(&subject.dirty, &subject.assigned, subject.cut);
            for (key, quantity) in &plan.soap_stacks {
                summary.total_units += quantity;
                match key.source {
                    SoapSource::Personal => summary.personal_units += quantity,
                    SoapSource::Party => summary.shared_units += quantity,
                }
                summary.stacks.push((*key, *quantity));
            }
            PlannedCharacterWash {
                character_id: subject.id,
                plan,
            }
        })
        .collect();
    summary.stacks.sort_by_key(|(key, _)| *key);
    Ok((planned, summary))
}

pub fn preview_party_wash(
    ctx: &ReducerContext,
    character_ids: &[adventuresim_core::identity::CharacterId],
) -> Result<WashSummary, String> {
    plan_party_wash(ctx, character_ids).map(|(_, summary)| summary)
}

pub fn wash_party_before_explicit_rest(
    ctx: &ReducerContext,
    character_ids: &[adventuresim_core::identity::CharacterId],
) -> Result<WashSummary, String> {
    let (planned, summary) = plan_party_wash(ctx, character_ids)?;
    // Preflight exact tagged identities before the first mutation.
    for (key, quantity) in &summary.stacks {
        match key.source {
            SoapSource::Personal => {
                let stack = ctx
                    .db
                    .inventory_item()
                    .id()
                    .find(key.id)
                    .ok_or("Planned personal soap stack is missing")?;
                if stack.item_id != SOFT_SOAP_ID
                    || crate::inventory_amount::personal_fraction(ctx, stack.id).unwrap_or_default()
                        < SOAP_FRACTION_PER_CLEANSING_POINT
                            .checked_mul(*quantity)
                            .ok_or("Planned personal soap amount overflow")?
                {
                    return Err("Planned personal soap is no longer available".into());
                }
            }
            SoapSource::Party => {
                let stack = ctx
                    .db
                    .party_inventory_item()
                    .id()
                    .find(key.id)
                    .ok_or("Planned shared soap stack is missing")?;
                if stack.item_id != SOFT_SOAP_ID
                    || crate::inventory_amount::party_fraction(ctx, stack.id).unwrap_or_default()
                        < SOAP_FRACTION_PER_CLEANSING_POINT
                            .checked_mul(*quantity)
                            .ok_or("Planned shared soap amount overflow")?
                {
                    return Err("Planned shared soap is no longer available".into());
                }
            }
        }
    }
    for (key, quantity) in &summary.stacks {
        match key.source {
            SoapSource::Personal => consume_personal(ctx, key.id.into(), *quantity)?,
            SoapSource::Party => consume_party(ctx, key.id.into(), *quantity)?,
        }
    }
    for character in planned {
        for (id, removed) in character.plan.cleaned_deposits {
            if let Some(mut row) = ctx.db.character_filth().id().find(id) {
                if adventuresim_core::identity::CharacterId::from(row.character_id)
                    != character.character_id
                    || row.amount < removed
                {
                    return Err("Planned filth deposit changed before washing".into());
                }
                row.amount = row
                    .amount
                    .checked_sub(removed)
                    .ok_or("Filth removal underflow")?;
                if row.amount == 0 {
                    for snapshot in ctx
                        .db
                        .filth_disease_snapshot()
                        .filth_id()
                        .filter(id)
                        .collect::<Vec<_>>()
                    {
                        ctx.db.filth_disease_snapshot().id().delete(snapshot.id);
                    }
                    ctx.db.filth_provenance().filth_id().delete(id);
                    ctx.db.character_filth().id().delete(id);
                } else {
                    ctx.db.character_filth().id().update(row);
                }
            } else {
                return Err("Planned filth deposit is missing".into());
            }
        }
    }
    Ok(summary)
}

/// Single-character settlement rest wrapper.
pub fn wash_before_explicit_rest(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) -> Result<u32, String> {
    Ok(wash_party_before_explicit_rest(ctx, &[character_id])?.total_units)
}

pub fn preview_soap_units(ctx: &ReducerContext, character_id: u64) -> u32 {
    let amount: u32 = ctx
        .db
        .character_filth()
        .character_id()
        .filter(character_id)
        .map(|d| u32::from(d.amount))
        .sum();
    if amount == 0 {
        return 0;
    }
    let available: u32 = ctx
        .db
        .inventory_item()
        .character_and_item_id()
        .filter((character_id, SOFT_SOAP_ID))
        .map(|s| {
            crate::inventory_amount::personal_fraction(ctx, s.id)
                .unwrap_or_default()
                .get()
                / SOAP_FRACTION_PER_CLEANSING_POINT.get()
        })
        .sum::<u32>()
        + ctx
            .db
            .character()
            .id()
            .find(character_id)
            .and_then(|c| c.party_id)
            .map_or(0, |p| {
                ctx.db
                    .party_inventory_item()
                    .iter()
                    .filter(|s| s.party_id == p && s.item_id == SOFT_SOAP_ID)
                    .map(|s| {
                        crate::inventory_amount::party_fraction(ctx, s.id)
                            .unwrap_or_default()
                            .get()
                            / SOAP_FRACTION_PER_CLEANSING_POINT.get()
                    })
                    .sum()
            });
    amount.min(available)
}

pub(crate) fn seed_demo(
    ctx: &ReducerContext,
    character_id: u64,
    foreign_source_id: u64,
) -> Result<(), String> {
    for row in ctx
        .db
        .character_filth()
        .character_id()
        .filter(character_id)
        .collect::<Vec<_>>()
    {
        for snapshot in ctx
            .db
            .filth_disease_snapshot()
            .filth_id()
            .filter(row.id)
            .collect::<Vec<_>>()
        {
            ctx.db.filth_disease_snapshot().id().delete(snapshot.id);
        }
        ctx.db.character_filth().id().delete(row.id);
    }
    deposit(
        ctx,
        (character_id).into(),
        FilthSubstance::Dirt,
        None,
        38,
        StrategicMinute::new(86_000),
    )?;
    deposit(
        ctx,
        (character_id).into(),
        FilthSubstance::Blood,
        Some(adventuresim_core::identity::CharacterId::from(
            foreign_source_id,
        )),
        27,
        StrategicMinute::new(86_200),
    )?;
    crate::personality::set_personality_axis_score(
        ctx,
        character_id,
        crate::personality::MutablePersonalityAxis::Hygiene,
        crate::personality::PERSONALITY_SCORE_LIMIT,
    )?;
    let existing = ctx
        .db
        .inventory_item()
        .character_and_item_id()
        .filter((character_id, SOFT_SOAP_ID))
        .map(|row| row.quantity)
        .sum::<u32>();
    if existing < 3 {
        crate::add_inventory_item(
            ctx,
            character_id.into(),
            &(SOFT_SOAP_ID).into(),
            (3 - existing).into(),
        );
    }
    Ok(())
}
