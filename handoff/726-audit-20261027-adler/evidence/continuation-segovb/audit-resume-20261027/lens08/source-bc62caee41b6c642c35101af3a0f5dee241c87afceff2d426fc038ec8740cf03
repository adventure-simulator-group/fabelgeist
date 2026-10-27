//! Held attack observation, preparation admission and input publication.

use super::{DebugForceAttackTrigger, DirectControlState, MeleePreparationInput};
use adventuresim_tactical_core::{
    animation::{AnimationElapsed, AnimationValueResult, AttackHand, UnitInterval},
    combat_config::ClientInputConfig,
};
use bevy::prelude::{ButtonInput, MouseButton};

pub(super) struct AttackInputObservation<'a> {
    pub(super) mouse: &'a ButtonInput<MouseButton>,
    pub(super) left_trigger: bool,
    pub(super) left_trigger_value: f32,
    pub(super) controller_attack: bool,
    pub(super) forced_attack: bool,
    pub(super) elapsed: AnimationElapsed,
}

struct HeldAttack {
    armed: bool,
    elapsed: AnimationElapsed,
}

struct MeleePreparation {
    from: MeleePreparationInput,
    to: MeleePreparationInput,
    progress: UnitInterval,
    selected: MeleePreparationInput,
}

pub(super) struct PreparedAttackInput {
    main: HeldAttack,
    offhand: HeldAttack,
    main_released: bool,
    offhand_released: bool,
    requested: bool,
    alternate: bool,
    hand: AttackHand,
    preparation: MeleePreparation,
}

impl HeldAttack {
    fn observed(
        mut self,
        mouse: &ButtonInput<MouseButton>,
        button: MouseButton,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<Self> {
        if mouse.pressed(MouseButton::Right) && mouse.just_pressed(button) {
            self.armed = true;
            self.elapsed = AnimationElapsed::ZERO;
        }
        if self.armed && mouse.pressed(button) {
            self.elapsed = self.elapsed.checked_add(elapsed)?;
        }
        Ok(self)
    }
}

impl DirectControlState {
    pub(super) fn prepare_attack_input(
        &self,
        observation: AttackInputObservation<'_>,
        config: &ClientInputConfig,
    ) -> AnimationValueResult<PreparedAttackInput> {
        let main = HeldAttack {
            armed: self.mouse_main_attack_armed,
            elapsed: self.mouse_main_attack_seconds,
        }
        .observed(observation.mouse, MouseButton::Left, observation.elapsed)?;
        let offhand = HeldAttack {
            armed: self.mouse_offhand_attack_armed,
            elapsed: self.mouse_offhand_attack_seconds,
        }
        .observed(observation.mouse, MouseButton::Middle, observation.elapsed)?;
        let main_released = main.armed && observation.mouse.just_released(MouseButton::Left);
        let offhand_released =
            offhand.armed && observation.mouse.just_released(MouseButton::Middle);
        let mouse_alternate =
            main_released && main.elapsed > config.alternate_attack_hold_seconds.elapsed();
        let controller_preparation =
            if observation.left_trigger_value >= config.aim_trigger_threshold {
                MeleePreparationInput::Preferred
            } else {
                MeleePreparationInput::Alternate
            };
        let preparation = if offhand.armed && observation.mouse.pressed(MouseButton::Middle) {
            MeleePreparation::held(&offhand, MeleePreparationInput::Offhand, config)?
        } else if main.armed && observation.mouse.pressed(MouseButton::Left) {
            MeleePreparation::held(&main, MeleePreparationInput::Alternate, config)?
        } else {
            let selected = if observation.left_trigger {
                controller_preparation
            } else {
                MeleePreparationInput::Preferred
            };
            MeleePreparation {
                from: selected,
                to: selected,
                progress: UnitInterval::ONE,
                selected,
            }
        };
        Ok(PreparedAttackInput {
            main,
            offhand,
            main_released,
            offhand_released,
            requested: main_released
                || offhand_released
                || observation.controller_attack
                || observation.forced_attack,
            alternate: mouse_alternate
                || (observation.controller_attack
                    && observation.left_trigger_value < config.aim_trigger_threshold),
            hand: if offhand_released {
                AttackHand::Offhand
            } else {
                AttackHand::Main
            },
            preparation,
        })
    }
}

impl MeleePreparation {
    fn held(
        held: &HeldAttack,
        target: MeleePreparationInput,
        config: &ClientInputConfig,
    ) -> AnimationValueResult<Self> {
        // The complete ratio may overflow for finite inputs and still clamp to
        // valid completed progress; only its final bounded result is admitted.
        let progress = UnitInterval::from_fraction(
            (held.elapsed.seconds() / config.alternate_attack_hold_seconds.seconds())
                .clamp(0.0, 1.0),
        )?;
        Ok(Self {
            from: MeleePreparationInput::Preferred,
            to: target,
            progress,
            selected: if held.elapsed > config.alternate_attack_hold_seconds.elapsed() {
                target
            } else {
                MeleePreparationInput::Preferred
            },
        })
    }
}

impl PreparedAttackInput {
    pub(super) fn publish(
        self,
        controls: &mut DirectControlState,
        forced: &mut DebugForceAttackTrigger,
    ) {
        controls.mouse_main_attack_armed = self.main.armed;
        controls.mouse_main_attack_seconds = self.main.elapsed;
        controls.mouse_offhand_attack_armed = self.offhand.armed;
        controls.mouse_offhand_attack_seconds = self.offhand.elapsed;
        controls.attack_just_pressed = self.requested;
        forced.0 = false;
        controls.alternate_attack = self.alternate;
        controls.attack_hand = self.hand;
        controls.local_preparation_from = self.preparation.from;
        controls.local_preparation_to = self.preparation.to;
        controls.local_preparation_progress = self.preparation.progress;
        controls.melee_preparation = self.preparation.selected;
        if self.main_released {
            controls.mouse_main_attack_armed = false;
            controls.mouse_main_attack_seconds = AnimationElapsed::ZERO;
        }
        if self.offhand_released {
            controls.mouse_offhand_attack_armed = false;
            controls.mouse_offhand_attack_seconds = AnimationElapsed::ZERO;
        }
    }
}
