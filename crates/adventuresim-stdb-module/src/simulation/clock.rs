//! Bounded official-clock advancement for capability-owned disposable worlds.

use super::*;
use adventuresim_core::strategic_time::clock::{ClockEpochShiftError, OfficialClockEpoch};
use adventuresim_world_schema::calendar::{MINUTES_PER_YEAR, StrategicDuration};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SimulationClockAdvance(StrategicDuration);

struct SimulationClockProjection {
    epoch: OfficialClockEpoch,
    minute: StrategicMinute,
}

impl SimulationClockAdvance {
    const MAXIMUM: StrategicDuration = StrategicDuration::new(100 * MINUTES_PER_YEAR);

    fn try_new(requested: StrategicDuration) -> Result<Self, SimulationClockAdvanceError> {
        if requested == StrategicDuration::ZERO || requested > Self::MAXIMUM {
            return Err(SimulationClockAdvanceError::OutsideBound {
                requested,
                maximum: Self::MAXIMUM,
            });
        }
        Ok(Self(requested))
    }

    fn project(
        self,
        epoch: OfficialClockEpoch,
        minute: StrategicMinute,
    ) -> Result<SimulationClockProjection, SimulationClockAdvanceError> {
        let epoch = epoch.advance_by(self.0)?;
        Ok(SimulationClockProjection {
            epoch,
            minute: StrategicMinute::new(minute.get().saturating_add(self.0.get())),
        })
    }
}

#[derive(Debug)]
enum SimulationClockAdvanceError {
    OutsideBound {
        requested: StrategicDuration,
        maximum: StrategicDuration,
    },
    ClockNotInitialized,
    EpochShift(ClockEpochShiftError),
}

impl From<ClockEpochShiftError> for SimulationClockAdvanceError {
    fn from(source: ClockEpochShiftError) -> Self {
        Self::EpochShift(source)
    }
}

impl std::fmt::Display for SimulationClockAdvanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::OutsideBound { .. } => {
                "Simulation world-time advance is outside the bounded range"
            }
            Self::ClockNotInitialized => "Simulation world clock is not initialized",
            Self::EpochShift(ClockEpochShiftError::DurationOutOfRange { .. }) => {
                "Simulation world-time advance overflow"
            }
            Self::EpochShift(ClockEpochShiftError::EpochUnderflow { .. }) => {
                "Simulation world epoch underflow"
            }
        })
    }
}

impl std::error::Error for SimulationClockAdvanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EpochShift(source) => Some(source),
            _ => None,
        }
    }
}

#[reducer]
pub fn advance_simulation_world_time(
    ctx: &ReducerContext,
    nonce: String,
    delta_minutes: u64,
) -> Result<(), String> {
    owned_run(ctx, &nonce)?;
    let advance = SimulationClockAdvance::try_new(StrategicDuration::new(delta_minutes))
        .map_err(|error: SimulationClockAdvanceError| error.to_string())?;
    crate::time::refresh_clock(ctx)
        .map_err(|error: crate::time::WorldClockError| error.to_string())?;
    let mut clock = ctx
        .db
        .world_clock()
        .id()
        .find(0)
        .ok_or(SimulationClockAdvanceError::ClockNotInitialized)
        .map_err(|error: SimulationClockAdvanceError| error.to_string())?;
    let projection = advance
        .project(
            OfficialClockEpoch::from(clock.epoch_micros),
            clock.official_minutes,
        )
        .map_err(|error: SimulationClockAdvanceError| error.to_string())?;
    clock.epoch_micros = i64::from(projection.epoch);
    clock.official_minutes = projection.minute;
    ctx.db.world_clock().id().update(clock);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_core::strategic_time::clock::UnixMicrosecondInstant;
    use adventuresim_world_schema::calendar::WORLD_START_MINUTE;
    use std::error::Error;

    #[test]
    fn simulation_world_time_advance_is_positive_and_bounded() {
        for invalid in [
            StrategicDuration::ZERO,
            StrategicDuration::new(SimulationClockAdvance::MAXIMUM.get() + 1),
        ] {
            let error = SimulationClockAdvance::try_new(invalid).unwrap_err();
            assert!(
                matches!(error, SimulationClockAdvanceError::OutsideBound { requested, maximum } if requested==invalid && maximum==SimulationClockAdvance::MAXIMUM)
            );
        }
        assert!(SimulationClockAdvance::try_new(SimulationClockAdvance::MAXIMUM).is_ok());
        let elapsed = StrategicDuration::new(MINUTES_PER_DAY);
        let advance = SimulationClockAdvance::try_new(elapsed).unwrap();
        let projection = advance
            .project(OfficialClockEpoch::from(0), WORLD_START_MINUTE)
            .unwrap();
        assert_eq!(
            projection.epoch.elapsed_at(UnixMicrosecondInstant::from(0)),
            elapsed
        );
        assert_eq!(
            projection.minute,
            WORLD_START_MINUTE.saturating_add_minutes(MINUTES_PER_DAY)
        );
    }

    #[test]
    fn simulation_epoch_underflow_keeps_the_core_coordinates_and_message() {
        let advance = SimulationClockAdvance::try_new(StrategicDuration::new(1)).unwrap();
        let epoch = OfficialClockEpoch::from(i64::MIN);
        let error = advance.project(epoch, WORLD_START_MINUTE).err().unwrap();
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<ClockEpochShiftError>()
            .unwrap();
        assert!(
            matches!(source, ClockEpochShiftError::EpochUnderflow { epoch: failed, elapsed, .. } if *failed==epoch && *elapsed==StrategicDuration::new(1))
        );
        assert_eq!(error.to_string(), "Simulation world epoch underflow");
    }
}
