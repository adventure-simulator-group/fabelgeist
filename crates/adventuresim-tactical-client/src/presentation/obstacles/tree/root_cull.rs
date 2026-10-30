//! Frustum culling of whole trees at the obstacle root.
//!
//! Every LOD child carries its own `Aabb` and a `VisibilityRange` whose fade
//! margins drive the dithered crossfade, so `check_visibility_cpu_culling`
//! spends a range lookup and two frustum tests on each of them - and one mature
//! tree resolves to a trunk pair plus an entity per crown cluster per leaf
//! representation. Hiding the root instead lets that system reject the whole
//! subtree on its opening `InheritedVisibility` check, before any of that work.
//!
//! This deliberately does not mark the children `NoCpuCulling`: that component
//! also removes them from `check_visibility_ranges`, which is what selects the
//! LOD, so every representation would render at once. Trees in view keep the
//! unchanged pipeline; only trees the camera cannot see get cheaper.

use bevy::camera::primitives::{Frustum, Sphere};
use bevy::prelude::*;

use super::presentation::StreamedTreePresentation;
use crate::presentation::TacticalGameplayCamera;

/// Radius about the obstacle origin that bounds any authored tree.
///
/// The tallest species stands 16 m with a 6 m crown radius, so a sphere at the
/// base needs about 17 m; the margin beyond that absorbs per-instance scale.
/// Erring large costs a few off-screen trees their saving rather than popping a
/// crown at the edge of the frustum.
const TREE_BOUNDING_RADIUS_METRES: f32 = 24.0;

pub(in crate::presentation) fn cull_tree_roots(
    camera: Option<Single<&Frustum, With<TacticalGameplayCamera>>>,
    mut trees: Query<(&GlobalTransform, &mut Visibility), With<StreamedTreePresentation>>,
) {
    let Some(frustum) = camera else {
        return;
    };
    for (transform, mut visibility) in &mut trees {
        let bounds = Sphere {
            center: transform.translation().into(),
            radius: TREE_BOUNDING_RADIUS_METRES,
        };
        let desired = if frustum.intersects_sphere(&bounds, false) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        // Writing unconditionally would re-dirty every tree each frame and make
        // the propagation pass walk subtrees that did not change.
        if *visibility != desired {
            *visibility = desired;
        }
    }
}
