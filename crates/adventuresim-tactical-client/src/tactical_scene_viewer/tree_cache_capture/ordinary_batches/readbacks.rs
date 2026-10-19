//! GPU handles remain in the render world; delivery retains the original owner.
use super::*;
use inbox::Inbox;
use snapshots::{EncodedSnapshots, MappedSnapshots};
use types::RenderFrameId;

#[derive(Resource, Default)]
pub(super) struct Queue {
    state: QueueState,
}

#[derive(Default)]
enum QueueState {
    #[default]
    Empty,
    Encoded(EncodedTask),
    Mapping(MappingTask),
}

struct EncodedTask {
    owner: Arc<Pose>,
    frame: RenderFrameId,
    inbox: Inbox,
    copies: EncodedSnapshots,
}

struct MappingTask {
    owner: Arc<Pose>,
    frame: RenderFrameId,
    inbox: Inbox,
    copies: MappedSnapshots,
}

impl Queue {
    pub(super) fn enqueue(
        &mut self,
        owner: Arc<Pose>,
        frame: RenderFrameId,
        inbox: Inbox,
        copies: EncodedSnapshots,
    ) -> Result<()> {
        ensure!(
            matches!(self.state, QueueState::Empty),
            "more than one pending GPU observation"
        );
        inbox.verify_owner(owner.view, frame)?;
        self.state = QueueState::Encoded(EncodedTask {
            owner,
            frame,
            inbox,
            copies,
        });
        Ok(())
    }

    fn after_submission(&mut self) -> Result<()> {
        self.state = match std::mem::replace(&mut self.state, QueueState::Empty) {
            QueueState::Empty => QueueState::Empty,
            QueueState::Encoded(task) => QueueState::Mapping(MappingTask {
                owner: task.owner,
                frame: task.frame,
                inbox: task.inbox,
                copies: task.copies.map(),
            }),
            QueueState::Mapping(mut task) => match task.copies.take_complete() {
                Ok(None) => QueueState::Mapping(task),
                Ok(Some(bytes)) => {
                    task.inbox.deliver(task.owner.view, task.frame, Ok(bytes))?;
                    QueueState::Empty
                }
                Err(error) => {
                    task.inbox
                        .deliver(task.owner.view, task.frame, Err(format!("{error:#}")))?;
                    QueueState::Empty
                }
            },
        };
        Ok(())
    }
}

pub(super) fn map_submitted(mut queue: ResMut<Queue>) -> bevy::prelude::Result {
    queue.after_submission().map_err(Into::into)
}
