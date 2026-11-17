//! Reducer calls and owned callback messages isolate the SDK from ECS state.

use super::*;
use crate::stdb::SpacetimeDb;
use adventuresim_stdb_client::{
    answer_dialogue_prompt, choose_dialogue_topic, close_tactical_dialogue, start_tactical_dialogue,
};

pub(super) fn dispatch(
    conn: Option<Res<SpacetimeDb>>,
    world: TalkContactWorld,
    visits: Option<Res<LocalVisits>>,
    dialogues: Res<SceneDialogues>,
    mut outbox: ResMut<DialogueOutbox>,
    mailbox: Res<DialogueMailbox>,
) {
    let Some(conn) = conn else {
        return;
    };
    for call in std::mem::take(&mut outbox.0) {
        if let DialogueCall::Close { key } = &call {
            if let Err(error) = close(&conn, key) {
                warn!(%error, "Could not queue dialogue closure");
            }
            continue;
        }
        // Device events can replace the order after a choice was queued. Check
        // the current physical proof again at the actual RPC boundary.
        if !dialogues
            .sessions
            .get(&call.key().actor)
            .is_some_and(|session| {
                session.key == *call.key() && session.is_current(&world, visits.as_deref())
            })
        {
            continue;
        }
        if let Err(error) = send(&conn, call.clone(), mailbox.clone()) {
            record(&mailbox, call, Err(error));
        }
    }
}

fn send(conn: &SpacetimeDb, call: DialogueCall, mailbox: DialogueMailbox) -> Result<(), String> {
    let catalog = adventuresim_dialogue::CATALOG_DIGEST.to_owned();
    let callback_call = call.clone();
    let callback =
        move |_: &adventuresim_stdb_client::ReducerEventContext,
              result: Result<Result<(), String>, spacetimedb_sdk::error::InternalError>| {
            record(
                &mailbox,
                callback_call,
                result.unwrap_or_else(|error| Err(format!("Dialogue callback failed: {error}"))),
            );
        };
    let queued = match call {
        DialogueCall::Start {
            key,
            actor_visit,
            resident_visit,
        } => conn.reducers().start_tactical_dialogue_then(
            actor_visit.get(),
            resident_visit.get(),
            key.session.into(),
            catalog,
            callback,
        ),
        DialogueCall::Action {
            key,
            character,
            action,
        } => match action {
            DialogueAction::Topic {
                topic_id,
                action_id,
                expected_revision,
            } => conn.reducers().choose_dialogue_topic_then(
                character.0,
                key.session.into(),
                topic_id,
                action_id,
                expected_revision,
                catalog,
                callback,
            ),
            DialogueAction::Answer {
                prompt_row_id,
                choice_ids,
                action_id,
                expected_revision,
            } => {
                let choices =
                    serde_json::to_string(&choice_ids).map_err(|error| error.to_string())?;
                conn.reducers().answer_dialogue_prompt_then(
                    character.0,
                    prompt_row_id.into(),
                    choices,
                    action_id,
                    expected_revision,
                    catalog,
                    callback,
                )
            }
            DialogueAction::Close => return Err("Dialogue closure requires its own request".into()),
        },
        DialogueCall::Close { .. } => {
            return Err("Dialogue closure requires its own request".into());
        }
    };
    queued.map_err(|error| error.to_string())
}

fn close(conn: &SpacetimeDb, key: &DialogueKey) -> Result<(), String> {
    conn.reducers()
        .close_tactical_dialogue_then(key.session.clone().into(), |_, result| {
            if let Err(error) = result.unwrap_or_else(|error| Err(error.to_string())) {
                debug!(%error, "Dialogue closure was not committed");
            }
        })
        .map_err(|error| error.to_string())
}

fn record(mailbox: &DialogueMailbox, call: DialogueCall, result: Result<(), String>) {
    match mailbox.0.lock() {
        Ok(mut mailbox) => mailbox.push(DialogueCompletion { call, result }),
        Err(error) => error!(%error, "Could not record dialogue acknowledgement"),
    }
}
