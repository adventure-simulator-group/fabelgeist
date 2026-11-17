//! A durable acknowledgement cannot revive a superseded physical contact.

use super::*;

pub(super) fn complete(
    mailbox: Res<DialogueMailbox>,
    world: TalkContactWorld,
    visits: Option<Res<LocalVisits>>,
    mut dialogues: ResMut<SceneDialogues>,
    mut outbox: ResMut<DialogueOutbox>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    let completions = match mailbox.0.lock() {
        Ok(mut mailbox) => std::mem::take(&mut *mailbox),
        Err(error) => {
            error!(%error, "Dialogue acknowledgement mailbox poisoned");
            exit.write(AppExit::error());
            return;
        }
    };
    for completion in completions {
        let key = completion.call.key();
        let current = dialogues.sessions.get(&key.actor).is_some_and(|session| {
            session.key == *key && session.is_current(&world, visits.as_deref())
        });
        if !current {
            if matches!(completion.call, DialogueCall::Start { .. }) {
                outbox.0.push(DialogueCall::Close { key: key.clone() });
            }
            continue;
        }
        let Some(session) = dialogues.sessions.get_mut(&key.actor) else {
            continue;
        };
        match &completion.call {
            DialogueCall::Start { .. } => {
                if !matches!(session.phase, DialoguePhase::Starting) {
                    continue;
                }
                let result = completion.result.map(|()| session.key.session.clone());
                let failed = result.is_err();
                send_start_result(
                    &mut commands,
                    &session.reached,
                    session.participants.actor,
                    session.participants.resident,
                    result,
                );
                if failed {
                    dialogues.end(key.actor, &mut outbox, &mut commands);
                } else {
                    session.phase = DialoguePhase::Open { pending: None };
                }
            }
            DialogueCall::Action { action, .. } => {
                let DialoguePhase::Open { pending } = &mut session.phase else {
                    continue;
                };
                if pending.as_ref() != Some(action) {
                    continue;
                }
                *pending = None;
                send_action_result(&mut commands, key, action, completion.result);
            }
            DialogueCall::Close { .. } => {}
        }
    }
}
