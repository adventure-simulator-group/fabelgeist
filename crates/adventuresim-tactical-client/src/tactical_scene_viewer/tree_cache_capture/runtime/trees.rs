//! Fixture-only batch attachment and bounded admission controls.
use super::*;

pub(super) fn install(world: &mut World) -> Result<inventory::OwnedTrees> {
    let role = world.resource::<Review>().role;
    let positions = world.resource::<Review>().roots.clone();
    let roots = TreeRootBatch::from_area_world_metres(world, positions)?;
    let installed = match role {
        Role::Reference => {
            let source = world.resource::<Review>().source;
            InstalledTreeSpecimen::from_generated(world, source, roots)?
        }
        Role::Candidate => {
            let cache = world
                .resource_mut::<Review>()
                .cache
                .take()
                .context("cache consumed once")?;
            cache.spawn(world, roots)?
        }
    };
    if let Some(repeated) = world.resource_mut::<Review>().readmission.take() {
        let positions = world.resource::<Review>().roots.clone();
        let first = installed
            .first()
            .context("shared admission control specimen")?;
        let report =
            crate::presentation::RootAdmissionReport::run(world, first, repeated, &positions)?;
        fs::write(
            world
                .resource::<SceneCaptureState>()
                .output
                .join("cache-root-admission-controls.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
    }
    inventory::OwnedTrees::from_cache(installed, world)
}
