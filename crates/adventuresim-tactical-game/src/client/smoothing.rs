//! This player's drawn body: the prediction plus an error that eases away. A correction moves
//! the prediction at once; the body follows it smoothly, so a misprediction never pops.

use bevy::prelude::*;
use bevy_netahoy::{OwnVisual, PendingCorrection, PredictedPlayer};

/// Peak acceleration (m/s²) a correction eases with; sets the spring's stiffness per fold.
const MAX_ACCEL: f32 = 40.0;
/// The fastest and slowest a correction settles (s, to 95%).
const SETTLE_MIN_SECS: f32 = 0.08;
const SETTLE_MAX_SECS: f32 = 0.35;
/// Critical damping settles to 95% in this many time constants.
const SETTLE_TIME_CONSTANTS: f32 = 4.7;

/// A critically damped offset, `drawn = predicted + offset`, decaying along
/// `(A + Bt)·e^(−ωt)`. Folding a correction keeps the offset and its velocity, so the drawn path
/// stays smooth through it.
#[derive(Default, Clone, Copy)]
pub(super) struct ErrorBridge {
    offset: Vec3,
    velocity: Vec3,
    omega: f32,
}

impl ErrorBridge {
    fn advance(&mut self, dt: f32) {
        if self.offset.length_squared() < 1e-6 && self.velocity.length_squared() < 1e-4 {
            *self = Self::default();
            return;
        }
        let a = self.offset;
        let b = self.velocity + self.omega * self.offset;
        let decay = (-self.omega * dt).exp();
        self.offset = (a + b * dt) * decay;
        self.velocity = (b - self.omega * (a + b * dt)) * decay;
    }

    /// Takes a correction in; a bigger one settles more slowly instead of yanking.
    fn fold(&mut self, position: Vec3, velocity: Vec3) {
        self.offset += position;
        self.velocity += velocity;
        let e0 = self.offset.length().max(1e-5);
        let v0 = self.velocity.length().max(1e-4);
        let omega_position = (MAX_ACCEL / e0).sqrt();
        let omega_velocity = MAX_ACCEL / (2.0 * v0);
        self.omega = omega_position.min(omega_velocity).clamp(
            SETTLE_TIME_CONSTANTS / SETTLE_MAX_SECS,
            SETTLE_TIME_CONSTANTS / SETTLE_MIN_SECS,
        );
    }
}

/// This player's prediction, not its visual: both have a `Transform`.
type PredictionOnly = (With<PredictedPlayer>, Without<OwnVisual>);

/// Places this player's visual at its prediction plus the easing error.
pub(super) fn place_own_visual(
    time: Res<Time>,
    mut predictions: Query<(&Transform, &mut PendingCorrection), PredictionOnly>,
    mut visuals: Query<(&OwnVisual, &mut Transform), Without<PredictedPlayer>>,
    mut bridge: Local<ErrorBridge>,
) {
    let Ok((visual, mut transform)) = visuals.single_mut() else {
        *bridge = ErrorBridge::default();
        return;
    };
    let Ok((prediction, mut pending)) = predictions.get_mut(visual.prediction_entity) else {
        return;
    };
    match pending.take() {
        PendingCorrection::None => {}
        PendingCorrection::Snap => *bridge = ErrorBridge::default(),
        PendingCorrection::Fold {
            position,
            velocity,
            snap_gate,
        } => {
            if position.length() >= snap_gate {
                *bridge = ErrorBridge::default();
            } else {
                bridge.fold(position, velocity);
            }
        }
    }
    bridge.advance(time.delta_secs());
    transform.translation = prediction.translation + bridge.offset;
}
