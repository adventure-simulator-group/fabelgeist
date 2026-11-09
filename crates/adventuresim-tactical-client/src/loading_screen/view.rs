//! The loading overlay, drawn by the browser page (`tactical-hud.js`). This
//! module turns the loading phase into the page's view and pushes it when it
//! changes; the page owns the fade and the Exit button, and sends Retry back
//! as a browser command.
use super::{LoadingPhase, LoadingScreen, REVEAL_SECONDS, progress::LoadingStage};
use crate::hud::{self, HudView, LastPushed};
use bevy::prelude::*;
use serde::Serialize;

/// Bar fractions cross to the page in thousandths, enough for a 440 px bar.
const FRACTION_QUANTUM: f32 = 1000.0;

#[derive(Debug, PartialEq, Serialize)]
struct StageView {
    label: &'static str,
    fraction: f32,
    finished: bool,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
enum LoadingView {
    Hidden,
    Loading {
        stages: Vec<StageView>,
    },
    /// The bars stay as they were when the load failed.
    Failed {
        message: String,
    },
    /// The page fades the overlay out over this many seconds.
    Revealing {
        seconds: f32,
    },
}

impl LoadingView {
    fn new(screen: &LoadingScreen) -> Self {
        match &screen.phase {
            LoadingPhase::Idle => Self::Hidden,
            LoadingPhase::Loading(run) => Self::Loading {
                stages: LoadingStage::ALL
                    .into_iter()
                    .map(|stage| {
                        let progress = run.progress.get(stage);
                        StageView {
                            label: stage.label(),
                            fraction: (progress.fraction() * FRACTION_QUANTUM).round()
                                / FRACTION_QUANTUM,
                            finished: progress.is_finished(),
                        }
                    })
                    .collect(),
            },
            LoadingPhase::Failed(failure) => Self::Failed {
                message: failure.to_string(),
            },
            LoadingPhase::Revealing(_) => Self::Revealing {
                seconds: REVEAL_SECONDS,
            },
        }
    }
}

pub(super) fn push_overlay(screen: Res<LoadingScreen>, mut last: Local<LastPushed<String>>) {
    let Ok(json) = serde_json::to_string(&LoadingView::new(&screen)) else {
        return;
    };
    if let Some(json) = last.update(json) {
        hud::view(HudView::Loading, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_screen_is_hidden_and_failure_carries_its_message() {
        assert_eq!(
            serde_json::to_string(&LoadingView::new(&LoadingScreen::default())).unwrap(),
            r#"{"state":"hidden"}"#
        );
        let view = LoadingView::Failed {
            message: "connection lost".into(),
        };
        assert_eq!(
            serde_json::to_string(&view).unwrap(),
            r#"{"state":"failed","message":"connection lost"}"#
        );
    }
}
