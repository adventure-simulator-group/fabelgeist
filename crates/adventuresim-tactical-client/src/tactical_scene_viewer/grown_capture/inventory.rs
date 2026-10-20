//! Exact source part/root bindings, independent of native GPU-visible membership.

use super::*;
use crate::presentation::{PresentedTree, TacticalGameplayCamera};
use adventuresim_environment::{
    prepared::RecordSpan,
    render::{
        ForestPart,
        grown::{GrownLeafMaterial, GrownMaterialHandle, GrownPartAsset, GrownWoodMaterial},
    },
};
use bevy::{
    asset::UntypedAssetId,
    camera::{
        primitives::Aabb,
        visibility::{NoAutoAabb, VisibleEntities},
    },
    ecs::system::SystemParam,
    mesh::MeshTag,
    render::batching::NoAutomaticBatching,
};
use owned::{BoundPart, RootBinding};

mod owned;

#[derive(Component)]
pub(super) struct GrownRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) struct PartOrdinal(usize);

#[derive(Resource)]
pub(super) struct OwnedGrown {
    assets: GrownAssets,
    roots: Vec<RootBinding>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) struct PartIdentity {
    level: Level,
    ordinal: PartOrdinal,
    role: ForestPart,
    records: RecordSpan,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Member {
    pub entity: Entity,
    pub root: Entity,
    pub part: PartIdentity,
    pub mesh: AssetId<Mesh>,
    pub material: UntypedAssetId,
    pub transform: Mat4,
}

#[derive(Clone, Copy)]
struct RootMember {
    entity: Entity,
    ordinal: layout::RootOrdinal,
    frame: Mat4,
}

#[derive(Resource, Default)]
pub(super) struct CurrentInventory {
    roots: Vec<RootMember>,
    pub members: Vec<Member>,
    pub visible: Vec<Entity>,
    pub valid: bool,
    pub background: Vec<background::Member>,
}

#[derive(bevy::ecs::query::QueryData)]
struct Part {
    entity: Entity,
    ordinal: &'static PartOrdinal,
    mesh: &'static Mesh3d,
    local: &'static Transform,
    global: &'static GlobalTransform,
    visibility: &'static Visibility,
    bounds: &'static Aabb,
    owned_bounds: Has<NoAutoAabb>,
    parent: &'static ChildOf,
    descendants: Option<&'static Children>,
    wood: Option<&'static MeshMaterial3d<GrownWoodMaterial>>,
    leaves: Option<&'static MeshMaterial3d<GrownLeafMaterial>>,
    unbatchable: Has<NoAutomaticBatching>,
    no_shadow: Has<bevy::light::NotShadowCaster>,
}

#[derive(SystemParam)]
#[expect(
    clippy::type_complexity,
    reason = "joins held root/source members to native scene and camera state"
)]
pub(in super::super) struct Inspector<'w, 's> {
    owned: Option<Res<'w, OwnedGrown>>,
    review: Option<Res<'w, Review>>,
    depth: Option<Res<'w, cameras::PerspectiveDepth>>,
    batches: Option<Res<'w, ordinary_batches::OrdinaryBatches>>,
    root_visibility: Option<Res<'w, controls::RootVisibility>>,
    roots: Query<
        'w,
        's,
        (
            Entity,
            &'static layout::RootOrdinal,
            &'static Transform,
            &'static GlobalTransform,
            &'static Visibility,
            &'static Children,
            Option<&'static ChildOf>,
        ),
        With<GrownRoot>,
    >,
    parts: Query<'w, 's, Part>,
    presented: Query<'w, 's, Entity, With<PresentedTree>>,
    obstacles: Query<'w, 's, Entity, With<adventuresim_tactical_core::prelude::SceneObstacle>>,
    cameras: Query<
        'w,
        's,
        (
            &'static Camera,
            &'static GlobalTransform,
            &'static Projection,
            &'static VisibleEntities,
        ),
        With<TacticalGameplayCamera>,
    >,
    tags: Query<'w, 's, &'static MeshTag>,
    background: background::Inspector<'w, 's>,
    support: cameras::SupportInspector<'w, 's>,
}

impl Inspector<'_, '_> {
    pub(super) fn check(&self, scene_digest: &str) -> Result<CurrentInventory> {
        self.support.check(scene_digest)?;
        let owned = self.owned.as_ref().context("grown ownership missing")?;
        let review = self.review.as_ref().context("grown review missing")?;
        owned.check_source()?;
        ensure!(
            owned.assets.encoding() == review.role.encoding()
                && owned.assets.source() == review.source,
            "grown source/encoding drift"
        );
        let held_level = owned.assets.level();
        ensure!(
            held_level == review.level,
            "grown level drift: held {held_level:?}, requested {:?}",
            review.level
        );
        ensure!(
            self.presented.is_empty() && self.obstacles.is_empty(),
            "old tactical tree/obstacle retained in isolated grown witness"
        );
        ensure!(
            self.roots.iter().count() == owned.roots.len()
                && self.parts.iter().count() == owned.assets.parts().len() * owned.roots.len(),
            "grown root/child inventory drift"
        );
        let visible_entities = self.cameras.single()?.3;
        let camera_visible = visible_entities.get(std::any::TypeId::of::<Mesh3d>());
        let mut inventory = CurrentInventory::default();
        for root in &owned.roots {
            inventory.roots.push(self.inspect_root(root)?);
            for (binding, asset) in root.parts.iter().zip(owned.assets.parts()) {
                let member = self.inspect_part(root, binding, asset, review)?;
                if camera_visible.contains(&member.entity) {
                    inventory.visible.push(member.entity);
                }
                inventory.members.push(member);
            }
        }
        inventory
            .members
            .sort_by_key(|member| member.entity.to_bits());
        inventory.visible.sort_by_key(|entity| entity.to_bits());
        inventory.background = self.background.collect(visible_entities)?;
        inventory.valid = true;
        Ok(inventory)
    }

