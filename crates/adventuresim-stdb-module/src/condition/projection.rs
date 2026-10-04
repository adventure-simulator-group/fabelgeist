//! Refresh authoritative strategic condition and dependent capability together.

use super::*;

fn evaluate_strategic_condition(
    ctx: &ReducerContext,
    character_id: CharacterId,
    morale_bonus_cap: f32,
    morale_bonus_shares: &[(CharacterId, f32)],
) -> Result<(CharacterStrategicCondition, Vec<ProjectedMoraleSource>), StrategicConditionError> {
    initialize_character_condition(ctx, character_id);
    let condition = ctx
        .db
        .character_condition()
        .character_id()
        .find(u64::from(character_id))
        .ok_or(StrategicConditionError::Missing {
            character: character_id,
            component: ConditionComponent::Condition,
        })?;
    let (attributes, limbs, stats, _) = load_character_parts(ctx, character_id)?;
    let will = mental_check(ctx, character_id, Skill::Will)?;
    let (listener_base_morale, mut sources) = base_morale(ctx, character_id)?;
    let party_members = party_character_ids(ctx, character_id)?;
    let fervor =
        if let Some((_, religion)) = party_religion_context(ctx, character_id, &party_members)? {
            fervor_fraction(
                crate::personality::personality_or_neutral(ctx, character_id)
                    .conviction
                    .strength(),
                religion.own_cohort,
                listener_base_morale.max(0.0),
                religion.party_command,
            )
        } else {
            0.0
        };

    if listener_base_morale < 0.0 {
        let deficit = -listener_base_morale;
        let mut ally_lifts = Vec::new();
        for (member_id, fraction) in morale_bonus_shares.iter().copied() {
            if member_id != character_id && fraction > 0.0 {
                let ally = ctx.db.character().id().find(u64::from(member_id)).ok_or(
                    StrategicConditionError::Missing {
                        character: member_id,
                        component: ConditionComponent::PartyMember,
                    },
                )?;
                let (social_multiplier, social_trait) =
                    crate::personality::ally_restoration_multiplier_for_character(
                        ctx,
                        character_id,
                    );
                ally_lifts.push((
                    member_id,
                    ally.name,
                    deficit * fraction * social_multiplier,
                    social_trait,
                ));
            }
        }
        let total_lift: f32 = ally_lifts.iter().map(|(_, _, lift, _)| *lift).sum();
        let scale = if total_lift > deficit {
            deficit / total_lift
        } else {
            1.0
        };
        for (member_id, name, lift, _social_trait) in ally_lifts {
            sources.push(ProjectedMoraleSource {
                key: format!("ally-{member_id}"),
                kind: MoraleSourceKind::Ally,
                label: format!("Encouraged by {name}"),
                magnitude: lift * scale,
            });
        }
    }

    let morale = sources
        .iter()
        .map(|source| source.magnitude)
        .sum::<f32>()
        .min(listener_base_morale.max(0.0));
    let morale_bonus = morale_bonus_shares
        .iter()
        .find_map(|(member_id, bonus)| (*member_id == character_id).then_some(*bonus))
        .unwrap_or(0.0);
    let pain = pain_incapacitation(total_damage(&limbs), will);
    let blood_loss =
        blood_loss_incapacitation(condition.current_blood_ml, condition.maximum_blood_ml);
    let fatigue_ratio = stats.fatigue_by_parts(&attributes, &limbs);
    let SurvivalProjection {
        needs,
        exposure,
        water_capacity,
        hunger,
        thirst,
        thermal,
    } = SurvivalProjection::load(ctx, character_id)?;
    let incapacitation = StrategicIncapacitation {
        pain,
        blood_loss,
        fear: fear_incapacitation(morale),
        fatigue: fatigue_incapacitation(fatigue_ratio),
        hunger,
        thirst,
        thermal,
    };
    let status = incapacitation.status();
    Ok((
        CharacterStrategicCondition {
            character_id: u64::from(character_id),
            morale,
            morale_bonus,
            morale_bonus_cap,
            fervor,
            pain: incapacitation.pain,
            blood_loss: incapacitation.blood_loss,
            fear: incapacitation.fear,
            fatigue: incapacitation.fatigue,
            hunger: incapacitation.hunger,
            thirst: incapacitation.thirst,
            thermal: incapacitation.thermal,
            wetness_bps: exposure.wetness_bps,
            thermal_strain: exposure.thermal_strain,
            food_days: food_reserve_days(&needs),
            water_days: water_reserve_days(&needs),
            water_capacity_ml: water_capacity,
            incapacitation: incapacitation.total(),
            check_multiplier: incapacitation.check_multiplier(),
            status,
        },
        sources,
    ))
}

