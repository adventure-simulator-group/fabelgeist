//! Frame physical litter roots while retaining their scene and distance contracts.
use crate::presentation::GroundLitterCapturePair;
use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_tactical_core::{
    animation::NonnegativeLength,
    prelude::{SceneObstacle, SceneTerrain},
    scene_coordinates::{Scene, ScenePlanPoint},
};
use bevy::prelude::{GlobalTransform, Result, Vec2, Vec3, Vec3Swizzles};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DebrisCaptureTarget {
    pub(super) focus: Position<Scene>,
    pub(super) camera: Option<Position<Scene>>,
    pub(super) leaf_distance: NonnegativeLength,
    pub(super) twig_distance: NonnegativeLength,
}

struct DebrisPairCandidate {
    separation: NonnegativeLength,
    leaf: Position<Scene>,
    twig: Position<Scene>,
}

struct CaptureObstacle {
    position: Position<Scene>,
    radius: NonnegativeLength,
}

struct RankedTarget {
    score: f32,
    target: DebrisCaptureTarget,
}

impl DebrisCaptureTarget {
    pub(super) fn nearest(
        leaf_positions: &[Position<Scene>],
        twig_positions: &[Position<Scene>],
        maximum_separation: NonnegativeLength,
    ) -> Option<Self> {
        let mut candidates = leaf_positions
            .iter()
            .flat_map(|leaf| twig_positions.iter().map(move |twig| (*leaf, *twig)))
            .filter_map(|(leaf, twig)| {
                let distance = leaf.metres().xz().distance(twig.metres().xz());
                if distance > maximum_separation.metres() {
                    return None;
                }
                Some(DebrisPairCandidate {
                    separation: NonnegativeLength::from_arithmetic(distance).ok()?,
                    leaf,
                    twig,
                })
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            left.separation
                .metres()
                .total_cmp(&right.separation.metres())
                .then_with(|| left.leaf.metres().x.total_cmp(&right.leaf.metres().x))
                .then_with(|| left.leaf.metres().z.total_cmp(&right.leaf.metres().z))
                .then_with(|| left.twig.metres().x.total_cmp(&right.twig.metres().x))
                .then_with(|| left.twig.metres().z.total_cmp(&right.twig.metres().z))
        });
        let candidate = candidates.first()?;
        let leaf = candidate.leaf.metres();
        let twig = candidate.twig.metres();
        let focus = leaf.lerp(twig, 0.5);
        Some(Self {
            focus: Position::from_metres(focus).ok()?,
            camera: None,
            leaf_distance: NonnegativeLength::from_arithmetic(focus.xz().distance(leaf.xz()))
                .ok()?,
            twig_distance: NonnegativeLength::from_arithmetic(focus.xz().distance(twig.xz()))
                .ok()?,
        })
    }

    pub(super) fn terrain_snapped(
        leaf_positions: &[Position<Scene>],
        twig_positions: &[Position<Scene>],
        maximum_separation: NonnegativeLength,
        terrain: &SceneTerrain,
    ) -> Option<Self> {
        let mut target = Self::nearest(leaf_positions, twig_positions, maximum_separation)?;
        let terrain_height = terrain.height_at(ScenePlanPoint::from(target.focus))?;
        let mut focus = target.focus.metres();
        focus.y = focus.y.max(terrain_height.metres() + 0.001);
        target.focus = Position::from_metres(focus).ok()?;
        Some(target)
    }

