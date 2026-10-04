//! Replay admission and transient command edge consumption.
use super::*;

/// Newest complete continuous-input sample accepted from the unreliable
/// channel. Wrap-aware ordering prevents a delayed packet from restoring stale
/// movement or look intent.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AuthoritativeInputTick {
    pub(super) tick: Option<InputTick>,
}

impl AuthoritativeInputTick {
    pub(crate) fn accept(&mut self, tick: InputTick) -> bool {
        if self
            .tick
            .is_some_and(|previous| !tick.is_newer_than(previous))
        {
            return false;
        }
        self.tick = Some(tick);
        true
    }
}

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AuthoritativePostureIntent {
    pub(super) facing: CameraFacingIntent,
    pub(super) last_jump_sequence: JumpSequence,
    pub(super) last_command_sequence: PostureSequence,
}

impl AuthoritativePostureIntent {
    pub(crate) fn consume_jump(&mut self, sequence: JumpSequence) -> bool {
        if !sequence.is_newer_than(self.last_jump_sequence) {
            return false;
        }
        // Consume even when incapacitated or airborne, preventing a repeated
        // command from becoming a stale jump after recovery or landing.
        self.last_jump_sequence = sequence;
        true
    }
    pub(crate) fn consume_posture(&mut self, sequence: PostureSequence) -> bool {
        if !sequence.is_newer_than(self.last_command_sequence) {
            return false;
        }
        self.last_command_sequence = sequence;
        true
    }
}
