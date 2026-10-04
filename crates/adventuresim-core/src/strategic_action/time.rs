//! Requested duration clipping at terminal, interruption and clock boundaries.
use super::{DomainInterruption, RequestedDuration};
use adventuresim_world_schema::calendar::{StrategicDuration, StrategicMinute};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledInterruption<I: DomainInterruption> {
    pub at_minute: StrategicMinute,
    pub cause: I,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeBoundaries<I: DomainInterruption> {
    pub terminal_minute: Option<StrategicMinute>,
    pub interruption: Option<ScheduledInterruption<I>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeOutcome<I: DomainInterruption> {
    Completed,
    TerminalBoundary,
    Interrupted(I),
    /// The requested positive duration cannot fit in the strategic clock.
    ClockExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeResolution<I: DomainInterruption> {
    pub start_minute: StrategicMinute,
    pub requested_minutes: StrategicDuration,
    pub elapsed_minutes: StrategicDuration,
    pub end_minute: StrategicMinute,
    pub outcome: TimeOutcome<I>,
}

impl<I: DomainInterruption> TimeResolution<I> {
    /// Only this outcome permits a domain to emit completion-only effects.
    pub const fn permits_completion_effects(&self) -> bool {
        matches!(self.outcome, TimeOutcome::Completed)
    }
}

pub fn resolve_time<I: DomainInterruption>(
    current_minute: StrategicMinute,
    duration: RequestedDuration,
    boundaries: &TimeBoundaries<I>,
) -> TimeResolution<I> {
    let requested_end = current_minute.checked_add_minutes(duration.minutes());
    let latest_representable_end = requested_end.unwrap_or(StrategicMinute::MAX);
    let terminal = boundaries
        .terminal_minute
        .map(|minute| minute.max(current_minute));
    let interruption = boundaries
        .interruption
        .as_ref()
        .map(|value| (value.at_minute.max(current_minute), value.cause.clone()));
    let (end_minute, outcome) = match (terminal, interruption) {
        (Some(terminal), Some((at, cause))) if at < terminal && at <= latest_representable_end => {
            (at, TimeOutcome::Interrupted(cause))
        }
        (Some(terminal), _) if terminal <= latest_representable_end => {
            (terminal, TimeOutcome::TerminalBoundary)
        }
        (None, Some((at, cause))) if at <= latest_representable_end => {
            (at, TimeOutcome::Interrupted(cause))
        }
        _ => match requested_end {
            Some(end) if end == StrategicMinute::MAX => (end, TimeOutcome::ClockExhausted),
            Some(end) => (end, TimeOutcome::Completed),
            None => (StrategicMinute::MAX, TimeOutcome::ClockExhausted),
        },
    };
    TimeResolution {
        start_minute: current_minute,
        requested_minutes: StrategicDuration::new(duration.minutes()),
        elapsed_minutes: StrategicDuration::new(end_minute.elapsed_since(current_minute)),
        end_minute,
        outcome,
    }
}
