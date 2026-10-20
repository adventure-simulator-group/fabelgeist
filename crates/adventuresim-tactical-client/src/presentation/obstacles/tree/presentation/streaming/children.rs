//! Attach only requested resident recipe representations to physical roots.

use super::*;
use anyhow::{Context, Result};
use bevy::ecs::hierarchy::ChildSpawnerCommands;

// Exact triangle counts of the existing recipe leaf meshes, used at the
// native mesh-count diagnostic boundary; this does not set mesh quality.
const CAMBERED_TRIANGLES_PER_LEAF: usize = 8;
const CARD_TRIANGLES_PER_LEAF: usize = 2;

struct ClusterFrame {
    marker: TreeLodCluster,
    bounds: Aabb,
    leaves: usize,
}

pub(super) fn spawn_streamed_tree_children(
    commands: &mut Commands,
    entity: Entity,
    cached: &CachedTreePresentation,
    new_mask: u8,
    new_leaf_mask: u8,
) -> Result<()> {
    let mut result = Ok(());
    commands.entity(entity).with_children(|parent| {
        result = (|| {
            // Trunk visibility must not hide the canopy-card sibling.
            if new_mask & 1 != 0 {
                spawn_trunks(parent, cached)?;
            }
            if new_mask & (1 << 1) != 0 || new_leaf_mask != 0 {
                spawn_clusters(parent, cached, new_mask, new_leaf_mask)?;
            }
            spawn_aggregate_canopies(parent, cached, new_mask)?;
            if new_mask & (1 << 5) != 0 {
                spawn_whole_card(parent, cached)?;
            }
            Ok(())
        })();
    });
    result
}

pub(super) fn aggregate_tree_wood_material(
    material: Handle<TacticalTreeAggregateBarkMaterial>,
) -> MeshMaterial3d<TacticalTreeAggregateBarkMaterial> {
    MeshMaterial3d(material)
}

pub(super) fn aggregate_tree_wood_casts_shadows(lod: u8) -> bool {
    lod != 2
}

fn spawn_clusters(
    parent: &mut ChildSpawnerCommands,
    cached: &CachedTreePresentation,
    new_mask: u8,
    new_leaf_mask: u8,
) -> Result<()> {
    for cluster in cached
        .clusters
        .as_ref()
        .context("requested detailed crown assets are resident")?
    {
        let frame = ClusterFrame {
            marker: TreeLodCluster {
                primary_group: cluster.primary_group,
                center: cluster.center,
                radius: cluster.radius,
            },
            bounds: tree_cluster_aabb(cluster.center, cluster.radius),
            leaves: cached
                .leaves
                .iter()
                .filter(|leaf| leaf.primary_group == cluster.primary_group)
                .count(),
        };
        if new_leaf_mask & 1 != 0 {
            spawn_cambered_leaves(parent, cached, cluster, &frame)?;
        }
        if new_leaf_mask & 2 != 0 {
            spawn_leaf_cards(parent, cached, cluster, &frame)?;
        }
        if new_mask & (1 << 1) != 0 {
            spawn_cluster_wood(parent, cached, cluster, &frame)?;
        }
    }
    Ok(())
}

fn spawn_trunks(parent: &mut ChildSpawnerCommands, cached: &CachedTreePresentation) -> Result<()> {
    parent.spawn((
        Name::new(format!("{} detailed trunk and roots", cached.species_name)),
        StreamedTreeChild,
        TreeTrunkLod,
        PlayableTreeTrunk,
        PlayableTreeDetailedTrunk,
        Mesh3d(
            cached
                .trunk_mesh
                .clone()
                .context("requested trunk mesh is resident")?,
        ),
        MeshMaterial3d(cached.bark_material.clone()),
        tree_trunk_visibility(),
    ));
    parent.spawn((
        Name::new(format!("{} mid-distance trunk", cached.species_name)),
        StreamedTreeChild,
        TreeTrunkLod,
        PlayableTreeTrunk,
        PlayableTreeMidTrunk,
        Mesh3d(
            cached
                .mid_trunk_mesh
                .clone()
                .context("requested mid trunk mesh is resident")?,
        ),
        MeshMaterial3d(cached.bark_material.clone()),
        tree_mid_trunk_visibility(),
    ));
    Ok(())
}

fn spawn_cambered_leaves(
    parent: &mut ChildSpawnerCommands,
    cached: &CachedTreePresentation,
    cluster: &CachedTreeClusterPresentation,
    frame: &ClusterFrame,
) -> Result<()> {
    parent.spawn((
        Name::new(format!(
            "{} scaffold {} cambered PBR leaves",
            cached.species_name, cluster.primary_group
        )),
        StreamedTreeChild,
        TreeLod(0),
        frame.marker,
        frame.bounds,
        TreeLeafRepresentation::TexturedMesh,
        PlayableTreeDetailedLeaves,
        TreeLeafTriangleCount(frame.leaves * CAMBERED_TRIANGLES_PER_LEAF),
        Mesh3d(
            cluster
                .cambered_leaf_mesh
                .clone()
                .context("requested cambered leaf mesh is resident")?,
        ),
        MeshMaterial3d(cached.leaf_material.clone()),
        tree_leaf_visibility(TreeLeafRepresentation::TexturedMesh, 1.0, cluster.radius),
    ));
    Ok(())
}

