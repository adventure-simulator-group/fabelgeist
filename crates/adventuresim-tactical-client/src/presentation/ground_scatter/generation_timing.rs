//! CPU generation phases within the normal tactical ground-scatter call.

#[derive(Debug)]
pub(super) enum GroundScatterPhase {
    UnderstoryModels,
    FloorAssets,
    UnderstoryPlacements,
    FloorPlacements,
    LooseStones,
}

impl GroundScatterPhase {
    pub(super) fn measure<T>(self, generate: impl FnOnce() -> T) -> T {
        let started = web_time::Instant::now();
        let result = generate();
        tracing::info!(
            phase = ?self,
            elapsed_ms = started.elapsed().as_millis(),
            "Finished tactical ground-scatter CPU phase"
        );
        result
    }
}
