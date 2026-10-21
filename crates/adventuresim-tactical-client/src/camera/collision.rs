//! Camera clearance and close-range composition. Every displayed position is
//! swept from the subject, including positions modified by framing or recovery.

use fabelgeist_geometry::SignedLength;
use adventuresim_building_generator::spatial_geometry::{
    CuboidDimensions, Displacement, GeometryFrame, Position, RigidRotation,
    SpatialDirection,
};
use adventuresim_core::linear_speed::SignedSpeed;
use adventuresim_tactical_core::{
    animation::{AnimationElapsed, UnitInterval},
    prelude::{Collider, ShapeCastConfig, SpatialQuery, SpatialQueryFilter},
    scene_coordinates::Scene,
};
use bevy::prelude::*;

use super::{
    CameraRigConfig,
    config::authored_metres,
    smoothing::critical_damp_scalar,
    values::{CameraResult, FramingOffset},
};

/// Bounds constraint refinement when close framing reveals another obstacle.
const MAX_FRAMING_SWEEPS: usize = 8;

/// Close framing keeps the subject beside the view without changing manual aim.
#[derive(Debug, Clone, Copy)]
pub struct CloseCameraProfile {
    pub start_distance: SignedLength,
    pub full_distance: SignedLength,
    /// Camera-local right and up offsets from the subject focus.
    pub offset: FramingOffset,
}

/// Render view coordinates; positive Z points behind the eye.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
enum CameraView {}
impl GeometryFrame for CameraView {}

