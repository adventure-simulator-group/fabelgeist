//! Presentation clock shared by procedural owners and deterministic tools.

use adventuresim_tactical_core::animation::AnimationElapsed;
pub(in crate::animation) use adventuresim_tactical_core::animation::AnimationSampleTick;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct FixedAnimationStep {
    pub(crate) tick: AnimationSampleTick,
    pub(crate) delta_seconds: AnimationElapsed,
}

/// Optional deterministic clock for tools that render the same simulation
/// tick more than once. Gameplay leaves the override unset and advances from
/// Bevy's render delta.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct ProceduralAnimationClock {
    pub(crate) fixed_tick: Option<FixedAnimationStep>,
}

impl ProceduralAnimationClock {
    pub(crate) fn set_fixed_step(
        &mut self,
        tick: AnimationSampleTick,
        delta_seconds: AnimationElapsed,
    ) {
        self.fixed_tick = Some(FixedAnimationStep {
            tick,
            delta_seconds,
        });
    }

    pub(crate) fn fixed_step(&self) -> Option<FixedAnimationStep> {
        self.fixed_tick
    }
}
