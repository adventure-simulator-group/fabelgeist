//! Authoritative locomotion observations emitted as bounded presentation events.

use super::*;
use adventuresim_tactical_core::animation::{
    AnimationSampleTick, ContactSequence, LandingSequence,
};
use std::fmt;

#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct LocomotionPresentationEvent {
    pub owner: Entity,
    /// Authoritative sample tick of observation, not reconstructed contact time.
    pub sample_tick: AnimationSampleTick,
    pub payload: LocomotionPresentationPayload,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub(super) struct LocomotionEventCursor {
    observed: Option<ObservedSequences>,
}

#[derive(Debug, Clone, Copy)]
struct ObservedSequences {
    contact: ContactSequence,
    landing: LandingSequence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::animation) struct CoalescedContact {
    pub sequence: ContactSequence,
    pub foot: LeadFoot,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocomotionPresentationPayload {
    Contact {
        sequence: ContactSequence,
        foot: LeadFoot,
    },
    Landing {
        sequence: LandingSequence,
    },
}

impl LocomotionPresentationPayload {
    /// Native scalar projection for tracing, flat report serialization or audio seeds.
    pub(crate) fn encoded_sequence(self) -> u64 {
        match self {
            Self::Contact { sequence, .. } => sequence.encoded(),
            Self::Landing { sequence } => sequence.encoded(),
        }
    }

    pub(crate) fn is_contact(self) -> bool {
        matches!(self, Self::Contact { .. })
    }

    pub(crate) fn same_stream(self, previous: Self) -> bool {
        self.is_contact() == previous.is_contact()
    }

    pub(crate) fn advances_from(self, previous: Self) -> bool {
        match (self, previous) {
            (
                Self::Contact { sequence, .. },
                Self::Contact {
                    sequence: earlier, ..
                },
            ) => sequence
                .events_since(earlier)
                .is_some_and(|delta| delta > 0),
            (Self::Landing { sequence }, Self::Landing { sequence: earlier }) => sequence
                .events_since(earlier)
                .is_some_and(|delta| delta > 0),
            _ => false,
        }
    }

    pub(crate) fn report_kind(self) -> &'static str {
        match self {
            Self::Contact {
                foot: LeadFoot::Left,
                ..
            } => "contact_left",
            Self::Contact {
                foot: LeadFoot::Right,
                ..
            } => "contact_right",
            Self::Landing { .. } => "landing",
        }
    }
}

impl fmt::Debug for LocomotionPresentationPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contact { foot, .. } => formatter.debug_tuple("Contact").field(foot).finish(),
            Self::Landing { .. } => formatter.write_str("Landing"),
        }
    }
}

pub(in crate::animation) fn latest_coalesced_landing(
    previous: LandingSequence,
    current: LandingSequence,
) -> Option<LandingSequence> {
    current
        .events_since(previous)
        .filter(|delta| *delta > 0 && *delta <= maximum_coalesced_presentation_events())
        .map(|_| current)
}

pub(in crate::animation) fn coalesced_contacts(
    previous: ContactSequence,
    current: ContactSequence,
    latest_foot: LeadFoot,
) -> Option<Vec<CoalescedContact>> {
    let delta = current
        .events_since(previous)
        .filter(|delta| *delta <= maximum_coalesced_presentation_events())?;
    (0..delta)
        .rev()
        .map(|offset| {
            let foot = if offset % 2 == 0 {
                latest_foot
            } else {
                match latest_foot {
                    LeadFoot::Left => LeadFoot::Right,
                    LeadFoot::Right => LeadFoot::Left,
                }
            };
            Some(CoalescedContact {
                sequence: current.preceding(offset)?,
                foot,
            })
        })
        .collect()
}

pub(in crate::animation) fn trace_locomotion_presentation_events(
    mut messages: MessageReader<LocomotionPresentationEvent>,
) {
    for message in messages.read() {
        trace!(
            owner = ?message.owner,
            sequence = message.payload.encoded_sequence(),
            sample_tick = message.sample_tick.encoded(),
            kind = ?message.payload,
            "locomotion presentation event"
        );
    }
}

pub(super) fn emit_locomotion_presentation_events(
    mut commands: Commands,
    mut messages: MessageWriter<LocomotionPresentationEvent>,
    mut skeletons: Query<(Entity, &SkeletonState, Option<&mut LocomotionEventCursor>)>,
) {
    for (owner, skeleton, cursor) in &mut skeletons {
        let mut next = cursor.as_deref().copied().unwrap_or_default();
        if let Some(observed) = next.observed {
            if let Some(contacts) = coalesced_contacts(
                observed.contact,
                skeleton.contact_sequence,
                skeleton.contact_foot,
            ) {
                for CoalescedContact { sequence, foot } in contacts {
                    messages.write(LocomotionPresentationEvent {
                        owner,
                        sample_tick: skeleton.locomotion_sample_tick,
                        payload: LocomotionPresentationPayload::Contact { sequence, foot },
                    });
                }
            }
            if let Some(sequence) =
                latest_coalesced_landing(observed.landing, skeleton.landing_sequence)
            {
                messages.write(LocomotionPresentationEvent {
                    owner,
                    sample_tick: skeleton.locomotion_sample_tick,
                    payload: LocomotionPresentationPayload::Landing { sequence },
                });
            }
        }
        next.observed = Some(ObservedSequences {
            contact: skeleton.contact_sequence,
            landing: skeleton.landing_sequence,
        });
        if let Some(mut cursor) = cursor {
            *cursor = next;
        } else {
            commands.entity(owner).insert(next);
        }
    }
}

fn maximum_coalesced_presentation_events() -> u64 {
    playback_tuning().maximum_coalesced_events
}
