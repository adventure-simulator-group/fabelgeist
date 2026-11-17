//! Priority fallback composite.

use super::*;

/// Tries its children in order and succeeds with the first child that
/// succeeds. It fails once every child has failed.
#[derive(Debug, Default, Reflect, Clone, Copy)]
pub struct FallbackNode;

/// Task for [`FallbackNode`].
#[derive(Debug, Default, Reflect)]
pub struct FallbackTask {
    current: usize,
}

impl BehaviourNode for FallbackNode {
    type Info<'a> = CompositeNodeInfo<'a>;
    type Task = FallbackTask;

    fn build_task(&self) -> Self::Task {
        FallbackTask::default()
    }
}

pub(super) fn register(app: &mut App) {
    app.add_behaviour_node::<FallbackNode>()
        .with_setup_observer(start_first_child)
        .with_child_finish_observer(try_next_child)
        .register();
}

fn start_first_child(
    event: On<TaskAdded, TaskWorker<FallbackNode>>,
    mut cmd: Commands,
    tasks: Query<NodeRef<FallbackNode>>,
) {
    let Ok(node) = tasks.get(event.entity) else {
        return;
    };
    match node.info().get_child(0) {
        Some(child) => {
            cmd.entity(event.entity).spawn_task(child);
        }
        None => {
            cmd.entity(event.entity).insert(TaskStatus::Failure);
        }
    }
}

fn try_next_child(
    event: On<ChildTaskFinished, TaskWorker<FallbackNode>>,
    mut cmd: Commands,
    mut tasks: Query<(TaskMut<FallbackNode>, NodeRef<FallbackNode>)>,
) {
    let Ok((mut task, node)) = tasks.get_mut(event.task) else {
        return;
    };
    if event.child_status == TaskStatus::Success {
        cmd.entity(event.task).insert(TaskStatus::Success);
        return;
    }
    task.current += 1;
    match node.info().get_child(task.current) {
        Some(child) => {
            cmd.entity(event.task).spawn_task(child);
        }
        None => {
            cmd.entity(event.task).insert(TaskStatus::Failure);
        }
    }
}
