//! Checked linear budgets for the terrain target followers.

use super::{LocomotionSettleState, ik_tuning};
use adventuresim_core::linear_speed::NonnegativeSpeed;
use adventuresim_tactical_core::animation::AnimationValueResult;

#[derive(Clone, Copy)]
pub(super) struct FootTargetSpeeds {
    pub(super) release: NonnegativeSpeed,
    pub(super) run_airborne: NonnegativeSpeed,
    first_release: NonnegativeSpeed,
    raised_settle: NonnegativeSpeed,
}

impl FootTargetSpeeds {
    pub(super) fn configured() -> AnimationValueResult<Self> {
        let tuning = ik_tuning();
        let speed = |step: f32| {
            NonnegativeSpeed::from_metres_per_second(step * tuning.continuity_sample_hz.hertz())
        };
        Ok(Self {
            release: speed(tuning.airborne_release_step_metres.metres())?,
            run_airborne: speed(tuning.run_airborne_owner_step_metres.metres())?,
            first_release: speed(tuning.run_first_release_owner_step_metres.metres())?,
            raised_settle: speed(tuning.raised_settle_step_metres.metres())?,
        })
    }

    pub(super) fn run_airborne(self, just_released: bool) -> NonnegativeSpeed {
        if just_released {
            self.first_release
        } else {
            self.run_airborne
        }
    }

    pub(super) fn run_sample(
        self,
        just_released: bool,
        settle_cancelled_for_restart: bool,
    ) -> NonnegativeSpeed {
        if settle_cancelled_for_restart {
            self.release
        } else {
            self.run_airborne(just_released)
        }
    }

    pub(super) fn settle(self, settle: LocomotionSettleState) -> NonnegativeSpeed {
        if settle.raised_handoff {
            self.raised_settle
        } else {
            self.release
        }
    }
}
