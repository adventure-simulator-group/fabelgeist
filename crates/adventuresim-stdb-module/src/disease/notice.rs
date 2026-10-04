//! Typed disease notice vocabulary and the admitted illness-status projection.

use super::*;
use adventuresim_core::identity::{CharacterId, InfectionEpisodeId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DiseaseNoticeKind {
    SymptomOnset,
    Critical,
    Resolution,
}

impl DiseaseNoticeKind {
    fn stored_kind(self) -> &'static str {
        match self {
            Self::SymptomOnset => "symptom-onset",
            Self::Critical => "critical",
            Self::Resolution => "resolution",
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::SymptomOnset => "New symptoms have appeared.",
            Self::Critical => "A vital humour is failing.",
            Self::Resolution => "The illness's visible effects have resolved.",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DiseaseNoticeSource {
    Episode(InfectionEpisodeId),
    TerminalFailure,
}

impl DiseaseNoticeSource {
    /// Encode the existing native deduplication key. Terminal aggregate failure
    /// has the separate zero coordinate; it is not an infection episode.
    fn encode_notice_id(self, minute: StrategicMinute, kind: DiseaseNoticeKind) -> String {
        let coordinate = match self {
            Self::Episode(episode) => episode.get(),
            Self::TerminalFailure => 0,
        };
        format!("disease-{coordinate}-{minute}-{}", kind.stored_kind())
    }
}

pub(super) fn notice(
    ctx: &ReducerContext,
    character: CharacterId,
    source: DiseaseNoticeSource,
    minute: StrategicMinute,
    kind: DiseaseNoticeKind,
) -> Result<(), EpisodeDecodeError> {
    let id = source.encode_notice_id(minute, kind);
    if ctx.db.disease_notice().id().find(&id).is_none() {
        ctx.db.disease_notice().insert(DiseaseNotice {
            id,
            character_id: u64::from(character),
            minute,
            kind: kind.stored_kind().into(),
            message: kind.message().into(),
        });
    }
    let immunity = ctx
        .db
        .character_attributes()
        .character_id()
        .find(u64::from(character))
        .map_or(3.0, |attributes| attributes.immunity);
    let states = character_episodes(ctx, character)?
        .into_iter()
        .map(|episode| disease::evaluate(episode, minute, immunity))
        .collect::<Vec<_>>();
    let symptomatic = states.iter().any(|state| {
        !matches!(
            state.stage,
            disease::DiseaseStage::Incubating | disease::DiseaseStage::Resolved
        )
    });
    let critical = states
        .iter()
        .any(|state| state.stage == disease::DiseaseStage::Critical);
    let row = CharacterIllnessStatus {
        character_id: u64::from(character),
        symptomatic,
        critical,
        updated_at_minute: minute,
    };
    if ctx
        .db
        .character_illness_status()
        .character_id()
        .find(u64::from(character))
        .is_some()
    {
        ctx.db.character_illness_status().character_id().update(row);
    } else {
        ctx.db.character_illness_status().insert(row);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn episode_and_terminal_notices_keep_their_distinct_native_coordinates() {
        let episode = DiseaseNoticeSource::Episode(InfectionEpisodeId::new(17));
        let minute = StrategicMinute::new(41);
        assert_eq!(
            episode.encode_notice_id(minute, DiseaseNoticeKind::SymptomOnset),
            "disease-17-41-symptom-onset"
        );
        assert_eq!(
            episode.encode_notice_id(minute, DiseaseNoticeKind::Critical),
            "disease-17-41-critical"
        );
        assert_eq!(
            episode.encode_notice_id(minute, DiseaseNoticeKind::Resolution),
            "disease-17-41-resolution"
        );
        assert_eq!(
            DiseaseNoticeSource::TerminalFailure
                .encode_notice_id(minute, DiseaseNoticeKind::Critical),
            "disease-0-41-critical"
        );
    }
}
