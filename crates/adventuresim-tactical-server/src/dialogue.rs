//! Physical contact owns every live dialogue start, action and confirmation.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use adventuresim_core::{
    dialogue_boundary::DialogueSessionId, tactical_session::LocalSceneVisitId,
};
use adventuresim_tactical_core::control::*;
use adventuresim_tactical_netcode::{bevy_replicon::prelude::*, dialogue::*};
use bevy::prelude::*;

use crate::{
    control::{ControlSet, ReachedTalkTarget, TalkContactWorld, TalkParticipants},
    local_time::LocalVisits,
};

mod callbacks;
mod transport;

#[derive(Resource, Default)]
struct SceneDialogues {
    sessions: HashMap<Entity, LiveDialogue>,
    serial: DialogueSerial,
}

#[derive(Resource, Default)]
struct DialogueOutbox(Vec<DialogueCall>);

#[derive(Clone, Resource, Default)]
struct DialogueMailbox(Arc<Mutex<Vec<DialogueCompletion>>>);

#[derive(Clone, PartialEq, Eq)]
struct DialogueKey {
    actor: Entity,
    session: DialogueSessionId,
}

struct LiveDialogue {
    key: DialogueKey,
    reached: ReachedTalkTarget,
    participants: TalkParticipants,
    actor_visit: LocalSceneVisitId,
    resident_visit: LocalSceneVisitId,
    phase: DialoguePhase,
}

struct DialogueCompletion {
    call: DialogueCall,
    result: Result<(), String>,
}

#[derive(Default)]
struct DialogueSerial(u64);

pub(crate) struct TacticalDialoguePlugin;

#[derive(Clone)]
enum DialogueCall {
    Start {
        key: DialogueKey,
        actor_visit: LocalSceneVisitId,
        resident_visit: LocalSceneVisitId,
    },
    Action {
        key: DialogueKey,
        character: adventuresim_tactical_core::player::CharacterId,
        action: DialogueAction,
    },
    Close {
        key: DialogueKey,
    },
}

enum DialoguePhase {
    Starting,
    Open { pending: Option<DialogueAction> },
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum DialogueSet {
    Route,
}

impl Plugin for TacticalDialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneDialogues>()
            .init_resource::<DialogueOutbox>()
            .init_resource::<DialogueMailbox>()
            .add_observer(on_request)
            .add_systems(
                Update,
                (invalidate, callbacks::complete, begin)
                    .chain()
                    .in_set(DialogueSet::Route)
                    .after(ControlSet::Decide),
            )
            .add_systems(Update, transport::dispatch.after(DialogueSet::Route));
    }
}

impl LiveDialogue {
    fn admit(
        reached: ReachedTalkTarget,
        participants: TalkParticipants,
        visits: &LocalVisits,
        serial: &mut DialogueSerial,
    ) -> Result<Self, String> {
        let actor_visit = visits
            .live_id(participants.actor)
            .ok_or("Player visit has ended")?;
        let resident_visit = visits
            .live_id(participants.resident)
            .ok_or("Resident visit has ended")?;
        serial.0 = serial
            .0
            .checked_add(1)
            .ok_or("Dialogue identity space is exhausted")?;
        let session = DialogueSessionId::try_from(format!(
            "dialogue:{}:scene:{}:{}",
            participants.actor.0,
            actor_visit.get(),
            serial.0,
        ))
        .map_err(|error| error.to_string())?;
        Ok(Self {
            key: DialogueKey {
                actor: reached.actor,
                session,
            },
            reached,
            participants,
            actor_visit,
            resident_visit,
            phase: DialoguePhase::Starting,
        })
    }

    fn is_current(&self, world: &TalkContactWorld, visits: Option<&LocalVisits>) -> bool {
        let Some(visits) = visits else {
            return false;
        };
        let Ok(participants) = world.confirm(&self.reached) else {
            return false;
        };
        participants.actor == self.participants.actor
            && participants.resident == self.participants.resident
            && visits.live_id(participants.actor) == Some(self.actor_visit)
            && visits.live_id(participants.resident) == Some(self.resident_visit)
    }
}

impl SceneDialogues {
    fn end(&mut self, actor: Entity, outbox: &mut DialogueOutbox, commands: &mut Commands) {
        let Some(session) = self.sessions.remove(&actor) else {
            return;
        };
        outbox.0.push(DialogueCall::Close {
            key: session.key.clone(),
        });
        commands.server_trigger(ToClients {
            targets: SendTargets::Single(ClientId::Client(actor)),
            message: DialogueEnded {
                session_id: session.key.session,
            },
        });
    }
}

impl DialogueCall {
    fn key(&self) -> &DialogueKey {
        match self {
            Self::Start { key, .. } | Self::Action { key, .. } | Self::Close { key } => key,
        }
    }
}

