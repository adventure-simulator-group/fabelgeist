//! Member observations for terrain capability and the departure frontier.
use super::super::{AppState, data::character_as_observed};
use super::error::{
    DepartureQueryStage, PartyDepartureError, PartyTerrainProfileError, TerrainProfileStage,
};
use crate::spacetimedb::{
    self as db, CharacterAttributes, CharacterLimbs, CharacterSkills, CharacterTime, CharacterView,
    PartyMember, SpacetimeError, SqlQuery, sql_string_literal,
};
use adventuresim_core::identity::CharacterId;
use adventuresim_world_schema::calendar::StrategicMinute;

pub(crate) async fn party_terrain_profile(
    state: &AppState,
    actor: &CharacterView,
) -> std::result::Result<(adventuresim_terrain::TerrainSkillProfile, u16), PartyTerrainProfileError>
{
    let member_ids = if let Some(party_id) = actor.party_id.as_deref() {
        state
            .db
            .query_sats::<PartyMember>(SqlQuery::from(format!(
                "SELECT * FROM party_member WHERE party_id = {}",
                sql_string_literal(party_id)
            )))
            .await
            .map_err(|source: SpacetimeError| -> PartyTerrainProfileError {
                PartyTerrainProfileError::query(TerrainProfileStage::Membership, None, source)
            })?
            .into_iter()
            .map(|member: PartyMember| -> CharacterId { CharacterId::from(member.character_id) })
            .collect::<Vec<_>>()
    } else {
        vec![CharacterId::from(actor.id)]
    };
    let mut checks = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];
    for id in member_ids {
        let Some(TerrainMemberInputs {
            attributes,
            limbs,
            skills,
        }) = TerrainMemberInputs::load(state, id, actor.id.into()).await?
        else {
            continue;
        };
        let direct_hours = |skill| match skill {
            adventuresim_core::skill::Skill::TerrainPlains => skills.terrain_plains_hours,
            adventuresim_core::skill::Skill::TerrainForest => skills.terrain_forest_hours,
            adventuresim_core::skill::Skill::TerrainHills => skills.terrain_hills_hours,
            adventuresim_core::skill::Skill::TerrainWetlands => skills.terrain_wetlands_hours,
            adventuresim_core::skill::Skill::TerrainUrban => skills.terrain_urban_hours,
            adventuresim_core::skill::Skill::TerrainSnow => skills.terrain_snow_hours,
            _ => 0.0,
        };
        for (index, skill) in [
            adventuresim_core::skill::Skill::TerrainPlains,
            adventuresim_core::skill::Skill::TerrainForest,
            adventuresim_core::skill::Skill::TerrainHills,
            adventuresim_core::skill::Skill::TerrainWetlands,
            adventuresim_core::skill::Skill::TerrainUrban,
            adventuresim_core::skill::Skill::TerrainSnow,
        ]
        .into_iter()
        .enumerate()
        {
            let hours = direct_hours(skill)
                + skill
                    .ordinary_correlations()
                    .iter()
                    .map(|(source, coefficient)| direct_hours(*source) * coefficient)
                    .sum::<f32>();
            checks[index].push(terrain_mental_check(
                skill.training_rank(hours),
                attributes.intelligence,
                limbs.head_health,
            ));
        }
    }
    let aggregate = |values: &[f32]| {
        (adventuresim_core::capability::aggregate_bounded_party_check(values.iter().copied())
            .clamp(0.0, 5.0)
            * 1_000.0)
            .round() as u16
    };
    Ok((
        adventuresim_terrain::TerrainSkillProfile {
            plains: aggregate(&checks[0]),
            forest: aggregate(&checks[1]),
            hills: aggregate(&checks[2]),
            wetlands: aggregate(&checks[3]),
            urban: aggregate(&checks[4]),
        },
        aggregate(&checks[5]),
    ))
}

pub(in crate::routes) fn terrain_mental_check(
    training_rank: f32,
    intelligence: f32,
    head_health: f32,
) -> f32 {
    let head_health = head_health.clamp(0.0, 1.0);
    training_rank.min(intelligence.clamp(0.0, 5.0)) * head_health
}

pub(super) async fn authoritative_party_departure_minute(
    state: &AppState,
    actor: &CharacterView,
) -> std::result::Result<StrategicMinute, PartyDepartureError> {
    let member_ids = if let Some(party_id) = actor.party_id.as_deref() {
        state
            .db
            .query_sats::<PartyMember>(SqlQuery::from(format!(
                "SELECT * FROM party_member WHERE party_id = {}",
                sql_string_literal(party_id)
            )))
            .await
            .map_err(|source: SpacetimeError| -> PartyDepartureError {
                PartyDepartureError::query(DepartureQueryStage::Membership, None, source)
            })?
            .into_iter()
            .map(|member: PartyMember| -> CharacterId { CharacterId::from(member.character_id) })
            .collect::<Vec<_>>()
    } else {
        vec![CharacterId::from(actor.id)]
    };
    let mut departure = StrategicMinute::ZERO;
    for id in member_ids {
        let living = character_as_observed(state, id, actor.id.into())
            .await
            .map_err(|source: SpacetimeError| -> PartyDepartureError {
                PartyDepartureError::query(DepartureQueryStage::ObservedMember, Some(id), source)
            })?
            .is_some_and(|character| character.alive);
        if !living {
            continue;
        }
        if let Some(time) = state
            .db
            .query_one_sats::<CharacterTime>(db::character_time_by_character_id(id))
            .await
            .map_err(|source: SpacetimeError| -> PartyDepartureError {
                PartyDepartureError::query(DepartureQueryStage::MemberClock, Some(id), source)
            })?
        {
            departure = departure.max(StrategicMinute::new(time.minutes.minutes));
        }
    }
    Ok(departure)
}

struct TerrainMemberInputs {
    attributes: CharacterAttributes,
    limbs: CharacterLimbs,
    skills: CharacterSkills,
}
impl TerrainMemberInputs {
    async fn load(
        state: &AppState,
        id: CharacterId,
        observer: CharacterId,
    ) -> std::result::Result<Option<Self>, PartyTerrainProfileError> {
        let Some(character) = character_as_observed(state, id, observer).await.map_err(
            |source: SpacetimeError| -> PartyTerrainProfileError {
                PartyTerrainProfileError::query(
                    TerrainProfileStage::ObservedMember,
                    Some(id),
                    source,
                )
            },
        )?
        else {
            return Ok(None);
        };
        if !character.alive {
            return Ok(None);
        }
        let Some(attributes) = state
            .db
            .query_one_sats::<CharacterAttributes>(db::character_attributes_by_character_id(id))
            .await
            .map_err(|source: SpacetimeError| -> PartyTerrainProfileError {
                PartyTerrainProfileError::query(TerrainProfileStage::Attributes, Some(id), source)
            })?
        else {
            return Ok(None);
        };
        let Some(limbs) = state
            .db
            .query_one_sats::<CharacterLimbs>(db::character_limbs_by_character_id(id))
            .await
            .map_err(|source: SpacetimeError| -> PartyTerrainProfileError {
                PartyTerrainProfileError::query(TerrainProfileStage::Limbs, Some(id), source)
            })?
        else {
            return Ok(None);
        };
        let Some(skills) = state
            .db
            .query_one_sats::<CharacterSkills>(db::character_skills_by_character_id(id))
            .await
            .map_err(|source: SpacetimeError| -> PartyTerrainProfileError {
                PartyTerrainProfileError::query(TerrainProfileStage::Skills, Some(id), source)
            })?
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            attributes,
            limbs,
            skills,
        }))
    }
}
