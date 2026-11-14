//! Loading stages and the rules that decide when a load is ready or stuck.
use crate::presentation::pipeline_readiness::SETTLED_RENDER_FRAMES;
use std::{fmt, time::Duration};

/// How long every stage may stand still before the load counts as stuck. Long
/// enough for one slow browser pipeline compile; the connection has no timeout
/// of its own, so this also bounds an unanswered connect. Measured in game
/// time, because a browser stops running frames in a background tab and the
/// wall clock would fail the load the moment the tab returned.
const STALL_LIMIT: Duration = Duration::from_secs(90);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LoadingStage {
    Connection,
    Character,
    Textures,
    Terrain,
    Vista,
    Trees,
    GroundCover,
    Sky,
    Equipment,
    Pose,
    Shaders,
}

impl LoadingStage {
    pub(super) const ALL: [Self; 11] = [
        Self::Connection,
        Self::Character,
        Self::Textures,
        Self::Terrain,
        Self::Vista,
        Self::Trees,
        Self::GroundCover,
        Self::Sky,
        Self::Equipment,
        Self::Pose,
        Self::Shaders,
    ];

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Connection => "Connecting to the server",
            Self::Character => "Joining the scene",
            Self::Textures => "Loading textures",
            Self::Terrain => "Shaping the terrain",
            Self::Vista => "Raising the distant landscape",
            Self::Trees => "Growing trees",
            Self::GroundCover => "Laying ground cover",
            Self::Sky => "Lighting the sky",
            Self::Equipment => "Fitting equipment",
            Self::Pose => "Posing your character",
            Self::Shaders => "Compiling shaders",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct StageProgress {
    pub(super) completed: usize,
    pub(super) total: usize,
}

impl StageProgress {
    pub(super) const PENDING: Self = Self {
        completed: 0,
        total: 1,
    };
    pub(super) const FINISHED: Self = Self {
        completed: 1,
        total: 1,
    };

    pub(super) fn from_flag(finished: bool) -> Self {
        if finished {
            Self::FINISHED
        } else {
            Self::PENDING
        }
    }

    pub(super) fn is_finished(self) -> bool {
        self.total > 0 && self.completed >= self.total
    }

    pub(super) fn fraction(self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        self.completed.min(self.total) as f32 / self.total as f32
    }
}

/// One entry per [`LoadingStage`], in [`LoadingStage::ALL`] order.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct LoadingProgress {
    stages: [StageProgress; LoadingStage::ALL.len()],
}

impl LoadingProgress {
    pub(super) fn get(&self, stage: LoadingStage) -> StageProgress {
        self.stages[stage as usize]
    }

    pub(super) fn set(&mut self, stage: LoadingStage, progress: StageProgress) {
        self.stages[stage as usize] = progress;
    }

    fn first_unfinished(&self) -> Option<LoadingStage> {
        LoadingStage::ALL
            .into_iter()
            .find(|stage| !self.get(*stage).is_finished())
    }
}

/// Turns a shrinking backlog into progress. The size of a backlog is unknown
/// up front, so its largest observed size stands in for the total.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct BacklogPeak(usize);

impl BacklogPeak {
    pub(super) fn observe(&mut self, pending: usize) -> StageProgress {
        self.0 = self.0.max(pending);
        if self.0 == 0 {
            return StageProgress::FINISHED;
        }
        StageProgress {
            completed: self.0 - pending,
            total: self.0,
        }
    }
}

pub(super) enum RunVerdict {
    Waiting,
    Ready,
    Stalled(LoadingStage),
}

/// One attempt at loading a scene.
pub(super) struct LoadingRun {
    pub(super) progress: LoadingProgress,
    pub(super) trees: BacklogPeak,
    pub(super) pipelines: BacklogPeak,
    settled_frames: usize,
    last_advance: Duration,
}

impl LoadingRun {
    pub(super) fn new(now: Duration) -> Self {
        Self {
            progress: LoadingProgress::default(),
            trees: BacklogPeak::default(),
            pipelines: BacklogPeak::default(),
            settled_frames: 0,
            last_advance: now,
        }
    }

    pub(super) fn observe(&mut self, progress: LoadingProgress, now: Duration) -> RunVerdict {
        if progress != self.progress {
            self.last_advance = now;
        }
        self.progress = progress;
        let Some(stage) = progress.first_unfinished() else {
            self.settled_frames += 1;
            return if self.settled_frames >= SETTLED_RENDER_FRAMES {
                RunVerdict::Ready
            } else {
                RunVerdict::Waiting
            };
        };
        self.settled_frames = 0;
        if now.saturating_sub(self.last_advance) >= STALL_LIMIT {
            RunVerdict::Stalled(stage)
        } else {
            RunVerdict::Waiting
        }
    }
}

/// Why a load stopped. Presentation text comes from [`fmt::Display`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum LoadingFailure {
    ConnectionLost(String),
    Asset(String),
    Stalled(LoadingStage),
}

impl fmt::Display for LoadingFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionLost(reason) => write!(formatter, "Connection lost: {reason}"),
            Self::Asset(path) => write!(formatter, "Could not load {path}"),
            Self::Stalled(stage) => {
                write!(formatter, "Loading stopped at: {}", stage.label())
            }
        }
    }
}
