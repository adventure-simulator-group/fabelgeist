//! Same-minute admission and concrete failure context for a prospective father.

use crate::time::CharacterClockError;
use adventuresim_core::identity::CharacterId;
use adventuresim_world_schema::calendar::StrategicMinute;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FatherAdmissionError {
    Clock {
        child: CharacterId,
        requested: StrategicMinute,
        source: CharacterClockError,
    },
    FrontierMismatch {
        child: CharacterId,
        father: CharacterId,
        requested: StrategicMinute,
        observed: StrategicMinute,
    },
}

pub(super) fn require_frontier(
    child: CharacterId,
    father: CharacterId,
    requested: StrategicMinute,
    frontier: Result<StrategicMinute, CharacterClockError>,
) -> Result<(), FatherAdmissionError> {
    let observed = frontier.map_err(|source: CharacterClockError| FatherAdmissionError::Clock {
        child,
        requested,
        source,
    })?;
    if observed != requested {
        return Err(FatherAdmissionError::FrontierMismatch {
            child,
            father,
            requested,
            observed,
        });
    }
    Ok(())
}

impl std::fmt::Display for FatherAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clock { source, .. } => source.fmt(f),
            Self::FrontierMismatch { .. } => {
                f.write_str("The prospective bride's father has not reached the relationship date")
            }
        }
    }
}
impl std::error::Error for FatherAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Clock { source, .. } => Some(source),
            Self::FrontierMismatch { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn father_frontier_accepts_only_the_exact_requested_minute() {
        let child = CharacterId::from(7);
        let father = CharacterId::from(17);
        let requested = StrategicMinute::new(100);
        assert_eq!(
            require_frontier(child, father, requested, Ok(requested)),
            Ok(())
        );
        for observed in [StrategicMinute::new(99), StrategicMinute::new(101)] {
            let error = require_frontier(child, father, requested, Ok(observed)).unwrap_err();
            assert_eq!(
                error,
                FatherAdmissionError::FrontierMismatch {
                    child,
                    father,
                    requested,
                    observed,
                }
            );
            assert!(error.source().is_none());
            assert_eq!(
                error.to_string(),
                "The prospective bride's father has not reached the relationship date"
            );
        }
    }

    #[test]
    fn missing_father_clock_preserves_the_child_date_and_concrete_clock_cause() {
        let child = CharacterId::from(7);
        let father = CharacterId::from(17);
        let requested = StrategicMinute::new(100);
        let error = require_frontier(
            child,
            father,
            requested,
            Err(CharacterClockError { character: father }),
        )
        .unwrap_err();
        assert!(matches!(error, FatherAdmissionError::Clock {
            child: actual_child, requested: actual_date, ..
        } if actual_child == child && actual_date == requested));
        let cause = error
            .source()
            .unwrap()
            .downcast_ref::<CharacterClockError>()
            .unwrap();
        assert_eq!(cause.character, father);
        assert_eq!(error.to_string(), "Character time record not found");
    }
}
