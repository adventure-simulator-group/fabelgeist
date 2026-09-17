//! Celestial lighting written into the materials that shade from it directly.
//!
//! Bark, tree impostors, vista cards, pebble billboards and foliage all carry
//! their own sun direction and ambient response rather than reading Bevy's
//! lights, so one system keeps them in step with the presented celestial
//! snapshot. It lives apart from any one of them because it owns none of them.

use super::PresentedCelestialLighting;
use super::ground_scatter::{TacticalFoliageMaterial, TacticalPebbleBillboardMaterial};
use super::obstacles::tree::{TacticalTreeBarkMaterial, TacticalTreeImpostorMaterial};
use super::vista::tree_cards::TacticalVistaTreeInstancedMaterial;
use bevy::prelude::*;

pub(in crate::presentation) fn update_celestial_material_lighting(
    celestial: Res<PresentedCelestialLighting>,
    mut bark_materials: ResMut<Assets<TacticalTreeBarkMaterial>>,
    mut impostor_materials: ResMut<Assets<TacticalTreeImpostorMaterial>>,
    mut vista_card_materials: ResMut<
        Assets<crate::presentation::vista::tree_cards::TacticalVistaTreeInstancedMaterial>,
    >,
    mut pebble_materials: ResMut<Assets<TacticalPebbleBillboardMaterial>>,
    mut written: Local<Option<(Vec4, Vec4)>>,
    mut foliage_materials: ResMut<Assets<TacticalFoliageMaterial>>,
) {
    if !celestial.is_changed() {
        return;
    }
    let Some(celestial) = celestial.snapshot.as_ref() else {
        return;
    };
    let direction = if celestial.sun_altitude_degrees > -6.0 {
        celestial.sun_direction
    } else if celestial.moon_altitude_degrees > -2.0 {
        celestial.moon_direction
    } else {
        Vec3::new(0.25, 0.92, 0.3).normalize()
    };
    // The presented snapshot refreshes on every replicated environment tick,
    // but celestial motion is glacial on gameplay timescales. Rewriting the
    // material assets dirties them and re-queues every tree and pebble, so
    // skip the write until the change would be visible.
    let lighting = direction.extend(celestial.material_light_factor);
    let ambient = celestial
        .ambient_color
        .extend(celestial.material_ambient_response);
    if written.is_some_and(|(written_lighting, written_ambient)| {
        written_lighting.distance_squared(lighting) < 1e-6
            && written_ambient.distance_squared(ambient) < 1e-6
    }) {
        return;
    }
    *written = Some((lighting, ambient));
    for (_, material) in bark_materials.iter_mut() {
        material.extension.lighting = lighting;
    }
    for (_, material) in impostor_materials.iter_mut() {
        material.lighting = lighting;
        material.ambient = ambient;
    }
    for (_, material) in vista_card_materials.iter_mut() {
        material.lighting = lighting;
        material.ambient = ambient;
    }
    for (_, material) in pebble_materials.iter_mut() {
        material.lighting = lighting;
        material.ambient = ambient;
    }
    for (_, material) in foliage_materials.iter_mut() {
        if material.quality.x > 0.5 {
            material.lighting = direction.extend(celestial.material_light_factor);
            material.ambient = celestial
                .ambient_color
                .extend(celestial.material_ambient_response);
        }
    }
}
