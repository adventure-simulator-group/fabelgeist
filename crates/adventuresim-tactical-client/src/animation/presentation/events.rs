//! Authoritative locomotion observations emitted as bounded presentation events.

use super::*;
use adventuresim_tactical_core::animation::{
    AnimationSampleTick, ContactSequence, LandingSequence,
};

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

#[derive(Clone, Copy, PartialEq, Eq, derive_more::Debug)]
pub(crate) enum LocomotionPresentationPayload {
    #[debug("Contact({foot:?})")]
    Contact {
        sequence: ContactSequence,
        foot: LeadFoot,
    },
    #[debug("Landing")]
    Landing { sequence: LandingSequence },
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
                .is_some_and(|delta| !delta.is_zero()),
            (Self::Landing { sequence }, Self::Landing { sequence: earlier }) => sequence
                .events_since(earlier)
                .is_some_and(|delta| !delta.is_zero()),
            _ => false,
        }
    }
}

pub(in crate::animation) fn latest_coalesced_landing(
    previous: LandingSequence,
    current: LandingSequence,
) -> Option<LandingSequence> {
    current
        .events_since(previous)
        .filter(|delta| {
            !delta.is_zero() && playback_tuning().maximum_coalesced_events.contains(*delta)
        })
        .map(|_| current)
}

pub(in crate::animation) fn coalesced_contacts(
    previous: ContactSequence,
    current: ContactSequence,
    latest_foot: LeadFoot,
) -> Option<Vec<CoalescedContact>> {
    let delta = current
        .events_since(previous)
        .filter(|delta| playback_tuning().maximum_coalesced_events.contains(*delta))?;
    delta
        .offsets()
        .rev()
        .map(|offset| {
            let foot = if offset.is_even() {
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