    fn inspect_root(&self, binding: &RootBinding) -> Result<RootMember> {
        let (entity, ordinal, local, global, visibility, children, parent) =
            self.roots.get(binding.entity)?;
        let expected_visibility = self
            .root_visibility
            .as_ref()
            .context("grown root visibility policy")?
            .expected(binding.placement.ordinal);
        ensure!(
            entity == binding.entity
                && *ordinal == binding.placement.ordinal
                && *local == binding.placement.frame
                && global.to_matrix() == binding.placement.frame.to_matrix()
                && *visibility == expected_visibility
                && parent.is_none(),
            "grown root frame/ownership drift"
        );
        ensure!(
            children.len() == binding.parts.len()
                && binding
                    .parts
                    .iter()
                    .all(|part| children.contains(&part.entity)),
            "grown child hierarchy drift"
        );
        Ok(RootMember {
            entity,
            ordinal: *ordinal,
            frame: global.to_matrix(),
        })
    }

    fn inspect_part(
        &self,
        root: &RootBinding,
        binding: &BoundPart,
        asset: &GrownPartAsset,
        review: &Review,
    ) -> Result<Member> {
        let part = self.parts.get(binding.entity)?;
        ensure!(
            part.parent.parent() == root.entity
                && part.descendants.is_none_or(|children| children.is_empty()),
            "grown part hierarchy drift"
        );
        ensure!(
            *part.ordinal == binding.ordinal
                && part.mesh.0 == *asset.mesh()
                && *part.local == Transform::IDENTITY
                && part.global.to_matrix() == root.placement.frame.to_matrix(),
            "grown part/frame binding drift"
        );
        let bounds = asset.bounds();
        ensure!(
            part.owned_bounds
                && part.bounds.center == bounds.center
                && part.bounds.half_extents == bounds.half_extents,
            "grown culling bounds drift"
        );
        ensure!(
            *part.visibility == Visibility::Inherited
                && part.unbatchable == (review.control == controls::BatchControl::ForcedSingletons)
                && !part.no_shadow,
            "grown visibility/shadow/draw policy drift"
        );
        let material = match (asset.material(), part.wood, part.leaves) {
            (GrownMaterialHandle::Wood(expected), Some(actual), None) if actual.0 == *expected => {
                expected.id().untyped()
            }
            (GrownMaterialHandle::Leaves(expected), None, Some(actual))
                if actual.0 == *expected =>
            {
                expected.id().untyped()
            }
            _ => anyhow::bail!("grown material component drift"),
        };
        Ok(Member {
            entity: part.entity,
            root: root.entity,
            part: PartIdentity {
                level: review.level,
                ordinal: binding.ordinal,
                role: asset.role(),
                records: asset.records(),
            },
            mesh: asset.mesh().id(),
            material,
            transform: part.global.to_matrix(),
        })
    }

    pub(in super::super) fn check_capture(
        &self,
        state: &super::super::capture_state::SceneCaptureState,
    ) -> Result<()> {
        let inventory = self.check(&state.digest)?;
        let expected = match state
            .views
            .get(state.view)
            .context("grown callback view")?
            .pose
        {
            CapturePose::Grown(camera) => camera,
            _ => anyhow::bail!("grown callback pose missing"),
        };
        let (camera, transform, projection, _) = self.cameras.single()?;
        ensure!(
            expected.matches(
                camera,
                transform,
                projection,
                self.depth.as_deref().context("grown projection depth")?
            ),
            "grown callback camera drift"
        );
        self.batches
            .as_ref()
            .context("grown GPU observer missing")?
            .check_capture(state.view, &inventory, &self.tags)?;
        Ok(())
    }
}

impl CurrentInventory {
    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"whole_inventory_verified":self.valid,"owned_grown_roots":self.roots.len(),"roots":self.roots.iter().map(|root|serde_json::json!({"entity_bits":root.entity.to_bits(),"ordinal":root.ordinal,"world_from_root":root.frame.to_cols_array()})).collect::<Vec<_>>(),"members":self.members.iter().map(|member|serde_json::json!({"entity_bits":member.entity.to_bits(),"root_bits":member.root.to_bits(),"part":member.part,"mesh":format!("{:?}",member.mesh),"material":format!("{:?}",member.material),"world_from_mesh":member.transform.to_cols_array()})).collect::<Vec<_>>(),"draw_policy":"ordinary_gpu_membership","limits":"held source/asset identities and actual GPU instance membership; no CPU byte seals, shadow/prepass membership or performance claim"})
    }
}
