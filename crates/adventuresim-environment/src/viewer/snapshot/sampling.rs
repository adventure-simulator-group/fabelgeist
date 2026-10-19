//! Fixed-frame sampling and the existing optional viewer camera/refeed actions.

use std::f32::consts::TAU;
use std::time::Instant;

use crate::benchmark::Measurement;
use crate::input::FrozenForest;
use adventuresim_environment::render::{ForestFeed, trees::TreeFeed};
use bevy::prelude::*;

use super::{Still, WARMUP, WARMUP_FRAMES};

impl Still {
    /// Preserve an early arrival image before the ordinary settling period.
    pub(super) fn settle(
        &mut self,
        commands: &mut Commands,
        measurement: Option<&mut Measurement>,
    ) -> bool {
        let drawn = *self.drawn.get_or_insert_with(Instant::now);
        if let Some(measurement) = measurement {
            let Some(target) = self.target.clone() else {
                return false;
            };
            measurement.capture_arrival(commands, target);
            if !measurement.arrival_ready() {
                return false;
            }
        }
        if self.settling < WARMUP_FRAMES || drawn.elapsed() < WARMUP {
            self.settling += 1;
            return false;
        }
        true
    }

    pub(super) fn sample_frames(
        &mut self,
        frames: u32,
        measurement: Option<&mut Measurement>,
        feed: &mut ForestFeed,
        trees: Option<&mut TreeFeed>,
        cameras: &mut Query<&mut Transform, With<Camera3d>>,
    ) -> bool {
        if self.timed <= frames {
            if let Some(measurement) = measurement {
                measurement.frame();
            }
            self.timing.get_or_insert_with(Instant::now);
            let timed = self.timed;
            self.timed += 1;
            if timed < frames {
                if self.turning {
                    let mut orbit = self.orbit;
                    orbit.yaw += TAU * timed as f32 / frames as f32;
                    for mut transform in cameras {
                        *transform = orbit.transform();
                    }
                }
                if self
                    .refeed
                    .is_some_and(|every| timed.is_multiple_of(every.max(1)))
                {
                    match &self.forest {
                        FrozenForest::Unique(snapshot) => feed.show(snapshot.clone()),
                        FrozenForest::Shared(forest) => {
                            if let Some(trees) = trees {
                                trees.show(forest.clone());
                            }
                        }
                    }
                }
                return true;
            }
            self.timed_elapsed = self.timing.map(|started| started.elapsed());
        }
        false
    }
}
