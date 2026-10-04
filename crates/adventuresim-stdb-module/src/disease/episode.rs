//! Admit persisted episode vocabulary and versions without erasing their causes.

use super::{EpisodeDecodeError, InfectionEpisodeRow, infection_episode};
use adventuresim_core::{
    disease::{DiseaseId, InfectionEpisode},
    identity::{CharacterId, InfectionEpisodeId},
    physiology,
};
use spacetimedb::ReducerContext;

impl TryFrom<&InfectionEpisodeRow> for InfectionEpisode {
    type Error = EpisodeDecodeError;

    fn try_from(row: &InfectionEpisodeRow) -> Result<Self, Self::Error> {
        let episode = InfectionEpisodeId::new(row.id);
        if row.ruleset_version != physiology::PHYSIOLOGY_RULESET_VERSION {
            return Err(EpisodeDecodeError::RulesetVersion {
                episode,
                stored_version: row.ruleset_version,
            });
        }
        if row.phenotype_key_version != physiology::PHENOTYPE_KEY_VERSION {
            return Err(EpisodeDecodeError::PhenotypeKeyVersion {
                episode,
                stored_version: row.phenotype_key_version,
            });
        }
        let disease_id = row.disease_id.parse::<DiseaseId>().map_err(|source| {
            EpisodeDecodeError::DiseaseKey {
                episode,
                stored_key: row.disease_id.clone(),
                source,
            }
        })?;
        Ok(Self {
            id: row.id,
            character_id: CharacterId::from(row.character_id),
            disease_id,
            contracted_at: row.contracted_at,
            ruleset_version: row.ruleset_version,
            phenotype_key_version: row.phenotype_key_version,
        })
    }
}

pub(crate) fn character_episodes(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<Vec<InfectionEpisode>, EpisodeDecodeError> {
    ctx.db
        .infection_episode()
        .character_id()
        .filter(u64::from(character_id))
        .map(|row| InfectionEpisode::try_from(&row))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_core::disease::ParseDiseaseIdError;
    use adventuresim_world_schema::calendar::StrategicMinute;
    use std::error::Error;

    #[test]
    fn persisted_episode_admits_every_authored_key_without_changing_coordinates() {
        for definition in adventuresim_core::disease::STARTER_DISEASES {
            let row = InfectionEpisodeRow {
                id: 17,
                character_id: 23,
                disease_id: definition.id.stable_id().into(),
                contracted_at: StrategicMinute::new(41),
                ruleset_version: physiology::PHYSIOLOGY_RULESET_VERSION,
                phenotype_key_version: physiology::PHENOTYPE_KEY_VERSION,
            };
            let decoded = InfectionEpisode::try_from(&row).unwrap();
            assert_eq!(decoded.id, row.id);
            assert_eq!(decoded.character_id, row.character_id.into());
            assert_eq!(decoded.disease_id, definition.id);
            assert_eq!(decoded.contracted_at, row.contracted_at);
            assert_eq!(decoded.ruleset_version, row.ruleset_version);
            assert_eq!(decoded.phenotype_key_version, row.phenotype_key_version);
        }
    }

    #[test]
    fn rejection_order_and_invalid_storage_context_remain_inspectable() {
        let mut row = InfectionEpisodeRow {
            id: 17,
            character_id: 23,
            disease_id: "Influenza".into(),
            contracted_at: StrategicMinute::new(41),
            ruleset_version: physiology::PHYSIOLOGY_RULESET_VERSION + 1,
            phenotype_key_version: physiology::PHENOTYPE_KEY_VERSION + 1,
        };
        let error = InfectionEpisode::try_from(&row).unwrap_err();
        assert!(matches!(error, EpisodeDecodeError::RulesetVersion {
            episode, stored_version,
        } if episode == InfectionEpisodeId::new(17) && stored_version == row.ruleset_version));
        assert_eq!(
            error.to_string(),
            "Unsupported physiology ruleset version 2"
        );
        row.ruleset_version = physiology::PHYSIOLOGY_RULESET_VERSION;
        let error = InfectionEpisode::try_from(&row).unwrap_err();
        assert!(matches!(error, EpisodeDecodeError::PhenotypeKeyVersion {
            episode, stored_version,
        } if episode == InfectionEpisodeId::new(17) && stored_version == row.phenotype_key_version));
        assert_eq!(
            error.to_string(),
            "Unsupported immutable physiology key version 2"
        );
        row.phenotype_key_version = physiology::PHENOTYPE_KEY_VERSION;
        let error = InfectionEpisode::try_from(&row).unwrap_err();
        assert!(matches!(&error, EpisodeDecodeError::DiseaseKey {
            episode, stored_key, ..
        } if *episode == InfectionEpisodeId::new(17) && stored_key == "Influenza"));
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<ParseDiseaseIdError>(),
            Some(&ParseDiseaseIdError)
        );
        assert_eq!(row.id, 17);
        assert_eq!(row.disease_id, "Influenza");
        assert_eq!(row.contracted_at, StrategicMinute::new(41));
    }
}
