//! Project morale stimuli and share party command support.

use super::*;

pub(super) fn base_morale(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<(f32, Vec<ProjectedMoraleSource>), StrategicConditionError> {
    let MoraleInputs {
        character,
        current_minute,
        limbs,
        will,
    } = MoraleInputs::load(ctx, character_id)?;
    let personality = crate::personality::personality_or_neutral(ctx, character_id);
    let mut raw_sources = Vec::new();
    let mut add_source = |key: String,
                          kind: MoraleSourceKind,
                          label: String,
                          magnitude: f32,
                          stimulus: crate::personality::MoraleStimulus| {
        // True personality changes the authoritative magnitude, but labels are
        // public presentation data and must never reveal that private truth.
        let (magnitude, _) =
            crate::personality::react_raw_for_character(ctx, character_id, stimulus, magnitude);
        raw_sources.push(ProjectedMoraleSource {
            key,
            kind,
            label,
            magnitude,
        });
    };

    let injury = total_damage(&limbs) * INJURY_MORALE_PER_HEALTH_DEFICIT;
    if injury > 0.0 {
        add_source(
            "injuries".into(),
            MoraleSourceKind::Injury,
            "Injuries".into(),
            -injury,
            crate::personality::MoraleStimulus::Other,
        );
    }

    let filth_total = ctx
        .db
        .character_filth()
        .character_id()
        .filter(u64::from(character_id))
        .map(|deposit| f32::from(deposit.amount))
        .sum::<f32>()
        .min(f32::from(adventuresim_core::filth::MAX_FILTH));
    let filth_fraction = filth_total / f32::from(adventuresim_core::filth::MAX_FILTH);
    let hygiene_score =
        crate::personality::personality_scores_or_neutral(ctx, character_id).hygiene;
    let baseline_hygiene_morale = -8.0 * filth_fraction;
    let hygiene_endpoint = if hygiene_score >= 0 {
        if filth_total == 0.0 {
            2.0
        } else {
            -20.0 * filth_fraction
        }
    } else {
        0.0
    };
    let hygiene_ratio = f32::from(hygiene_score.unsigned_abs())
        / f32::from(crate::personality::PERSONALITY_SCORE_LIMIT as u16);
    let hygiene_morale =
        baseline_hygiene_morale + (hygiene_endpoint - baseline_hygiene_morale) * hygiene_ratio;
    if hygiene_morale != 0.0 {
        add_source(
            "cleanliness".into(),
            MoraleSourceKind::Cleanliness,
            if hygiene_morale > 0.0 {
                "Clean".into()
            } else {
                "Filthy".into()
            },
            hygiene_morale,
            crate::personality::MoraleStimulus::Other,
        );
    }

    let party_members = party_character_ids(ctx, character_id)?;
    if let Some((religion_id, religion)) =
        party_religion_context(ctx, character_id, &party_members)?
    {
        if personality.conviction == crate::personality::Conviction::Zealous
            && religion.knowledge > 0.0
        {
            add_source(
                format!("religion-{religion_id}"),
                MoraleSourceKind::Religion,
                format!("Religious leadership for {}", religion_label(&religion_id)),
                religion.knowledge,
                crate::personality::MoraleStimulus::Religious,
            );
        }
        let discord = religious_discord(religion.foreign_pressure, religion.party_command);
        if discord > 0.0 {
            add_source(
                "religious-discord".into(),
                MoraleSourceKind::ReligiousDiscord,
                "Religious discord".into(),
                -discord,
                crate::personality::MoraleStimulus::Religious,
            );
        }
        let prayer_minutes = ctx
            .db
            .character_training_schedule()
            .character_id()
            .find(u64::from(character_id))
            .map_or(0, |schedule| schedule.downtime.prayer_minutes);
        if prayer_minutes > 0 {
            add_source(
                "daily-prayer".into(),
                MoraleSourceKind::Prayer,
                "Daily prayer".into(),
                led_prayer_morale(prayer_minutes, religion.knowledge),
                crate::personality::MoraleStimulus::Religious,
            );
        }
        let prayer_fervor = fervor_fraction(
            crate::personality::conviction_strength_for_character(ctx, character_id),
            religion.own_cohort,
            0.0,
            religion.party_command,
        );
        let neglect = religious_neglect_morale(prayer_fervor, religion.party_command)
            * (1.0 - prayer_observance(prayer_fervor, prayer_minutes));
        if neglect > 0.0 {
            add_source(
                "neglected-prayer".into(),
                MoraleSourceKind::Prayer,
                "Insufficient daily prayer".into(),
                -neglect,
                crate::personality::MoraleStimulus::Religious,
            );
        }
    } else {
        let meditation_minutes = ctx
            .db
            .character_training_schedule()
            .character_id()
            .find(u64::from(character_id))
            .map_or(0, |schedule| schedule.downtime.prayer_minutes);
        if meditation_minutes > 0 {
            // Meditation is independent of religious knowledge and Conviction.
            add_source(
                "daily-meditation".into(),
                MoraleSourceKind::Meditation,
                "Daily meditation".into(),
                meditation_morale(meditation_minutes),
                crate::personality::MoraleStimulus::Other,
            );
        }
    }
    let mut allied_power = 0.0;
    for member_id in party_members {
        let capability = crate::capability::refresh_character_capability(ctx, member_id)?;
        allied_power += capability.athletics
            + capability.endurance
            + capability.weapon_precision
            + if capability.melee || capability.ranged {
                2.0
            } else {
                0.0
            }
            + if capability.full_armor {
                2.0
            } else if capability.half_armor || capability.three_quarter_armor {
                1.0
            } else if capability.quarter_armor {
                0.5
            } else {
                0.0
            };
    }

    if let Some(case_site_id) =
        crate::investigation::character_case_site_id(ctx, (character.id).into())
        && let Some(site) = ctx.db.case_site_authority().id_key().find(&case_site_id)
        && let Some(group) = ctx
            .db
            .hostile_group_authority()
            .iter()
            .find(|group| group.case_site_id == site.id)
    {
        let enemy_power = group.enemy_count.max(1) as f32 * (group.difficulty.max(1) as f32 + 4.0);
        let difference = allied_power - enemy_power;
        if difference != 0.0 {
            add_source(
                format!("power-{}", group.id),
                MoraleSourceKind::Power,
                if difference > 0.0 {
                    "Superior allied strength".into()
                } else {
                    format!("Outmatched by {}", group.enemy_type)
                },
                if difference > 0.0 {
                    difference
                } else {
                    difference.abs()
                        * -enemy_fear_multiplier(group.enemy_type.parse().map_err(|source| {
                            StrategicConditionError::UnknownThreat {
                                character: character_id,
                                stored_key: group.enemy_type.clone(),
                                source,
                            }
                        })?)
                },
                if difference < 0.0 {
                    crate::personality::MoraleStimulus::Threat
                } else {
                    crate::personality::MoraleStimulus::Other
                },
            );
        }
    }

    for event in ctx
        .db
        .morale_event()
        .character_id()
        .filter(u64::from(character_id))
    {
        let occurred_at = event.occurred_at_minute;
        let duration = event.expires_at_minute.elapsed_since(occurred_at);
        let age = current_minute.elapsed_since(occurred_at);
        let effect = if event.source_id.as_deref() == Some(LEISURE_MORALE_SOURCE_ID) {
            leisure_morale_effect(event.magnitude, age as f32, duration)
        } else if event.source_id.as_deref() == Some(MASTERY_MORALE_SOURCE_ID) {
            event.magnitude * adventuresim_core::morale::mastery_enjoyment_decay(age, duration)
        } else {
            event.magnitude * morale_event_decay(age, duration)
        };
        if effect != 0.0 {
            let stimulus = crate::personality::morale_event_stimulus(event.kind);
            add_source(
                format!("event-{}", event.id),
                event.kind.into(),
                match event.kind {
                    MoraleEventKind::Victory => "Recent victory".into(),
                    MoraleEventKind::Defeat => "Recent defeat".into(),
                    MoraleEventKind::Leisure => "Restful leisure".into(),
                    MoraleEventKind::MasteryEnjoyment => "Mastery enjoyment".into(),
                    other => other.as_str().replace('_', " "),
                },
                effect,
                stimulus,
            );
        }
    }

    rank_morale_sources(&mut raw_sources, will);
    let morale = raw_sources.iter().map(|source| source.magnitude).sum();
    Ok((morale, raw_sources))
}

pub(super) fn party_morale_support(
    ctx: &ReducerContext,
    party_members: &[CharacterId],
) -> Result<(f32, Vec<(CharacterId, f32)>), StrategicConditionError> {
    let mut commands = Vec::new();
    let mut surplus_weights = Vec::new();
    for member_id in party_members.iter().copied() {
        commands.push(mental_check(ctx, member_id, Skill::Command)?);
        let (member_base_morale, _) = base_morale(ctx, member_id)?;
        let surplus = member_base_morale.max(0.0);
        if surplus > 0.0 {
            surplus_weights.push((member_id, surplus));
        }
    }
    let party_command = aggregate_party_command(commands);
    let bonus_cap = MORALE_BONUS_PER_COMMAND * party_command;
    let combined_surplus = cumulative_morale(surplus_weights.iter().map(|(_, surplus)| *surplus));
    let total_bonus = morale_bonus_fraction(combined_surplus, party_command);
    let total_weight: f32 = surplus_weights.iter().map(|(_, surplus)| *surplus).sum();
    let shares = surplus_weights
        .into_iter()
        .map(|(member_id, surplus)| (member_id, total_bonus * surplus / total_weight))
        .collect();
    Ok((bonus_cap, shares))
}

struct MoraleInputs {
    character: crate::character::Character,
    current_minute: StrategicMinute,
    limbs: CharacterLimbs,
    will: f32,
}

impl MoraleInputs {
    fn load(
        ctx: &ReducerContext,
        character_id: CharacterId,
    ) -> Result<Self, StrategicConditionError> {
        let character = ctx
            .db
            .character()
            .id()
            .find(u64::from(character_id))
            .ok_or(StrategicConditionError::Missing {
                character: character_id,
                component: ConditionComponent::Character,
            })?;
        let current_minute = ctx
            .db
            .character_time()
            .character_id()
            .find(u64::from(character_id))
            .map_or(StrategicMinute::ZERO, |t| t.minutes);
        let (_, limbs, _, _) = load_character_parts(ctx, character_id)?;
        let will = mental_check(ctx, character_id, Skill::Will)?.max(MINIMUM_WILL_CHECK);
        Ok(Self {
            character,
            current_minute,
            limbs,
            will,
        })
    }
}
