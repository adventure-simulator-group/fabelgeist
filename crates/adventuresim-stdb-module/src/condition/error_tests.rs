//! Condition error integration retains actual persisted episode admission.

use super::error::*;
use crate::capability::CapabilityEvaluationError;
use crate::disease::{EpisodeDecodeError, InfectionEpisodeRow};
use adventuresim_core::{
    disease::{InfectionEpisode, ParseDiseaseIdError},
    identity::InfectionEpisodeId,
    physiology,
};
use adventuresim_world_schema::calendar::StrategicMinute;
use std::error::Error;

#[test]
fn condition_projection_retains_episode_admission_context_through_capability() {
    let stored = InfectionEpisodeRow {
        id: 53,
        character_id: 71,
        disease_id: "ShroudFever".into(),
        contracted_at: StrategicMinute::new(109),
        ruleset_version: physiology::PHYSIOLOGY_RULESET_VERSION,
        phenotype_key_version: physiology::PHENOTYPE_KEY_VERSION,
    };
    let source = InfectionEpisode::try_from(&stored).unwrap_err();
    let condition = StrategicConditionError::from(CapabilityEvaluationError::from(source));
    let capability = condition
        .source()
        .unwrap()
        .downcast_ref::<CapabilityEvaluationError>()
        .unwrap();
    let episode = capability
        .source()
        .unwrap()
        .downcast_ref::<EpisodeDecodeError>()
        .unwrap();
    assert!(matches!(episode, EpisodeDecodeError::DiseaseKey {
            episode, stored_key, ..
        } if *episode == InfectionEpisodeId::new(53) && stored_key == "ShroudFever"));
    assert_eq!(
        episode
            .source()
            .unwrap()
            .downcast_ref::<ParseDiseaseIdError>(),
        Some(&ParseDiseaseIdError)
    );
    assert_eq!(condition.to_string(), "Unknown disease");
    assert_eq!(stored.disease_id, "ShroudFever");
}