/// A conservative box enclosing the eye and all four near-plane corners.
/// Its orientation and dimensions follow the actual render projection.
pub(crate) struct CameraVolume {
    pub(crate) shape: Collider,
    local_center: Displacement<CameraView>,
    size: CuboidDimensions,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CameraVolumeDebug {
    centre: Position<Scene>,
    rotation: RigidRotation,
    size: CuboidDimensions,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CameraHit {
    pub(crate) entity: Entity,
    pub(crate) normal: SpatialDirection<Scene>,
}

struct CameraSweep {
    fraction: UnitInterval,
    hit: Option<CameraHit>,
}

pub(crate) struct CameraFrame {
    /// Controller capsule center: a collision origin independent of focus lag.
    pub(crate) origin: Position<Scene>,
    pub(crate) anchor: Position<Scene>,
    pub(crate) focus: Position<Scene>,
    pub(crate) rotation: RigidRotation,
    pub(crate) shoulder_offset: SignedLength,
    pub(crate) distance: SignedLength,
}

pub(crate) struct CameraPlacement {
    pub(crate) position: Position<Scene>,
    pub(crate) desired: Position<Scene>,
    pub(crate) hit: Option<CameraHit>,
    pub(crate) limited_distance: SignedLength,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct BoomRecovery {
    pub(crate) distance: SignedLength,
    pub(crate) velocity: SignedSpeed,
    hold_remaining: AnimationElapsed,
}

impl Default for CloseCameraProfile {
    fn default() -> Self {
        const { Self {
            start_distance: authored_metres(1.8),
            full_distance: authored_metres(0.65),
            offset: FramingOffset { right: authored_metres(0.38), up: authored_metres(0.12) },
        } }
    }
}

impl CloseCameraProfile {
    fn blend(self, distance: SignedLength) -> f32 {
        let t = ((self.start_distance.metres() - distance.metres())
            / (self.start_distance.metres() - self.full_distance.metres()))
            .clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

impl CameraVolume {
    pub(crate) fn new(projection: &Projection, margin: SignedLength) -> CameraResult<Self> {
        // Inverse projection is a native render-view numerical kernel. Its
        // result is admitted once before retaining physical collider geometry.
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
        let size = CuboidDimensions::from_metres(max - min + Vec3::splat(2.0 * margin.metres()))?;
        let native_size = size.metres();
        Ok(Self {
            shape: Collider::cuboid(native_size.x, native_size.y, native_size.z),
            local_center: Displacement::from_metres((min + max) * 0.5)?,
            size,
        })
    }

    pub(crate) fn debug_volume(&self, position: Position<Scene>, rotation: RigidRotation) -> CameraResult<CameraVolumeDebug> {
        Ok(CameraVolumeDebug {
            centre: Position::from_metres(position.metres() + rotation.quaternion() * self.local_center.metres())?,
            rotation,
            size: self.size,
        })
    }

    fn sweep(
        &self,
        spatial: &SpatialQuery,
        filter: &SpatialQueryFilter,
        hard_obstacle: &dyn Fn(Entity) -> bool,
        frame: &CameraFrame,
        endpoint: Position<Scene>,
    ) -> CameraResult<CameraSweep> {
        // The physics cast consumes scene metres and a view-to-scene rotation.
        let delta = endpoint.metres() - frame.origin.metres();
        let Ok(direction) = Dir3::new(delta) else {
            return Ok(CameraSweep { fraction: UnitInterval::from_fraction(1.0)?, hit: None });
        };
        let distance = delta.length();
        let hit = spatial.cast_shape_predicate(
            &self.shape,
            frame.origin.metres() + frame.rotation.quaternion() * self.local_center.metres(),
            frame.rotation.quaternion(),
            direction,
            &ShapeCastConfig::from_max_distance(distance),
            filter,
            hard_obstacle,
        );
        Ok(CameraSweep {
            fraction: UnitInterval::from_fraction(hit.map_or(1.0, |hit| (hit.distance / distance).clamp(0.0, 1.0)))?,
            hit: hit.map(|hit| Ok::<_, super::values::CameraError>(CameraHit {
                entity: hit.entity,
                normal: SpatialDirection::from_normalized(hit.normal1)?,
            })).transpose()?,
        })
    }
}

impl CameraVolumeDebug {
    pub(crate) fn native_transform(self) -> Transform {
        Transform {
            translation: self.centre.metres(),
            rotation: self.rotation.quaternion(),
            scale: self.size.metres(),
        }
    }
}

impl CameraFrame {
    fn endpoint(&self, distance: SignedLength, close: CloseCameraProfile) -> CameraResult<Position<Scene>> {
        // Native affine composition preserves the original glam lerp/order.
        let blend = close.blend(distance);
        let focus = self.focus.metres().lerp(self.anchor.metres(), blend);
        let offset = Vec2::new(self.shoulder_offset.metres(), 0.0)
            .lerp(Vec2::new(close.offset.right.metres(), close.offset.up.metres()), blend);
        Ok(Position::from_metres(focus + self.rotation.quaternion() * Vec3::new(offset.x, offset.y, distance.metres()))?)
    }
}

impl BoomRecovery {
    pub(crate) fn reset(&mut self, distance: SignedLength) {
        *self = Self { distance, ..default() };
    }

    fn retract(&mut self, distance: SignedLength, config: &CameraRigConfig) {
        self.distance = distance;
        self.velocity = SignedSpeed::ZERO;
        self.hold_remaining = config.collision_hold_time;
    }

    fn recover(
        &mut self,
        desired: SignedLength,
        limited: SignedLength,
        config: &CameraRigConfig,
        elapsed: AnimationElapsed,
    ) -> CameraResult<()> {
        if limited.metres() < self.distance.metres() {
            self.retract(limited, config);
            return Ok(());
        }
        if limited.metres() < desired.metres()
            && limited.metres() - self.distance.metres() <= config.collision_hysteresis.metres()
        {
            self.velocity = SignedSpeed::ZERO;
            self.hold_remaining = config.collision_hold_time;
            return Ok(());
        }
        if !self.hold_remaining.is_zero() {
            self.hold_remaining = self.hold_remaining.saturating_sub(elapsed);
            return Ok(());
        }
        // Analytic damping consumes metres and signed metres per second. Admit
        // both outputs before committing the replacement recovery sample.
        let mut velocity = self.velocity.metres_per_second();
        let distance = SignedLength::from_metres(critical_damp_scalar(
            self.distance.metres(),
            desired.metres().min(limited.metres()),
            &mut velocity,
            config.collision_recovery_time,
            elapsed,
        ).min(limited.metres()))?;
        let velocity = SignedSpeed::from_metres_per_second(velocity)?;
        self.distance = distance;
        self.velocity = velocity;
        Ok(())
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
        elapsed: AnimationElapsed,
    ) -> CameraResult<CameraPlacement> {
        let desired = frame.endpoint(frame.distance, config.close)?;
        let mut limited = frame.distance;
        let mut obstruction = None;
        // Solve the geometric limit independently of the previous frame. A
        // close shoulder offset can meet a second obstacle after the boom has
        // already retracted; recovery must not target that obstructed pose.
        for _ in 0..MAX_FRAMING_SWEEPS {
            let endpoint = frame.endpoint(limited, config.close)?;
            let sweep = volume.sweep(spatial, filter, hard_obstacle, frame, endpoint)?;
            obstruction = sweep.hit.or(obstruction);
            limited = SignedLength::from_metres(limited.metres() * sweep.fraction.fraction())?;
            if sweep.hit.is_none() { break; }
        }
        let mut recovery = *self;
        recovery.recover(frame.distance, limited, config, elapsed)?;
        let candidate = frame.endpoint(recovery.distance, config.close)?;
        // Framing, focus lag and damping all change the tested path. Validate
        // their final result; no camera translation is applied after this sweep.
        let final_sweep = volume.sweep(spatial, filter, hard_obstacle, frame, candidate)?;
        let position = Position::from_metres(frame.origin.metres().lerp(candidate.metres(), final_sweep.fraction.fraction()))?;
        if final_sweep.fraction.fraction() < 1.0 {
            recovery.retract(SignedLength::from_metres(recovery.distance.metres() * final_sweep.fraction.fraction())?, config);
            limited = limited.min(recovery.distance);
        }
        *self = recovery;
        Ok(CameraPlacement { position, desired, hit: final_sweep.hit.or(obstruction), limited_distance: limited })
    }
}

#[cfg(test)]
mod tests;
