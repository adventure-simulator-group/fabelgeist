//! Guarded social request vocabulary and executing-action acceptance.
use super::*;
use adventuresim_core::morale::MoraleSourceKind;

fn prepare_pair(ctx: &ReducerContext, actor: u64, target: u64) -> Result<(), String> {
    use crate::personality::MutablePersonalityAxis;
    use adventuresim_world_schema::OfficialReligion;
    for id in [actor, target] {
        crate::character::create_named_character_with_id(ctx, id, "Social action fixture".into())?;
        let mut character = ctx
            .db
            .character()
            .id()
            .find(id)
            .ok_or("Character missing")?;
        character.current_settlement_id = Some("riverdale".into());
        ctx.db.character().id().update(character);
        for axis in [
            MutablePersonalityAxis::Mirth,
            MutablePersonalityAxis::Courtship,
            MutablePersonalityAxis::Conviction,
        ] {
            crate::personality::set_personality_axis_score(ctx, id, axis, 0)?;
        }
        let mut condition = ctx
            .db
            .character_condition()
            .character_id()
            .find(id)
            .ok_or("Condition missing")?;
        condition.religion_id = Some(OfficialReligion::Lutheran.religion_id().into());
        ctx.db
            .character_condition()
            .character_id()
            .update(condition);
    }
    let mut skills = ctx
        .db
        .character_skills()
        .character_id()
        .find(actor)
        .ok_or("Skills missing")?;
    *skills.religion_hours.direct_mut(OfficialReligion::Lutheran) = 100.0;
    ctx.db.character_skills().character_id().update(skills);
    crate::strategic::attach_seeded_party_member(ctx, actor, target, "companion")?;
    Ok(())
}

#[reducer]
pub fn authority_test_social_action_keys(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    for (index, key) in [
        "reflect",
        "listen",
        "commiserate",
        "pray",
        "reassure",
        "lighten_mood",
        "command",
        "deception",
        "flirt",
        "commiserate",
    ]
    .into_iter()
    .enumerate()
    {
        let actor = 732910 + index as u64 * 2;
        let companion = actor + 1;
        prepare_pair(ctx, actor, companion)?;
        let action = key
            .parse::<SocialActionKind>()
            .map_err(|error| error.to_string())?;
        let target = if action == SocialActionKind::Reflect {
            actor
        } else {
            companion
        };
        let kind = if action == SocialActionKind::Reassure {
            MoraleSourceKind::Injury
        } else {
            MoraleSourceKind::Defeat
        };
        let source_id = format!("authority-social-source:{target}");
        ctx.db
            .character_morale_source()
            .insert(crate::condition::CharacterMoraleSource {
                id: source_id.clone(),
                character_id: target,
                kind,
                label: "Concern fixture".into(),
                magnitude: -15.0,
            });
        if index == 9 {
            ctx.db
                .character_morale_source()
                .insert(crate::condition::CharacterMoraleSource {
                    id: format!("authority-social-source:{actor}"),
                    character_id: actor,
                    kind,
                    label: "Shared concern fixture".into(),
                    magnitude: -15.0,
                });
        }
        if action == SocialActionKind::Commiserate
            && action.skill(shares_concern(ctx, (actor).into(), SocialTopic::Defeat))
                != if index == 9 {
                    Skill::Insight
                } else {
                    Skill::Deception
                }
        {
            return Err("Commiseration selected the wrong executing skill".into());
        }
        // Use the same authoritative path as manual and automatic requests.
        // Clock advancement is covered by the automatic-care lifecycle fixture.
        perform_social_action_authoritative(
            ctx,
            (actor).into(),
            (target).into(),
            source_id.clone(),
            key.into(),
            false,
        )?;
        let receipts: Vec<_> = ctx
            .db
            .social_interaction()
            .actor_id()
            .filter(actor)
            .collect();
        if receipts.len() != 1
            || receipts[0].action_kind != key
            || receipts[0].target_id != target
            || receipts[0].source_id != source_id
        {
            return Err("Executing social action changed its request/receipt binding".into());
        }
        if perform_social_action_authoritative(
            ctx,
            (actor).into(),
            (target).into(),
            source_id,
            key.into(),
            false,
        )
        .is_ok()
        {
            return Err("Repeated social action bypassed its cooldown".into());
        }
    }
    for key in ["", "unknown", "rally", "reframe", "Command"] {
        let error = perform_social_action(ctx, 0, 0, "absent".into(), key.into())
            .expect_err("Unknown key must reject");
        if error != "Unknown social action" {
            return Err("Unknown action reached character admission".into());
        }
    }
    Ok(())
}
