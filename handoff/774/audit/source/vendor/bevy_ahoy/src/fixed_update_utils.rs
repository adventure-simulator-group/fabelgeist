use crate::prelude::*;

pub struct AhoyFixedUpdateUtilsPlugin;

impl Plugin for AhoyFixedUpdateUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DidFixedTimestepRunThisFrame>()
            // At the beginning of each frame, clear the flag that indicates whether the fixed timestep has run this frame.
            .add_systems(PreUpdate, clear_fixed_timestep_flag)
            // At the beginning of each fixed timestep, set the flag that indicates whether the fixed timestep has run this frame.
            .add_systems(FixedPreUpdate, set_fixed_time_step_flag);
    }
}

/// A simple resource that tells us whether the fixed timestep ran this frame.
#[derive(Resource, Debug, Deref, DerefMut, Default)]
pub(crate) struct DidFixedTimestepRunThisFrame(bool);

/// Reset the flag at the start of every frame.
fn clear_fixed_timestep_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = false;
}

/// Set the flag during each fixed timestep.
fn set_fixed_time_step_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = true;
}

pub(crate) fn did_fixed_timestep_run_this_frame(
    did_fixed_timestep_run_this_frame: Res<DidFixedTimestepRunThisFrame>,
) -> bool {
    did_fixed_timestep_run_this_frame.0
}
