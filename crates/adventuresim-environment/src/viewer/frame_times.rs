//! How fast the window draws.

use std::collections::VecDeque;
use std::time::Instant;

/// Frames the rate is averaged over.
const WINDOW: usize = 60;

pub struct FrameTimes {
    seconds: VecDeque<f32>,
    last: Instant,
}

impl FrameTimes {
    pub fn new() -> Self {
        Self {
            seconds: VecDeque::with_capacity(WINDOW),
            last: Instant::now(),
        }
    }

    /// Count a frame drawn now.
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.seconds.push_back((now - self.last).as_secs_f32());
        self.last = now;
        while self.seconds.len() > WINDOW {
            self.seconds.pop_front();
        }
    }

    /// Frames per second over the last [`WINDOW`] frames.
    pub fn rate(&self) -> f32 {
        let elapsed: f32 = self.seconds.iter().sum();
        self.seconds.len() as f32 / elapsed.max(f32::EPSILON)
    }
}
