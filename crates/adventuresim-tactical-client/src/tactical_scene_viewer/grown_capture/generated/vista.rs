//! Render-only vista ownership joins the ordinary observer without physics.
use super::super::ordinary_batches::inventory::{
    CurrentInventory, Member, PartIdentity, PartOrdinal, RootMember, RootOrigin,
};
use super::*;
use crate::presentation::{
    GrownSceneAssets, GrownVistaPart, VistaTreeCensus, VistaTreePresentation,
};
use adventuresim_environment::render::grown::{
    GrownLeafMaterial, GrownMaterialHandle, GrownPartAsset, GrownWoodMaterial,
};
use bevy::{
    camera::{
        primitives::Aabb,
        visibility::{NoAutoAabb, VisibilityRange, VisibleEntities},
    },
    ecs::system::SystemParam,
    render::batching::NoAutomaticBatching,
};

#[derive(Default, PartialEq)]
pub(super) struct HeldVista {
    roots: Vec<Root>,
}

#[derive(PartialEq)]
struct Root {
    entity: Entity,
    frame: Mat4,
    source: crate::presentation::GrownSource,
    parts: Vec<Member>,
    ranges: Vec<VisibilityRange>,
}

struct ExpectedPart<'a> {
    scene: Entity,
    root: Entity,
    frame: Mat4,
    identity: PartIdentity,
    asset: &'a GrownPartAsset,
}

struct CheckedPart {
    member: Member,
    range: VisibilityRange,
}

#[derive(bevy::ecs::query::QueryData)]
struct GroundRoot {
    local: &'static Transform,
    global: &'static GlobalTransform,
    children: &'static Children,
    visibility: &'static Visibility,
    parent: Option<&'static ChildOf>,
    physical: Has<adventuresim_tactical_core::prelude::SceneObstacle>,
    streamed: Has<crate::presentation::PresentedTree>,
}

#[derive(bevy::ecs::query::QueryData)]
struct VistaPart {
    binding: &'static GrownVistaPart,
    mesh: &'static Mesh3d,
    local: &'static Transform,
    global: &'static GlobalTransform,
    parent: &'static ChildOf,
    children: Option<&'static Children>,
    visibility: &'static Visibility,
    range: &'static VisibilityRange,
    bounds: &'static Aabb,
    source_bounds: Has<NoAutoAabb>,
    singleton: Has<NoAutomaticBatching>,
    no_shadow: Has<bevy::light::NotShadowCaster>,
    wood: Option<&'static MeshMaterial3d<GrownWoodMaterial>>,
    leaves: Option<&'static MeshMaterial3d<GrownLeafMaterial>>,
}

#[derive(SystemParam)]
pub(super) struct Inspector<'w, 's> {
    census: Query<'w, 's, &'static VistaTreeCensus>,
    roots: Query<'w, 's, GroundRoot, With<VistaTreePresentation>>,
    parts: Query<'w, 's, VistaPart>,
    entities: Query<'w, 's, (Entity, &'static GrownVistaPart)>,
}

impl HeldVista {
    pub(super) fn admit(world: &mut World, assets: &GrownSceneAssets) -> Result<Self> {
        let mut state = SystemState::<Inspector>::new(world);
        state.get(world)?.snapshot(assets, None)
    }

    pub(super) fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.roots
            .iter()
            .flat_map(|root| root.parts.iter().map(|part| part.entity))
    }

    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"purpose":"vista_ground", "roots":self.roots.len(),
            "parts":self.entities().count(), "level":Level::Coarse,
            "source_counts":self.roots.iter().map(|root|root.source).fold(
                std::collections::BTreeMap::new(), |mut counts, source| {
                    *counts.entry(format!("{source:?}")).or_insert(0_usize) += 1; counts
                }),
            "frame":"complete scaled tactical-local ground-origin matrix; no physical collider offset"})
    }
}

impl Inspector<'_, '_> {
    pub(super) fn check(
        &self,
        held: &HeldVista,
        assets: &GrownSceneAssets,
        visible: &VisibleEntities,
        demand: Option<&demand::Demand>,
        inventory: &mut CurrentInventory,
    ) -> Result<()> {
        let actual = self.snapshot(assets, demand)?;
        ensure!(
            actual == *held,
            "held vista ownership/frame/parts/ranges changed"
        );
        let visible = visible.get(std::any::TypeId::of::<Mesh3d>());
        for root in &held.roots {
            inventory.roots.push(RootMember {
                entity: root.entity,
                origin: RootOrigin::Vista {
                    source: root.source,
                },
                frame: root.frame,
            });
            for part in &root.parts {
                if visible.contains(&part.entity) {
                    inventory.visible.push(part.entity);
                }
                inventory.members.push(*part);
            }
        }
        Ok(())
    }

