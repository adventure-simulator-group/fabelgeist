use super::*;

mod landscape;

pub(super) type CaptureCameraResult<T> = std::result::Result<T, CaptureCameraError>;

#[derive(Debug, thiserror::Error)]
pub(super) enum CaptureCameraError {
    #[error("the requested vista capture has no geographic samples")]
    MissingVista,
    #[error("botanical review has no production root for the requested species")]
    MissingPlant,
    #[error("building capture camera is unavailable for {pose:?}")]
    MissingBuildingCamera { pose: CapturePose },
    #[error("capture camera requires a live production query for {pose:?}")]
    RequiresLiveQuery { pose: CapturePose },
    #[error("capture has no authored camera for {pose:?}")]
    MissingAuthoredCamera { pose: CapturePose },
}


pub(super) fn tree_review(state: &SceneCaptureState) -> (Vec3, Vec3, Vec3) {
state.tree_focus.map_or(
            (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
            |tree| {
                let tree = tree.metres();
                let azimuth = state.tree_review_azimuth_degrees.to_radians();
                // Frame both current mature-tree presentations, including the
                // taller closed-canopy beech. The previous 21.2 m radius and
                // 4.5 m target cropped its crown at every review azimuth.
                let radius = 30.0;
                (
                    tree + Vec3::new(azimuth.sin() * radius, 10.0, azimuth.cos() * radius),
                    tree + Vec3::new(0.0, 8.0, 0.0),
                    Vec3::Y,
                )
            },
        )
}

pub(super) fn recursive_tree(state: &SceneCaptureState) -> (Vec3, Vec3, Vec3) {
state.tree_focus.map_or(
            (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
            |tree| {
                let tree = tree.metres();
                (
                    // Exercise the current detailed-cluster to leafed-twig
                    // handoff. The older 47 m plate predated the tightened
                    // production LOD ranges and could only see whole-crown
                    // aggregates, so it could never prove recursive mixing.
                    tree + Vec3::new(8.0, 4.5, 8.0),
                    tree + Vec3::new(0.0, 4.5, 0.0),
                    Vec3::Y,
                )
            },
        )
}

pub(super) fn tree_root(state: &SceneCaptureState) -> (Vec3, Vec3, Vec3) {
state.tree_focus.map_or(
            (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
            |tree| {
                let tree = tree.metres();
                let root = tree - Vec3::Y * (TREE_TRUNK_HEIGHT_METRES * 0.5 - 0.22);
                let azimuth = state.tree_review_azimuth_degrees.to_radians();
                (
                    // Include the surrounding soil in the root-junction plate:
                    // the camera-local terrain recipe now continues the root
                    // ridges beyond the tree mesh, which cannot be judged in
                    // an extreme trunk-only close-up.
                    root + Vec3::new(azimuth.sin() * 3.8, 1.3, azimuth.cos() * 3.8),
                    root + Vec3::Y * 0.08,
                    Vec3::Y,
                )
            },
        )
}

pub(super) fn branch_junction(state: &SceneCaptureState) -> (Vec3, Vec3, Vec3) {
state.tree_focus.map_or(
            (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
            |tree| {
                let tree = tree.metres();
                let junction = tree + Vec3::Y * 0.15;
                let azimuth = state.tree_review_azimuth_degrees.to_radians();
                (
                    junction + Vec3::new(azimuth.sin() * 3.2, 0.65, azimuth.cos() * 3.2),
                    junction,
                    Vec3::Y,
                )
            },
        )
}

pub(super) fn rock(state: &SceneCaptureState) -> (Vec3, Vec3, Vec3) {
state.rock_focus.map_or(
            (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
            |rock| {
                let rock = rock.metres();
                let azimuth = state.tree_review_azimuth_degrees.to_radians();
                (
                    rock + Vec3::new(azimuth.sin() * 3.0, 1.1, azimuth.cos() * 3.0),
                    rock - Vec3::Y * 0.15,
                    Vec3::Y,
                )
            },
        )
}

pub(super) fn ground_cover(state: &SceneCaptureState) -> (Vec3, Vec3, Vec3) {
state.tree_focus.map_or(
            (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
            |tree| {
                let tree = tree.metres();
                (
                    tree + Vec3::new(5.5, -0.4, 5.5),
                    tree + Vec3::new(0.0, -2.25, 0.0),
                    Vec3::Y,
                )
            },
        )
}

pub(super) fn tree_lod(state: &SceneCaptureState, distance: f32) -> (Vec3, Vec3, Vec3) {
    state.tree_focus.map_or(
        (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
        |tree| {
                let tree = tree.metres();
            let diagonal = distance * 1.55 * core::f32::consts::FRAC_1_SQRT_2;
            (
                tree + Vec3::new(diagonal, 7.55, diagonal),
                tree + Vec3::new(0.0, 7.0, 0.0),
                Vec3::Y,
            )
        },
    )
}

pub(super) fn tree_cold_traversal(state: &SceneCaptureState, distance: f32) -> (Vec3, Vec3, Vec3) {
    state.tree_focus.map_or(
        (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
        |tree| {
                let tree = tree.metres();
            let horizontal = distance.max(0.75) * core::f32::consts::FRAC_1_SQRT_2;
            (
                tree + Vec3::new(horizontal, 7.55, horizontal),
                tree + Vec3::new(0.0, 7.0, 0.0),
                Vec3::Y,
            )
        },
    )
}


pub(super) fn lit_tree_review(
    state: &SceneCaptureState,
    species: Option<TreePresentationSpecies>,
) -> (Transform, Vec3) {
                let tree = state.tree_focus.map(|tree| tree.metres()).unwrap_or(state.obstacle_focus);
                let azimuth = state.tree_review_azimuth_degrees.to_radians();
                let (radius, camera_height, target_height) = if species == Some(TreePresentationSpecies::CommonBeech) {
                    (30.0, 10.0, 8.0)
                } else {
                    (18.0, 6.0, 4.5)
                };
                let target = tree + Vec3::Y * target_height;
                let position = tree
                    + Vec3::new(
                        azimuth.sin() * radius,
                        camera_height,
                        azimuth.cos() * radius,
                    );
                (
                    Transform::from_translation(position).looking_at(target, Vec3::Y),
                    target,
                )
}

pub(super) fn lit_branch_junction(
    state: &SceneCaptureState,
    species: Option<TreePresentationSpecies>,
) -> (Transform, Vec3) {
                let tree = state.tree_focus.map(|tree| tree.metres()).unwrap_or(state.obstacle_focus);
                let azimuth = state.tree_review_azimuth_degrees.to_radians();
                let (height, radius, camera_lift) = if species == Some(TreePresentationSpecies::CommonBeech) {
                    // Closed-canopy beech carries its first live scaffold limbs
                    // well above the oak junction sampled by the old camera.
                    (4.5, 8.0, 2.0)
                } else {
                    (0.8, 6.0, 1.4)
                };
                let target = tree + Vec3::Y * height;
                let position =
                    target + Vec3::new(azimuth.sin() * radius, camera_lift, azimuth.cos() * radius);
                (
                    Transform::from_translation(position).looking_at(target, Vec3::Y),
                    target,
                )
}

pub(super) fn camera_for_view(pose: CapturePose, state: &SceneCaptureState) -> CaptureCameraResult<(Transform, Vec3)> {
    let half = state.terrain.width_metres.max(state.terrain.depth_metres) * 0.5;
    let (position, target, up) = match pose {
        CapturePose::Plant { .. } | CapturePose::Fungus { .. } | CapturePose::PlantLod { .. } => {
            pose.plant_camera(state.plant_focus.ok_or(CaptureCameraError::MissingPlant)?)
        }
        CapturePose::Ground => (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
        CapturePose::AnimationPlay { yaw_degrees } => {
            let yaw = Quat::from_rotation_y(yaw_degrees.to_radians());
            let focus = state.animation_play_focus;
            let position = focus + yaw * Vec3::Z * 3.75;
            (position, focus, Vec3::Y)
        }
        CapturePose::AnimationPlayBoundary { .. } | CapturePose::AnimationPlayObstruction { .. } => {
            return Err(CaptureCameraError::RequiresLiveQuery { pose });
        }
        CapturePose::BuildingInterior { camera } => {
            let camera = state
                .building_interior_cameras
                .get(usize::from(camera))
                .ok_or(CaptureCameraError::MissingBuildingCamera { pose })?;
            (camera.position, camera.target, Vec3::Y)
        }
        CapturePose::CityExterior { camera } => {
            let camera = state
                .city_exterior_cameras
                .get(usize::from(camera))
                .ok_or(CaptureCameraError::MissingBuildingCamera { pose })?;
            (camera.position, camera.target, Vec3::Y)
        }
        CapturePose::TreeColdTraversal { distance } => tree_cold_traversal(state, distance),
        CapturePose::TreeReview => tree_review(state),
        CapturePose::RecursiveTree => recursive_tree(state),
        CapturePose::Root => tree_root(state),
        CapturePose::BranchJunction => branch_junction(state),
        CapturePose::Rock => rock(state),
        CapturePose::TerrainGrazing => landscape::terrain_grazing(state),
        CapturePose::FaultScarp | CapturePose::FaultScarpSeam | CapturePose::LandformUnderside => {
            pose.fault_camera().ok_or(CaptureCameraError::MissingAuthoredCamera { pose })?
        }
        CapturePose::GrassSeam => {
            let target = state.obstacle_focus - Vec3::Y * 1.30;
            (target + Vec3::new(-3.4, 1.25, 3.4), target, Vec3::Y)
        }
        CapturePose::Debris => state.debris_focus.map_or(
            (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
            |target| {
                debris_detail_camera(
                    target,
                    state.debris_camera,
                    state.tree_review_azimuth_degrees,
                )
            },
        ),
        CapturePose::GroundCover => ground_cover(state),
        CapturePose::LeafSpecimen => state.tree_leaf_focus.map_or(
            (state.ground_eye_position, state.ground_eye_target, Vec3::Y),
            |focus| {
                (
                    state.tree_leaf_camera.unwrap_or(focus + Vec3::Z),
                    focus,
                    Vec3::Y,
                )
            },
        ),
        CapturePose::UnderstoryReview { .. } => {
            return Err(CaptureCameraError::RequiresLiveQuery { pose });
        }
        CapturePose::TreeLod { distance } => tree_lod(state, distance),
        CapturePose::Overhead => {
            let frame_half = half.max(state.city_half_extent_metres + 30.0);
            (
                state.obstacle_focus + Vec3::new(0.0, frame_half * 1.75, frame_half * 0.08),
                state.obstacle_focus,
                Vec3::Z,
            )
        }
        CapturePose::CityOblique { azimuth_degrees } => landscape::city_oblique(state, half, azimuth_degrees),
        CapturePose::Horizon => landscape::horizon(state, half),
        CapturePose::VistaPeak => landscape::vista_peak(state, half)?,
        CapturePose::VistaValley => landscape::vista_valley(state, half)?,
    };
    Ok((
        Transform::from_translation(position).looking_at(target, up),
        target,
    ))
}