    pub(super) fn reviewable<'a>(
        pairs: &[GroundLitterCapturePair],
        terrain: &SceneTerrain,
        observed_obstacles: impl IntoIterator<Item = (&'a SceneObstacle, &'a GlobalTransform)>,
        fallback_azimuth_degrees: adventuresim_tactical_core::animation::SignedAngleDegrees,
    ) -> Result<Option<Self>> {
        // Admit the complete native ECS observation before selecting a target.
        let obstacles = observed_obstacles
            .into_iter()
            .map(|(obstacle, transform)| CaptureObstacle::from_observation(obstacle, transform))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self::select(
            pairs,
            terrain,
            &obstacles,
            fallback_azimuth_degrees,
        ))
    }

    fn select(
        pairs: &[GroundLitterCapturePair],
        terrain: &SceneTerrain,
        obstacles: &[CaptureObstacle],
        fallback_azimuth_degrees: adventuresim_tactical_core::animation::SignedAngleDegrees,
    ) -> Option<Self> {
        let half_width = terrain.width() * 0.5;
        let half_depth = terrain.depth() * 0.5;
        let mut candidates = pairs
            .iter()
            .filter_map(|pair| {
                let mut target = Self::terrain_snapped(
                    &[pair.dry_leaf],
                    &[pair.twig],
                    GroundLitterCapturePair::MAXIMUM_SEPARATION,
                    terrain,
                )?;
                let normal = terrain.normal_at(ScenePlanPoint::from(target.focus))?;
                // Native planar clearance, camera and weighted ranking math use
                // metres here; the selected roots and distances remain checked.
                let focus = target.focus.metres();
                let edge_clearance = (half_width - focus.x.abs()).min(half_depth - focus.z.abs());
                if normal.y < 0.82 || edge_clearance < 1.8 {
                    return None;
                }
                let nearest = obstacles.iter().min_by(|left, right| {
                    focus
                        .xz()
                        .distance(left.position.metres().xz())
                        .total_cmp(&focus.xz().distance(right.position.metres().xz()))
                });
                let obstacle_clearance = nearest.map_or(half_width.min(half_depth), |obstacle| {
                    focus.xz().distance(obstacle.position.metres().xz()) - obstacle.radius.metres()
                });
                if obstacle_clearance < 1.25 {
                    return None;
                }
                let fallback = fallback_azimuth_degrees.radians();
                let camera_horizontal = nearest
                    .map(|obstacle| {
                        (focus.xz() - obstacle.position.metres().xz()).normalize_or_zero()
                    })
                    .filter(|direction| direction.length_squared() > 0.5)
                    .unwrap_or(Vec2::new(fallback.sin(), fallback.cos()));
                let camera_xz = focus.xz() + camera_horizontal * 1.15;
                let camera_ground = terrain.height_at(ScenePlanPoint::from_metres(camera_xz)?)?;
                target.camera = Some(
                    Position::from_metres(Vec3::new(
                        camera_xz.x,
                        (focus.y + 1.35).max(camera_ground.metres() + 1.0),
                        camera_xz.y,
                    ))
                    .ok()?,
                );
                let score = obstacle_clearance.min(8.0) * 4.0 + edge_clearance.min(8.0) + normal.y;
                Some(RankedTarget { score, target })
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| {
                    left.target
                        .focus
                        .metres()
                        .x
                        .total_cmp(&right.target.focus.metres().x)
                })
                .then_with(|| {
                    left.target
                        .focus
                        .metres()
                        .z
                        .total_cmp(&right.target.focus.metres().z)
                })
        });
        candidates.first().map(|candidate| candidate.target)
    }
}

impl CaptureObstacle {
    fn from_observation(obstacle: &SceneObstacle, transform: &GlobalTransform) -> Result<Self> {
        let radius = match obstacle {
            SceneObstacle::Tree => 1.8,
            SceneObstacle::Rock(recipe) => recipe.collision_radius_metres(),
        };
        Ok(Self {
            position: Position::from_metres(transform.translation())?,
            radius: NonnegativeLength::from_metres(radius)?,
        })
    }
}

pub(super) fn debris_detail_camera(
    target: Position<Scene>,
    camera: Option<Position<Scene>>,
    azimuth_degrees: adventuresim_tactical_core::animation::SignedAngleDegrees,
) -> super::view_camera::CaptureCameraResult<super::view_camera::SceneCameraView> {
    let azimuth = azimuth_degrees.radians();
    let position = match camera {
        Some(camera) => camera,
        None => Position::from_metres(target.metres() + Vec3::new(azimuth.sin() * 1.15, 1.35, azimuth.cos() * 1.15))?,
    };
    super::view_camera::SceneCameraView::looking_at(position, target, adventuresim_building_generator::spatial_geometry::SpatialDirection::from_normalized(Vec3::Y)?)
}
