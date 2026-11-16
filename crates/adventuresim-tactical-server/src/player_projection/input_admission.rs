//! Authentication and control-generation admission precede movement execution.

use super::*;
use adventuresim_tactical_core::control::CharacterControl;

#[derive(Event, Deref)]
pub(crate) struct MovementAction {
    pub(crate) actor: Entity,
    #[deref]
    pub(crate) sample: PlayerInputRequest,
}

pub(crate) fn on_player_input(
    input: On<FromClient<PlayerInputRequest>>,
    controls: Query<&CharacterControl>,
    mut commands: Commands,
) {
    let Some(actor) = input.client_id.entity() else {
        return;
    };
    if !controls
        .get(actor)
        .is_ok_and(|control| control.accepts_direct(input.generation))
    {
        return;
    }
    commands.trigger(MovementAction {
        actor,
        sample: **input,
    });
}