fn refresh_one_strategic_condition(
    ctx: &ReducerContext,
    character_id: CharacterId,
    morale_bonus_cap: f32,
    morale_bonus_shares: &[(CharacterId, f32)],
) -> Result<CharacterStrategicCondition, StrategicConditionError> {
    let (row, sources) =
        evaluate_strategic_condition(ctx, character_id, morale_bonus_cap, morale_bonus_shares)?;
    if let Some(existing) = ctx
        .db
        .character_strategic_condition()
        .character_id()
        .find(u64::from(character_id))
    {
        if existing != row {
            ctx.db
                .character_strategic_condition()
                .character_id()
                .update(row.clone());
        }
    } else {
        ctx.db.character_strategic_condition().insert(row.clone());
    }
    let old_source_ids: Vec<String> = ctx
        .db
        .character_morale_source()
        .character_id()
        .filter(u64::from(character_id))
        .map(|source| source.id)
        .collect();
    for id in old_source_ids {
        ctx.db.character_morale_source().id().delete(&id);
    }
    for source in sources {
        ctx.db
            .character_morale_source()
            .insert(CharacterMoraleSource {
                id: format!("{character_id}:{}", source.key),
                character_id: u64::from(character_id),
                kind: source.kind,
                label: source.label,
                magnitude: source.magnitude,
            });
    }
    crate::social::prune_social_addresses(ctx, character_id);
    Ok(row)
}

pub(super) fn refresh_character_strategic_condition_projection(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<CharacterStrategicCondition, StrategicConditionError> {
    let party_members = party_character_ids(ctx, character_id)?;
    let rows = refresh_party_strategic_condition_projection(ctx, &party_members)?;
    rows.into_iter()
        .find(|row| CharacterId::from(row.character_id) == character_id)
        .ok_or_else(|| StrategicConditionError::NotPartyMember {
            character: character_id,
        })
}

pub(super) fn refresh_party_strategic_condition_projection(
    ctx: &ReducerContext,
    party_members: &[CharacterId],
) -> Result<Vec<CharacterStrategicCondition>, StrategicConditionError> {
    let (morale_bonus_cap, morale_bonus_shares) = party_morale_support(ctx, party_members)?;
    let rows = party_members
        .iter()
        .copied()
        .map(|member_id| {
            refresh_one_strategic_condition(ctx, member_id, morale_bonus_cap, &morale_bonus_shares)
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Combat power consumes this projection. Keep its public aggregate in the
    // same transaction as the condition rows so disease, time, and recovery
    // changes cannot leave a stale readiness snapshot behind.
    for row in &rows {
        crate::capability::refresh_character_capability(ctx, (row.character_id).into())?;
    }
    Ok(rows)
}

pub(crate) fn refresh_character_strategic_condition(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<CharacterStrategicCondition, StrategicConditionError> {
    let mut requested = refresh_character_strategic_condition_projection(ctx, character_id)?;
    if refuse_expired_holy_day_demands(ctx, character_id, false)? {
        requested = refresh_character_strategic_condition_projection(ctx, character_id)?;
    }
    ensure_holy_day_demand(ctx, &requested)?;
    Ok(requested)
}
