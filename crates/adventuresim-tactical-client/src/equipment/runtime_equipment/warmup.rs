//! Initial readiness owns first-use pipeline work; travel retains the resource.
use super::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};

#[derive(Default)]
enum State {
    #[default]
    AwaitingBody,
    Preparing(
        Task<anyhow::Result<adventuresim_character_creator::runtime_equipment::WarmupReport>>,
    ),
    Ready,
    Failed,
}

#[derive(Resource, Default)]
pub(crate) struct RuntimeEquipmentWarmup {
    state: State,
}

impl RuntimeEquipmentWarmup {
    /// Active device preparation may outlast an asset-loading frame budget.
    pub(crate) fn is_preparing(&self) -> bool {
        matches!(self.state, State::Preparing(_))
    }

    pub(crate) fn check(&self) -> Result<bool, &'static str> {
        match self.state {
            State::Ready => Ok(true),
            State::Failed => Err("Could not prepare character equipment pipelines"),
            State::AwaitingBody | State::Preparing(_) => Ok(false),
        }
    }

    fn poll(&mut self) {
        let State::Preparing(task) = &mut self.state else {
            return;
        };
        let Some(result) = block_on(future::poll_once(task)) else {
            return;
        };
        self.state = match result {
            Ok(report) => {
                info!(?report, "runtime equipment pipelines ready");
                State::Ready
            }
            Err(error) => {
                error!("runtime equipment preparation failed: {error:#}");
                State::Failed
            }
        };
    }

    fn start(&mut self, cache: &RuntimeEquipmentBodyCache) {
        if !matches!(self.state, State::AwaitingBody) {
            return;
        }
        if cache.failed {
            self.state = State::Failed;
            return;
        }
        let (Some(body), Some(bracer), Some(breastplate)) =
            (&cache.body, &cache.bracer_design, &cache.breastplate_design)
        else {
            return;
        };
        let body = body.body.clone();
        let bracer = bracer.clone();
        let breastplate = breastplate.clone();
        self.state = State::Preparing(AsyncComputeTaskPool::get().spawn(async move {
            adventuresim_character_creator::runtime_equipment::warm_up(&body, &bracer, &breastplate)
                .await
        }));
    }

    #[cfg(test)]
    pub(crate) fn ready_for_tests() -> Self {
        Self {
            state: State::Ready,
        }
    }
}

pub(in crate::equipment) fn ready(warmup: Res<RuntimeEquipmentWarmup>) -> bool {
    warmup.check() == Ok(true)
}

pub(in crate::equipment) fn prepare(
    cache: Res<RuntimeEquipmentBodyCache>,
    mut warmup: ResMut<RuntimeEquipmentWarmup>,
) {
    if warmup.is_preparing() {
        warmup.poll();
    } else {
        warmup.start(&cache);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_body_preparation_surfaces_without_starting_a_gpu_task() {
        let mut app = App::new();
        app.insert_resource(RuntimeEquipmentBodyCache {
            failed: true,
            ..default()
        });
        app.init_resource::<RuntimeEquipmentWarmup>();
        app.add_systems(Update, prepare);
        app.update();
        assert!(
            app.world()
                .resource::<RuntimeEquipmentWarmup>()
                .check()
                .is_err()
        );
        // A later update cannot clear the error or report successful readiness.
        app.update();
        assert!(
            app.world()
                .resource::<RuntimeEquipmentWarmup>()
                .check()
                .is_err()
        );
    }
}
