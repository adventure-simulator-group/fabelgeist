//! A ring over every fighter's head: the incapacitation wheel, red as far round as their injury
//! goes, the whole circle at [`Injury::DOWN`](crate::Injury::DOWN). Nothing it shows is hidden
//! anywhere else: injury is the one thing that takes a fighter out of the fight.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{color::palettes::css, prelude::*};
use bevy_netahoy::{OwnPlayer, OwnVisual, PlayerShape, RemoteVisual};

use crate::Vitals;

/// The ring's radius (m) and how far above the fighter's centre it floats (m).
const RADIUS: f32 = 0.18;
const ABOVE: f32 = 0.2;

pub(super) struct RingsPlugin;

impl Plugin for RingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_rings.after(TransformSystems::Propagate));
    }
}

fn draw_rings(
    mut gizmos: Gizmos,
    shape: Res<PlayerShape>,
    camera: Single<&GlobalTransform, With<Camera3d>>,
    own_visuals: Query<&GlobalTransform, With<OwnVisual>>,
    own_vitals: Query<&Vitals, With<OwnPlayer>>,
    remotes: Query<(&GlobalTransform, &RemoteVisual)>,
    vitals: Query<&Vitals>,
) {
    let own = own_visuals
        .iter()
        .zip(own_vitals.iter())
        .map(|(at, vitals)| (at.translation(), *vitals));
    let others = remotes.iter().filter_map(|(at, remote)| {
        Some((at.translation(), *vitals.get(remote.server_entity).ok()?))
    });
    for (at, vitals) in own.chain(others) {
        let center = at + Vec3::Y * (shape.half_height + ABOVE);
        // Face the camera.
        let toward = (camera.translation() - center).normalize_or_zero();
        let rotation = Transform::IDENTITY.looking_to(-toward, Vec3::Y).rotation;
        let isometry = Isometry3d::new(center, rotation);
        gizmos.circle(isometry, RADIUS, css::DARK_SLATE_GRAY);
        let injury = vitals.injury.fraction();
        if injury > 0.0 {
            // An arc lies across its isometry's Y axis, the circle across its Z: turn it into the
            // circle's plane.
            let arc = Isometry3d::new(center, rotation * Quat::from_rotation_x(FRAC_PI_2));
            gizmos.arc_3d(injury * TAU, RADIUS * 0.85, arc, css::RED);
        }
    }
}
