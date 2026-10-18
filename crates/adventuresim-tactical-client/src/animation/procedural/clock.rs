//! Presentation clock shared by procedural owners and deterministic tools.

pub(in crate::animation) use adventuresim_tactical_core::animation::AnimationSampleTick;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct FixedAnimationStep {
    pub(in crate::animation) tick: AnimationSampleTick,
    pub(crate) delta_seconds: f32,
}

/// Optional deterministic clock for tools that render the same simulation
/// tick more than once. Gameplay leaves the override unset and advances from
/// Bevy's render delta.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct ProceduralAnimationClock {
    pub(crate) fixed_tick: Option<(u64, f32)>,
}

impl ProceduralAnimationClock {
    pub(crate) fn fixed_step(&self) -> Option<FixedAnimationStep> {
        self.fixed_tick
            .map(|(tick, delta_seconds)| FixedAnimationStep {
                tick: tick.into(),
                delta_seconds,
            })
    }
}
