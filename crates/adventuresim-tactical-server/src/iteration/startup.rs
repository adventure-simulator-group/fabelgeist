//! Async scene preparation completes before the deterministic duel clock starts.

use std::time::{Duration, Instant};

use bevy::prelude::*;

use super::{TacticalIterationError, TacticalIterationResult};
use crate::navigation::TacticalNavigation;

const NAVIGATION_STARTUP_LIMIT: Duration = Duration::from_secs(15);

pub(super) fn prepare_navigation(app: &mut App) -> TacticalIterationResult<()> {
    let deadline = Instant::now() + NAVIGATION_STARTUP_LIMIT;
    loop {
        let navigation = app.world().resource::<TacticalNavigation>();
        if navigation.ready {
            return Ok(());
        }
        if navigation.failed {
            return Err(TacticalIterationError::NavigationUnavailable);
        }
        if Instant::now() >= deadline {
            return Err(TacticalIterationError::NavigationStartupTimedOut {
                limit: NAVIGATION_STARTUP_LIMIT,
            });
        }
        // No actors exist and manual duration is zero: asset tasks can complete
        // without consuming combat time, physics steps or combat randomness.
        app.update();
        std::thread::yield_now();
    }
}