fn spawn_leaf_cards(
    parent: &mut ChildSpawnerCommands,
    cached: &CachedTreePresentation,
    cluster: &CachedTreeClusterPresentation,
    frame: &ClusterFrame,
) -> Result<()> {
    parent.spawn((
        Name::new(format!(
            "{} scaffold {} PBR leaf cards",
            cached.species_name, cluster.primary_group
        )),
        StreamedTreeChild,
        TreeLod(0),
        frame.marker,
        frame.bounds,
        TreeLeafRepresentation::AlphaCard,
        PlayableTreeDetailedLeaves,
        TreeLeafTriangleCount(frame.leaves * CARD_TRIANGLES_PER_LEAF),
        Mesh3d(
            cluster
                .leaf_card_mesh
                .clone()
                .context("requested leaf card mesh is resident")?,
        ),
        MeshMaterial3d(cached.leaf_material.clone()),
        tree_leaf_visibility(TreeLeafRepresentation::AlphaCard, 1.0, cluster.radius),
    ));
    Ok(())
}

fn spawn_cluster_wood(
    parent: &mut ChildSpawnerCommands,
    cached: &CachedTreePresentation,
    cluster: &CachedTreeClusterPresentation,
    frame: &ClusterFrame,
) -> Result<()> {
    parent.spawn((
        Name::new(format!(
            "{} scaffold {} terminal buds",
            cached.species_name, cluster.primary_group
        )),
        StreamedTreeChild,
        TreeLod(0),
        PlayableTreeBuds,
        frame.marker,
        frame.bounds,
        Mesh3d(
            cluster
                .bud_mesh
                .clone()
                .context("requested bud mesh is resident")?,
        ),
        MeshMaterial3d(cached.bud_material.clone()),
        tree_lod_visibility(0),
    ));
    parent.spawn((
        Name::new(format!(
            "{} scaffold {} detailed wood",
            cached.species_name, cluster.primary_group
        )),
        StreamedTreeChild,
        TreeLod(0),
        PlayableTreeDetailedWood,
        frame.marker,
        frame.bounds,
        Mesh3d(
            cluster
                .detailed_branch_mesh
                .clone()
                .context("requested detailed branch mesh is resident")?,
        ),
        MeshMaterial3d(cached.bark_material.clone()),
        tree_lod_visibility(0),
    ));
    Ok(())
}

fn spawn_aggregate_canopies(
    parent: &mut ChildSpawnerCommands,
    cached: &CachedTreePresentation,
    new_mask: u8,
) -> Result<()> {
    for lod in 1..=3 {
        if new_mask & (1 << (lod + 1)) == 0 {
            continue;
        }
        let card = cached.lod_cards[lod as usize - 1]
            .as_ref()
            .context("requested aggregate tree card is resident")?;
        parent.spawn((
            Name::new(tree_lod_name(lod, true)),
            StreamedTreeChild,
            TreeLod(lod),
            PlayableTreeCanopyCard,
            NotShadowCaster,
            Mesh3d(card.mesh.clone()),
            MeshMaterial3d(card.material.clone()),
            tree_lod_visibility(lod),
        ));
        if lod <= 2 {
            let mut aggregate_wood = parent.spawn((
                Name::new(tree_lod_name(lod, false)),
                StreamedTreeChild,
                TreeLod(lod),
                PlayableTreeAggregateWood,
                Mesh3d(
                    cached.aggregate_branch_meshes[lod as usize - 1]
                        .clone()
                        .context("requested aggregate branch mesh is resident")?,
                ),
                aggregate_tree_wood_material(cached.aggregate_bark_material.clone()),
                tree_lod_visibility(lod),
            ));
            if !aggregate_tree_wood_casts_shadows(lod) {
                aggregate_wood.insert(NotShadowCaster);
            }
        }
    }
    Ok(())
}

fn spawn_whole_card(
    parent: &mut ChildSpawnerCommands,
    cached: &CachedTreePresentation,
) -> Result<()> {
    let card = cached.lod_cards[3]
        .as_ref()
        .context("requested whole-tree card is resident")?;
    parent.spawn((
        Name::new(tree_lod_name(4, true)),
        StreamedTreeChild,
        TreeLod(4),
        PlayableTreeCanopyCard,
        NoFrustumCulling,
        NotShadowCaster,
        Mesh3d(card.mesh.clone()),
        MeshMaterial3d(card.material.clone()),
        card.provenance.clone(),
        tree_lod_visibility(4),
    ));
    Ok(())
}

fn tree_cluster_aabb(center: Vec3, radius: f32) -> Aabb {
    let extent = Vec3::splat(radius.max(0.01));
    Aabb::from_min_max(center - extent, center + extent)
}
