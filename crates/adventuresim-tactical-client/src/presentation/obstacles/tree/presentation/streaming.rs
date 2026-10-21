//! Recipe demand residency and child attachment at the existing tree streamer.

use super::*;
use recipe::{RecipeDemand, RecipeStreamingAssets};

mod children;
mod recipe;

pub(super) fn spawn_cached_tree(
    commands: &mut Commands,
    entity: Entity,
    cache_key: Seed,
    species_name: &'static str,
) {
    commands.entity(entity).insert((
        Name::new(format!("Presented mature {species_name}")),
        PresentedTree,
        Visibility::Inherited,
        StreamedTreePresentation::Recipe(RecipeTreeStream {
            cache_key,
            resident_mask: 0,
            resident_leaf_mask: 0,
            active_mask: u8::MAX,
            active_leaf: None,
        }),
    ));
}

pub(in crate::presentation) fn stream_tree_lod_children(
    mut commands: Commands,
    camera: Single<(&GlobalTransform, &Projection), With<TacticalGameplayCamera>>,
    lod_override: Res<TreeLodRenderOverride>,
    mut trees: Query<(Entity, &GlobalTransform, &mut StreamedTreePresentation)>,
    mut assets: RecipeStreamingAssets,
    mut tree_cache: ResMut<TreePresentationCache>,
) -> bevy::prelude::Result {
    let focal_scale = match camera.1 {
        Projection::Perspective(projection) => {
            (80.0_f32.to_radians() * 0.5).tan() / (projection.fov * 0.5).tan()
        }
        _ => 1.0,
    } * lod_override.projected_scale.unwrap_or(1.0);
    for (entity, transform, mut presentation) in &mut trees {
        match &mut *presentation {
            StreamedTreePresentation::Recipe(recipe) => {
                let distance = camera.0.translation().distance(transform.translation());
                let demand =
                    RecipeDemand::for_distance_metres(distance, focal_scale, &lod_override);
                assets.stream(
                    &mut commands,
                    entity,
                    recipe,
                    demand,
                    tree_cache.recipe_mut()?,
                )?;
            }
            #[cfg(feature = "grown-trees")]
            StreamedTreePresentation::Grown(_) => {}
        }
    }
    Ok(())
}

fn tree_streamed_leaf_representation(scaled_distance: f32) -> Option<TreeLeafRepresentation> {
    // Keep residency decisions on the same projected-scale-aware base band as
    // tree_leaf_visibility. The cluster-specific visibility range may widen
    // this overlap, but it must never request a representation after its
    // corresponding visibility range has become active.
    if scaled_distance < TREE_LEAF_HANDOFF_START {
        Some(TreeLeafRepresentation::TexturedMesh)
    } else if scaled_distance > TREE_LEAF_HANDOFF_END {
        Some(TreeLeafRepresentation::AlphaCard)
    } else {
        None
    }
}

fn tree_streaming_mask(scaled_distance: f32) -> u8 {
    // Generate each incoming crown tier one full outgoing LOD band before it
    // becomes visible. Mesh/material insertion is asynchronous in the render
    // world: the old four-metre lead could leave only the already-resident
    // trunk on a first approach, while a second approach was correct because
    // the crown assets remained cached. This changes when eventual residency
    // is paid, not which tiers are drawn or retained.
    u8::from(scaled_distance < 75.0)
        | (u8::from(scaled_distance < 24.0) << 1)
        | (u8::from((3.0..44.0).contains(&scaled_distance)) << 2)
        | (u8::from((8.0..75.0).contains(&scaled_distance)) << 3)
        | (u8::from((18.0..100.0).contains(&scaled_distance)) << 4)
        | (u8::from((40.0..220.0).contains(&scaled_distance)) << 5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use children::{aggregate_tree_wood_casts_shadows, aggregate_tree_wood_material};

    #[test]
    fn streaming_keeps_adjacent_tree_lods_resident_through_every_handoff() {
        for (distance, required_bits) in [
            (7.0, (1 << 1) | (1 << 2)),
            (14.0, (1 << 2) | (1 << 3)),
            (28.0, (1 << 3) | (1 << 4)),
            (55.0, (1 << 4) | (1 << 5)),
        ] {
            assert_eq!(tree_streaming_mask(distance) & required_bits, required_bits);
        }
    }

    #[test]
    fn streaming_prefetches_each_incoming_crown_one_outgoing_band_early() {
        for (distance, incoming_lod_bit) in [
            (90.0, 1 << 4),
            (72.0, 1 << 3),
            (42.0, 1 << 2),
            (22.0, 1 << 1),
        ] {
            assert_ne!(tree_streaming_mask(distance) & incoming_lod_bit, 0);
        }
        assert_eq!(tree_streaming_mask(120.0), 1 << 5);
    }

    #[test]
    fn leaf_residency_only_overlaps_near_the_representation_handoff() {
        assert_eq!(
            tree_streamed_leaf_representation(1.4),
            Some(TreeLeafRepresentation::TexturedMesh)
        );
        assert_eq!(tree_streamed_leaf_representation(2.0), None);
        assert_eq!(
            tree_streamed_leaf_representation(2.6),
            Some(TreeLeafRepresentation::AlphaCard)
        );
    }

    #[test]
    fn aggregate_tree_children_use_the_cheap_bark_material_tier() {
        let mut world = World::new();
        let entity = world
            .spawn((
                PlayableTreeAggregateWood,
                aggregate_tree_wood_material(Handle::default()),
            ))
            .id();
        let entity = world.entity(entity);
        assert!(entity.contains::<MeshMaterial3d<TacticalTreeAggregateBarkMaterial>>());
        assert!(!entity.contains::<MeshMaterial3d<TacticalTreeBarkMaterial>>());
    }

    #[test]
    fn only_lod2_aggregate_wood_is_excluded_from_shadow_casting() {
        let mut world = World::new();
        for lod in [1, 2] {
            let mut entity = world.spawn((
                PlayableTreeAggregateWood,
                TreeLod(lod),
                aggregate_tree_wood_material(Handle::default()),
            ));
            if !aggregate_tree_wood_casts_shadows(lod) {
                entity.insert(NotShadowCaster);
            }
        }

        let mut aggregate_wood = world.query::<(&TreeLod, Has<NotShadowCaster>)>();
        let mut policies = aggregate_wood
            .iter(&world)
            .map(|(lod, not_shadow_caster)| (lod.0, not_shadow_caster))
            .collect::<Vec<_>>();
        policies.sort_unstable_by_key(|(lod, _)| *lod);

        assert_eq!(policies, [(1, false), (2, true)]);
        assert!(aggregate_tree_wood_casts_shadows(1));
        assert!(!aggregate_tree_wood_casts_shadows(2));
    }
}