    fn snapshot(
        &self,
        assets: &GrownSceneAssets,
        demand: Option<&demand::Demand>,
    ) -> Result<HeldVista> {
        let census = self
            .census
            .get(assets.scene())
            .context("vista producer census is not installed")?;
        let mut roots = Vec::new();
        let mut members = Vec::new();
        for entity in &census.roots {
            ensure!(
                !roots.iter().any(|root: &Root| root.entity == *entity),
                "duplicate vista root"
            );
            let root = self.root(*entity, assets, demand)?;
            for part in &root.parts {
                ensure!(
                    !members.contains(&part.entity),
                    "duplicate vista part owner"
                );
                members.push(part.entity);
            }
            roots.push(root);
        }
        let mut declared = census.parts.clone();
        declared.sort_unstable();
        members.sort_unstable();
        ensure!(
            declared == members && declared.windows(2).all(|pair| pair[0] != pair[1]),
            "vista producer part census is not a bijection"
        );
        ensure!(
            self.entities
                .iter()
                .filter(|(_, part)| part.0.scene == assets.scene())
                .count()
                == members.len(),
            "extra/missing scene-owned vista part"
        );
        ensure!(
            self.roots.iter().count() == roots.len(),
            "extra/missing vista root"
        );
        Ok(HeldVista { roots })
    }

    fn root(
        &self,
        entity: Entity,
        assets: &GrownSceneAssets,
        demand: Option<&demand::Demand>,
    ) -> Result<Root> {
        let actual = self.roots.get(entity)?;
        let frame = actual.local.to_matrix();
        ensure!(
            frame.is_finite()
                && actual.global.to_matrix() == frame
                && actual.local.rotation == Quat::IDENTITY
                && actual.local.scale.x > 0.0
                && actual.local.scale == Vec3::splat(actual.local.scale.x)
                && actual.parent.is_none()
                && !actual.physical
                && !actual.streamed
                && (matches!(
                    *actual.visibility,
                    Visibility::Inherited | Visibility::Visible
                ) || *actual.visibility == Visibility::Hidden
                    && demand.is_some_and(|value| value.allows_hidden(entity))),
            "vista ground root frame/hierarchy/visibility/physical policy changed"
        );
        let source = assets.catalogue().select(actual.local.translation);
        let closure = assets.catalogue().assets(source, Level::Coarse)?;
        ensure!(
            actual.children.len() == closure.parts().len(),
            "vista source part cardinality differs"
        );
        let mut parts = Vec::new();
        let mut ranges = Vec::new();
        for (ordinal, asset) in closure.parts().enumerate() {
            let identity = PartIdentity {
                bank: closure.bank().id(),
                source,
                level: Level::Coarse,
                encoding: closure.encoding(),
                ordinal: PartOrdinal(ordinal),
                role: asset.role(),
                records: asset.records(),
            };
            let checked = self.part(
                &ExpectedPart {
                    scene: assets.scene(),
                    root: entity,
                    frame,
                    identity,
                    asset,
                },
                actual.children,
                demand,
            )?;
            parts.push(checked.member);
            ranges.push(checked.range);
        }
        Ok(Root {
            entity,
            frame,
            source,
            parts,
            ranges,
        })
    }

    fn part(
        &self,
        expected: &ExpectedPart<'_>,
        children: &Children,
        demand: Option<&demand::Demand>,
    ) -> Result<CheckedPart> {
        let matching = children
            .iter()
            .filter(|child| {
                self.parts.get(*child).is_ok_and(|part| {
                    part.binding.0.scene == expected.scene
                        && part.binding.0.root == expected.root
                        && part.binding.0.source == expected.identity.source
                        && part.binding.0.level == Level::Coarse
                        && part.binding.0.bank == expected.identity.bank
                        && part.binding.0.role == expected.asset.role()
                        && part.binding.0.records == expected.asset.records()
                })
            })
            .collect::<Vec<_>>();
        ensure!(
            matching.len() == 1,
            "vista source partition missing/duplicated"
        );
        let child = matching[0];
        let part = self.parts.get(child)?;
        let material = match (part.wood, part.leaves) {
            (Some(wood), None) => wood.0.id().untyped(),
            (None, Some(leaves)) => leaves.0.id().untyped(),
            _ => anyhow::bail!("ambiguous vista material"),
        };
        let expected_material = match expected.asset.material() {
            GrownMaterialHandle::Wood(value) => value.id().untyped(),
            GrownMaterialHandle::Leaves(value) => value.id().untyped(),
        };
        ensure!(
            part.mesh.0.id() == expected.asset.mesh().id()
                && material == expected_material
                && *part.local == Transform::IDENTITY
                && part.global.to_matrix() == expected.frame
                && part.parent.parent() == expected.root
                && part.children.is_none_or(|value| value.is_empty())
                && *part.visibility == Visibility::Inherited
                && part.no_shadow
                && part.source_bounds
                && part.singleton == demand.is_some_and(|value| value.allows_singleton(child))
                && part.bounds.center == expected.asset.bounds().center
                && part.bounds.half_extents == expected.asset.bounds().half_extents,
            "vista part source/frame/hierarchy/material/bounds/policy changed"
        );
        Ok(CheckedPart {
            member: Member {
                entity: child,
                root: expected.root,
                part: expected.identity,
                mesh: expected.asset.mesh().id(),
                material,
                transform: expected.frame,
            },
            range: part.range.clone(),
        })
    }
}
