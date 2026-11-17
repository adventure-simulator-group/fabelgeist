//! The browser can open dialogue only for the current accepted approach.

use adventuresim_core::dialogue_boundary::DialogueSessionId;
use adventuresim_tactical_core::{control::*, player::CharacterId};
use adventuresim_tactical_netcode::{
    bevy_replicon::prelude::ClientTriggerExt,
    control::{ControlChange, LocalControl},
    dialogue::*,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    hud::{self, HudView},
    player::ClientPlayer,
};

type LocalPlayers<'w, 's> = Query<'w, 's, (Entity, &'static CharacterId), With<ClientPlayer>>;

#[derive(Resource, Default)]
struct LocalDialogue(Option<ConfirmedDialogue>);

struct ConfirmedDialogue {
    actor: Entity,
    session: DialogueSessionId,
    generation: ControlGeneration,
    sequence: OrderSequence,
}

/// Typed HTML command; the native client supplies the control fence itself.
#[derive(Event, Debug, Deserialize)]
pub(crate) struct BrowserDialogueRequest {
    session_id: DialogueSessionId,
    action: DialogueAction,
}

pub(crate) struct ClientDialoguePlugin;

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum DialogueNotice<'a> {
    Opened {
        session_id: &'a DialogueSessionId,
        resident_id: String,
    },
    Closed {
        session_id: &'a DialogueSessionId,
    },
    Action {
        session_id: &'a DialogueSessionId,
        action_id: &'a str,
        result: &'a Result<(), String>,
    },
    Unavailable {
        message: &'a str,
    },
}

enum ClosureOrigin {
    Local,
    Host,
}

impl Plugin for ClientDialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LocalDialogue>()
            .init_resource::<LocalControl>()
            .add_observer(on_start)
            .add_observer(on_action)
            .add_observer(on_ended)
            .add_observer(on_browser_request)
            .add_systems(
                Update,
                invalidate.after(crate::indirect_control::IndirectControlSet),
            );
    }
}

impl ConfirmedDialogue {
    fn is_current(&self, local: &LocalControl, players: &LocalPlayers) -> bool {
        accepts(local, self.generation, self.sequence)
            && players
                .get(self.actor)
                .is_ok_and(|(_, id)| id.0 == self.session.owner_character_id())
    }

    fn request(&self, action: DialogueAction) -> DialogueRequest {
        DialogueRequest {
            generation: self.generation,
            sequence: self.sequence,
            session_id: self.session.clone(),
            action,
        }
    }
}

impl LocalDialogue {
    fn close(&mut self, commands: &mut Commands, origin: ClosureOrigin) {
        let Some(active) = self.0.take() else {
            return;
        };
        if matches!(origin, ClosureOrigin::Local) {
            commands.client_trigger(active.request(DialogueAction::Close));
        }
        publish(&DialogueNotice::Closed {
            session_id: &active.session,
        });
    }
}

fn accepts(local: &LocalControl, generation: ControlGeneration, sequence: OrderSequence) -> bool {
    local.pending.is_none()
        && local.rejection.is_none()
        && local.acknowledged.mode == ControlMode::Indirect
        && local.acknowledged.generation == generation
        && local.sequence == sequence
}

fn on_start(
    response: On<DialogueStartResponse>,
    local: Res<LocalControl>,
    players: LocalPlayers,
    mut dialogue: ResMut<LocalDialogue>,
    mut commands: Commands,
) {
    if !accepts(&local, response.generation, response.sequence)
        || !players
            .get(response.controlled_actor)
            .is_ok_and(|(_, id)| *id == response.actor)
    {
        return;
    }
    let session = match &response.result {
        Ok(session) if session.owner_character_id() == response.actor.0 => session,
        Ok(_) => return,
        Err(message) => {
            publish(&DialogueNotice::Unavailable { message });
            return;
        }
    };
    if dialogue
        .0
        .as_ref()
        .is_some_and(|active| active.session == *session)
    {
        return;
    }
    dialogue.close(&mut commands, ClosureOrigin::Local);
    dialogue.0 = Some(ConfirmedDialogue {
        actor: response.controlled_actor,
        session: session.clone(),
        generation: response.generation,
        sequence: response.sequence,
    });
    publish(&DialogueNotice::Opened {
        session_id: session,
        resident_id: response.resident.0.to_string(),
    });
}

fn on_action(
    response: On<DialogueActionResponse>,
    local: Res<LocalControl>,
    dialogue: Res<LocalDialogue>,
    players: LocalPlayers,
) {
    if dialogue.0.as_ref().is_none_or(|active| {
        active.session != response.session_id || !active.is_current(&local, &players)
    }) {
        return;
    }
    publish(&DialogueNotice::Action {
        session_id: &response.session_id,
        action_id: &response.action_id,
        result: &response.result,
    });
}

fn on_ended(ended: On<DialogueEnded>, mut dialogue: ResMut<LocalDialogue>, mut commands: Commands) {
    if dialogue
        .0
        .as_ref()
        .is_some_and(|active| active.session == ended.session_id)
    {
        dialogue.close(&mut commands, ClosureOrigin::Host);
    }
}

fn on_browser_request(
    request: On<BrowserDialogueRequest>,
    mut local: ResMut<LocalControl>,
    players: LocalPlayers,
    mut dialogue: ResMut<LocalDialogue>,
    mut commands: Commands,
) {
    let Some(active) = &dialogue.0 else {
        return;
    };
    if active.session != request.session_id {
        return;
    }
    if request.action == DialogueAction::Close {
        let current = active.is_current(&local, &players);
        dialogue.close(&mut commands, ClosureOrigin::Local);
        // Ending conversation also ends the retained Talk goal. Advance the
        // ordinary control fence so late arrivals cannot reopen this session.
        if current {
            commands.client_trigger(local.request(ControlChange::Stop));
        }
        return;
    }
    if !active.is_current(&local, &players) {
        dialogue.close(&mut commands, ClosureOrigin::Local);
        return;
    }
    commands.client_trigger(active.request(request.action.clone()));
}

fn invalidate(
    local: Res<LocalControl>,
    players: LocalPlayers,
    mut dialogue: ResMut<LocalDialogue>,
    mut commands: Commands,
) {
    if dialogue
        .0
        .as_ref()
        .is_some_and(|active| !active.is_current(&local, &players))
    {
        dialogue.close(&mut commands, ClosureOrigin::Local);
    }
}

fn publish(notice: &DialogueNotice<'_>) {
    match serde_json::to_string(notice) {
        Ok(json) => hud::view(HudView::Dialogue, &json),
        Err(error) => warn!(%error, "Could not encode dialogue notice"),
    }
}

#[cfg(test)]
mod tests;
