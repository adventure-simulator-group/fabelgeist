//! Camera clearance and close-range composition. Every displayed position is
//! swept from the subject, including positions modified by framing or recovery.

use adventuresim_tactical_core::prelude::{
    Collider, ShapeCastConfig, ShapeHitData, SpatialQuery, SpatialQueryFilter,
};
use bevy::prelude::*;

use super::{CameraRigConfig, smoothing::critical_damp_scalar};

/// Bounds constraint refinement when close framing reveals another obstacle.
const MAX_FRAMING_SWEEPS: usize = 8;

/// Close framing keeps the subject beside the view without changing manual aim.
#[derive(Debug, Clone, Copy)]
pub struct CloseCameraProfile {
    /// Boom distance where close framing begins, in metres.
    pub start_distance: f32,
    /// Boom distance where close framing is fully applied, in metres.
    pub full_distance: f32,
    /// Camera-local right and up offsets from the subject focus, in metres.
    pub offset: Vec2,
}

impl Default for CloseCameraProfile {
    fn default() -> Self {
        Self {
            start_distance: 1.8,
            full_distance: 0.65,
            offset: Vec2::new(0.38, 0.12),
        }
    }
}

impl CloseCameraProfile {
    fn blend(self, distance: f32) -> f32 {
        let t = ((self.start_distance - distance) / (self.start_distance - self.full_distance))
            .clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

/// A conservative box enclosing the eye and all four near-plane corners.
/// Its orientation and dimensions follow the actual render projection.
pub(crate) struct CameraVolume {
    pub(crate) shape: Collider,
    pub(crate) local_center: Vec3,
    pub(crate) size: Vec3,
}

impl CameraVolume {
    pub(crate) fn new(projection: &Projection, margin: f32) -> Self {
        let view_from_clip = projection.get_clip_from_view().inverse();
        let mut min = Vec3::ZERO;
        let mut max = Vec3::ZERO;
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                // Bevy uses reverse Z: the near plane is at clip-space Z = 1.
                let corner = view_from_clip.project_point3(Vec3::new(x, y, 1.0));
                min = min.min(corner);
                max = max.max(corner);
            }
        }
        let size = max - min + Vec3::splat(2.0 * margin);
        Self {
            shape: Collider::cuboid(size.x, size.y, size.z),
            local_center: (min + max) * 0.5,
            size,
        }
    }

    fn sweep(
        &self,
        spatial: &SpatialQuery,
        filter: &SpatialQueryFilter,
        hard_obstacle: &dyn Fn(Entity) -> bool,
        frame: &CameraFrame,
        endpoint: Vec3,
    ) -> CameraSweep {
        let delta = endpoint - frame.origin;
        let Ok(direction) = Dir3::new(delta) else {
            return CameraSweep {
                fraction: 1.0,
                hit: None,
            };
        };
        let distance = delta.length();
        let hit = spatial.cast_shape_predicate(
            &self.shape,
            frame.origin + frame.rotation * self.local_center,
            frame.rotation,
            direction,
            &ShapeCastConfig::from_max_distance(distance),
            filter,
            hard_obstacle,
        );
        CameraSweep {
            fraction: hit.map_or(1.0, |hit| (hit.distance / distance).clamp(0.0, 1.0)),
            hit,
        }
    }
}

struct CameraSweep {
    fraction: f32,
    hit: Option<ShapeHitData>,
}

pub(crate) struct CameraFrame {
    /// Controller capsule center: a collision origin independent of focus lag.
    pub(crate) origin: Vec3,
    pub(crate) anchor: Vec3,
    pub(crate) focus: Vec3,
    pub(crate) rotation: Quat,
    pub(crate) shoulder_offset: f32,
    pub(crate) distance: f32,
}

impl CameraFrame {
    fn endpoint(&self, distance: f32, close: CloseCameraProfile) -> Vec3 {
        let blend = close.blend(distance);
        let focus = self.focus.lerp(self.anchor, blend);
        let offset = Vec2::new(self.shoulder_offset, 0.0).lerp(close.offset, blend);
        focus + self.rotation * Vec3::new(offset.x, offset.y, distance)
    }
}

pub(crate) struct CameraPlacement {
    pub(crate) position: Vec3,
    pub(crate) desired: Vec3,
    pub(crate) hit: Option<ShapeHitData>,
    pub(crate) limited_distance: f32,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BoomRecovery {
    pub(crate) distance: f32,
    pub(crate) velocity: f32,
    hold_remaining: f32,
}

impl BoomRecovery {
    pub(crate) fn reset(&mut self, distance: f32) {
        *self = Self {
            distance,
            ..default()
        };
    }

    fn retract(&mut self, distance: f32, config: &CameraRigConfig) {
        self.distance = distance;
        self.velocity = 0.0;
        self.hold_remaining = config.collision_hold_time;
    }

    fn recover(&mut self, desired: f32, limited: f32, config: &CameraRigConfig, dt: f32) {
        if limited < self.distance {
            self.retract(limited, config);
            return;
        }
        if limited < desired && limited - self.distance <= config.collision_hysteresis {
            self.velocity = 0.0;
            self.hold_remaining = config.collision_hold_time;
            return;
        }
        if self.hold_remaining > 0.0 {
            self.hold_remaining = (self.hold_remaining - dt).max(0.0);
            return;
        }
        self.distance = critical_damp_scalar(
            self.distance,
            desired.min(limited),
            &mut self.velocity,
            config.collision_recovery_time,
            dt,
        )
        .min(limited);
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "placement combines the retained boom with geometry, projection, framing, and timing"
    )]
    pub(crate) fn place(
        &mut self,
        frame: &CameraFrame,
        volume: &CameraVolume,
        spatial: &SpatialQuery,
        filter: &SpatialQueryFilter,
        hard_obstacle: &dyn Fn(Entity) -> bool,
        config: &CameraRigConfig,
        dt: f32,
    ) -> CameraPlacement {
        let desired = frame.endpoint(frame.distance, config.close);
        let mut limited = frame.distance;
        let mut obstruction = None;
        // Solve the geometric limit independently of the previous frame. A
        // close shoulder offset can meet a second obstacle after the boom has
        // already retracted; recovery must not target that obstructed pose.
        for _ in 0..MAX_FRAMING_SWEEPS {
            let endpoint = frame.endpoint(limited, config.close);
            let sweep = volume.sweep(spatial, filter, hard_obstacle, frame, endpoint);
            obstruction = sweep.hit.or(obstruction);
            limited *= sweep.fraction;
            if sweep.hit.is_none() {
                break;
            }
        }
        self.recover(frame.distance, limited, config, dt);

        let candidate = frame.endpoint(self.distance, config.close);
        // Framing, focus lag and damping all change the tested path. Validate
        // their final result; no camera translation is applied after this sweep.
        let final_sweep = volume.sweep(spatial, filter, hard_obstacle, frame, candidate);
        let position = frame.origin.lerp(candidate, final_sweep.fraction);
        if final_sweep.fraction < 1.0 {
            self.retract(self.distance * final_sweep.fraction, config);
            limited = limited.min(self.distance);
        }
        CameraPlacement {
            position,
            desired,
            hit: final_sweep.hit.or(obstruction),
            limited_distance: limited,
        }
    }
}

#[cfg(test)]
mod tests;
