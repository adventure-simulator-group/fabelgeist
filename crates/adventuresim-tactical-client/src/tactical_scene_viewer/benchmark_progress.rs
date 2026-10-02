//! Observable preparation progress for the native renderer benchmark.
use super::*;

const BENCHMARK_PROGRESS_INTERVAL: Duration = Duration::from_secs(30);

pub(super) struct ProgressClock {
    last: web_time::Instant,
    first: bool,
}
impl Default for ProgressClock {
    fn default() -> Self {
        Self {
            last: web_time::Instant::now(),
            first: true,
        }
    }
}

pub(super) fn report(
    mut clock: Local<ProgressClock>,
    state: Option<Res<ScenePerformanceBenchmarkState>>,
    cloud: Res<TacticalCloudAnimationStatus>,
) {
    let Some(state) = state else {
        return;
    };
    if !clock.first && clock.last.elapsed() < BENCHMARK_PROGRESS_INTERVAL {
        return;
    }
    clock.first = false;
    clock.last = web_time::Instant::now();
    info!(
        configured = state.configured_mode.is_some(),
        mode = state.mode,
        cloud_ready = cloud.is_ready(),
        warmup_remaining = state.warmup_remaining,
        measured_frames = state.samples_ms.len(),
        "Renderer benchmark progress"
    );
}

impl ScenePerformanceBenchmarkState {
    pub(super) fn new(sample_frames: u32) -> Self {
        let selected_mode = std::env::var("TACTICAL_BENCH_ONLY_MODE")
            .ok()
            .map(|requested| {
                SCENE_PERFORMANCE_MODES
                    .iter()
                    .position(|mode| mode.name == requested)
                    .unwrap_or_else(|| panic!("unknown benchmark mode {requested:?}"))
            });
        let mode = selected_mode.unwrap_or(0);
        Self {
            sample_frames,
            mode,
            configured_mode: None,
            warmup_remaining: SCENE_PERFORMANCE_WARMUP_FRAMES * 2,
            samples_ms: Vec::with_capacity(sample_frames as usize),
            render_diagnostic_samples: BTreeMap::new(),
            playable_tree_count: None,
            playable_leaf_entities: None,
            vista_tree_entities: None,
            scene_entity_counts: None,
            results: Vec::with_capacity(SCENE_PERFORMANCE_MODES.len()),
            stop_after_mode: selected_mode.unwrap_or(SCENE_PERFORMANCE_MODES.len() - 1),
        }
    }
}