fn begin(
    mut arrivals: MessageReader<ReachedTalkTarget>,
    world: TalkContactWorld,
    visits: Option<Res<LocalVisits>>,
    mut dialogues: ResMut<SceneDialogues>,
    mut outbox: ResMut<DialogueOutbox>,
    mut commands: Commands,
) {
    let Some(visits) = visits else {
        return;
    };
    for reached in arrivals.read() {
        if dialogues
            .sessions
            .get(&reached.actor)
            .is_some_and(|existing| {
                existing.reached.generation == reached.generation
                    && existing.reached.sequence == reached.sequence
            })
        {
            continue;
        }
        dialogues.end(reached.actor, &mut outbox, &mut commands);
        let Ok(participants) = world.confirm(reached) else {
            continue;
        };
        let resident = participants.resident;
        let actor_id = participants.actor;
        let session =
            match LiveDialogue::admit(*reached, participants, &visits, &mut dialogues.serial) {
                Ok(session) => session,
                Err(error) => {
                    send_start_result(&mut commands, reached, actor_id, resident, Err(error));
                    continue;
                }
            };
        outbox.0.push(DialogueCall::Start {
            key: session.key.clone(),
            actor_visit: session.actor_visit,
            resident_visit: session.resident_visit,
        });
        dialogues.sessions.insert(reached.actor, session);
    }
}

fn invalidate(
    world: TalkContactWorld,
    visits: Option<Res<LocalVisits>>,
    mut dialogues: ResMut<SceneDialogues>,
    mut outbox: ResMut<DialogueOutbox>,
    mut commands: Commands,
) {
    let obsolete: Vec<_> = dialogues
        .sessions
        .iter()
        .filter(|(_, session)| !session.is_current(&world, visits.as_deref()))
        .map(|(actor, _)| *actor)
        .collect();
    for actor in obsolete {
        dialogues.end(actor, &mut outbox, &mut commands);
    }
}

fn on_request(
    request: On<FromClient<DialogueRequest>>,
    world: TalkContactWorld,
    visits: Option<Res<LocalVisits>>,
    mut dialogues: ResMut<SceneDialogues>,
    mut outbox: ResMut<DialogueOutbox>,
    mut commands: Commands,
) {
    let Some(actor) = request.client_id.entity() else {
        return;
    };
    let Some(session) = dialogues.sessions.get_mut(&actor) else {
        return;
    };
    if session.key.session != request.session_id
        || session.reached.generation != request.generation
        || session.reached.sequence != request.sequence
    {
        return;
    }
    if request.action == DialogueAction::Close {
        dialogues.end(actor, &mut outbox, &mut commands);
        return;
    }
    if !session.is_current(&world, visits.as_deref()) {
        dialogues.end(actor, &mut outbox, &mut commands);
        return;
    }
    if let DialogueAction::Answer { prompt_row_id, .. } = &request.action
        && prompt_row_id.session_id() != &session.key.session
    {
        send_action_result(
            &mut commands,
            &session.key,
            &request.action,
            Err("The prompt belongs to another conversation".into()),
        );
        return;
    }
    let DialoguePhase::Open { pending } = &mut session.phase else {
        return;
    };
    if pending.as_ref() == Some(&request.action) {
        return;
    }
    if pending.is_some() {
        send_action_result(
            &mut commands,
            &session.key,
            &request.action,
            Err("Another dialogue response is still pending".into()),
        );
        return;
    }
    *pending = Some(request.action.clone());
    outbox.0.push(DialogueCall::Action {
        key: session.key.clone(),
        character: session.participants.actor,
        action: request.action.clone(),
    });
}

fn send_action_result(
    commands: &mut Commands,
    key: &DialogueKey,
    action: &DialogueAction,
    result: Result<(), String>,
) {
    let Some(action_id) = action.action_id() else {
        return;
    };
    commands.server_trigger(ToClients {
        targets: SendTargets::Single(ClientId::Client(key.actor)),
        message: DialogueActionResponse {
            session_id: key.session.clone(),
            action_id: action_id.to_owned(),
            result,
        },
    });
}

fn send_start_result(
    commands: &mut Commands,
    reached: &ReachedTalkTarget,
    actor: adventuresim_tactical_core::player::CharacterId,
    resident: adventuresim_tactical_core::player::CharacterId,
    result: Result<DialogueSessionId, String>,
) {
    commands.server_trigger(ToClients {
        targets: SendTargets::Single(ClientId::Client(reached.actor)),
        message: DialogueStartResponse {
            controlled_actor: reached.actor,
            actor,
            generation: reached.generation,
            sequence: reached.sequence,
            resident,
            result,
        },
    });
}

#[cfg(test)]
mod tests;
