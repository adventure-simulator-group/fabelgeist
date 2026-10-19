//! Whole source-owned inventory, separate from camera-visible submission.

use super::*;
use crate::presentation::{PresentedTree, TacticalGameplayCamera};
use adventuresim_environment::{
    prepared::RecordSpan,
    render::{
        ForestPart,
        full::{GrownLeafMaterial, GrownMaterialHandle, GrownWoodMaterial},
    },
};
use bevy::{
    asset::UntypedAssetId,
    camera::{
        primitives::Aabb,
        visibility::{NoAutoAabb, VisibleEntities},
    },
    ecs::system::SystemParam,
    render::batching::NoAutomaticBatching,
};

pub(super) use assets::watch_mutations;

mod assets;

#[derive(Component)]
pub(super) struct GrownRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) struct PartOrdinal(usize);

#[derive(Resource)]
pub(super) struct OwnedGrown {
    root: Entity,
    frame: Transform,
    assets: GrownFullAssets,
    parts: Vec<BoundPart>,
    held: assets::Snapshot,
}

struct BoundPart {
    entity: Entity,
    ordinal: PartOrdinal,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(super) struct PartIdentity {
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
    transform: Mat4,
}

#[derive(Resource, Default)]
pub(super) struct CurrentInventory {
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
    reason = "native witness joins exact root, part and effective terrain state"
)]
pub(in super::super) struct Inspector<'w, 's> {
    owned: Option<Res<'w, OwnedGrown>>,
    review: Option<Res<'w, Review>>,
    depth: Option<Res<'w, cameras::PerspectiveDepth>>,
    draws: Option<Res<'w, material_draws::SpecimenDraws>>,
    roots: Query<
        'w,
        's,
        (
            Entity,
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
    background: background::Inspector<'w, 's>,
    assets: assets::Stores<'w>,
}

struct ExpectedSpan {
    role: ForestPart,
    records: RecordSpan,
}

impl OwnedGrown {
    pub(super) fn install(
        geometry: GrownFullGeometry,
        encoding: GrownVertexEncoding,
        frame: Transform,
        world: &mut World,
    ) -> Result<Self> {
        let assets = GrownFullAssets::install(geometry, encoding, world)?;
        let root = world
            .spawn((
                GrownRoot,
                frame,
                GlobalTransform::default(),
                Visibility::Visible,
            ))
            .id();
        let mut parts = Vec::new();
        for (ordinal, asset) in assets.parts().enumerate() {
            let ordinal = PartOrdinal(ordinal);
            let mut child = world.spawn((
                ordinal,
                Mesh3d(asset.mesh().clone()),
                asset.bounds(),
                NoAutoAabb,
                Transform::IDENTITY,
                GlobalTransform::default(),
                Visibility::Inherited,
                NoAutomaticBatching,
                ChildOf(root),
            ));
            match asset.material() {
                GrownMaterialHandle::Wood(handle) => {
                    child.insert(MeshMaterial3d(handle.clone()));
                }
                GrownMaterialHandle::Leaves(handle) => {
                    child.insert(MeshMaterial3d(handle.clone()));
                }
            }
            parts.push(BoundPart {
                entity: child.id(),
                ordinal,
            });
        }
        let held = assets::Snapshot::capture(world, &assets)?;
        let owned = Self {
            root,
            frame,
            assets,
            parts,
            held,
        };
        owned.check_source()?;
        Ok(owned)
    }

    fn check_source(&self) -> Result<()> {
        let model = self
            .assets
            .bank()
            .library()
            .models
            .get(self.assets.variant().index())
            .context("held grown source")?;
        ensure!(
            model.source == self.assets.source(),
            "held grown source identity drift"
        );
        ensure!(
            self.parts.len() == self.assets.parts().len(),
            "held grown part count drift"
        );
        for expected in [
            ExpectedSpan {
                role: ForestPart::Branches,
                records: model.branches,
            },
            ExpectedSpan {
                role: ForestPart::Leaves,
                records: model.leaves,
            },
        ] {
            let source = expected.records;
            let mut start = source.start;
            for asset in self
                .assets
                .parts()
                .filter(|asset| asset.role() == expected.role)
            {
                let records = asset.records();
                ensure!(
                    records.start == start && records.count > 0,
                    "grown partition gap/overlap"
                );
                start = start
                    .checked_add(records.count)
                    .context("grown partition coverage overflow")?;
            }
            ensure!(
                start
                    == source
                        .start
                        .checked_add(source.count)
                        .context("grown source span overflow")?,
                "grown source incomplete"
            );
        }
        Ok(())
    }

    pub(super) fn owns_buffer(&self, id: AssetId<bevy::render::storage::ShaderBuffer>) -> bool {
        self.assets
            .buffers()
            .any(|buffer| buffer.handle().id() == id)
    }
    pub(super) fn owns_mesh(&self, id: AssetId<Mesh>) -> bool {
        self.assets.parts().any(|part| part.mesh().id() == id)
    }
    pub(super) fn owns_material(&self, id: UntypedAssetId) -> bool {
        self.assets
            .parts()
            .any(|part| material_id(part.material()) == id)
    }
}

impl Inspector<'_, '_> {
    pub(super) fn check(&self) -> Result<CurrentInventory> {
        let owned = self.owned.as_ref().context("grown ownership missing")?;
        let review = self.review.as_ref().context("grown review missing")?;
        owned.check_source()?;
        ensure!(
            owned.assets.encoding() == review.role.encoding()
                && owned.assets.source() == review.source,
            "grown source/encoding drift"
        );
        ensure!(
            self.presented.is_empty() && self.obstacles.is_empty(),
            "old tactical tree/obstacle retained in isolated grown witness"
        );
        let (root, local, global, visibility, children, parent) = self.roots.single()?;
        ensure!(
            root == owned.root
                && *local == owned.frame
                && global.to_matrix() == owned.frame.to_matrix()
                && *visibility == Visibility::Visible
                && parent.is_none(),
            "grown root frame/ownership drift"
        );
        ensure!(
            children.len() == owned.parts.len() && self.parts.iter().count() == owned.parts.len(),
            "grown children/inventory count drift"
        );
        let (_, _, _, camera_visible) = self.cameras.single()?;
        let camera_visible = camera_visible.get(std::any::TypeId::of::<Mesh3d>());
        let mut members = Vec::new();
        let mut visible = Vec::new();
        for (binding, asset) in owned.parts.iter().zip(owned.assets.parts()) {
            let part = self.parts.get(binding.entity)?;
            ensure!(
                children.contains(&binding.entity)
                    && part.parent.parent() == owned.root
                    && part.descendants.is_none_or(|children| children.is_empty()),
                "grown hierarchy drift"
            );
            ensure!(
                *part.ordinal == binding.ordinal
                    && part.mesh.0 == *asset.mesh()
                    && *part.local == Transform::IDENTITY
                    && part.global.to_matrix() == owned.frame.to_matrix(),
                "grown part/frame binding drift"
            );
            let expected_bounds = asset.bounds();
            ensure!(
                part.owned_bounds
                    && part.bounds.center == expected_bounds.center
                    && part.bounds.half_extents == expected_bounds.half_extents,
                "grown culling bounds drift"
            );
            ensure!(
                *part.visibility == Visibility::Inherited && part.unbatchable && !part.no_shadow,
                "grown visibility/shadow/draw policy drift"
            );
            let material = match (asset.material(), part.wood, part.leaves) {
                (GrownMaterialHandle::Wood(expected), Some(actual), None)
                    if actual.0 == *expected =>
                {
                    expected.id().untyped()
                }
                (GrownMaterialHandle::Leaves(expected), None, Some(actual))
                    if actual.0 == *expected =>
                {
                    expected.id().untyped()
                }
                _ => anyhow::bail!("grown material component drift"),
            };
            members.push(Member {
                entity: part.entity,
                root,
                part: PartIdentity {
                    ordinal: binding.ordinal,
                    role: asset.role(),
                    records: asset.records(),
                },
                mesh: asset.mesh().id(),
                material,
                transform: part.global.to_matrix(),
            });
            if camera_visible.contains(&part.entity) {
                visible.push(part.entity);
            }
        }
        owned.held.check(&self.assets, &owned.assets)?;
        members.sort_by_key(|member| member.entity.to_bits());
        visible.sort_by_key(|entity| entity.to_bits());
        let background = self.background.collect(self.cameras.single()?.3)?;
        Ok(CurrentInventory {
            members,
            visible,
            valid: true,
            background,
        })
    }

    pub(in super::super) fn check_capture(
        &self,
        state: &super::super::capture_state::SceneCaptureState,
    ) -> Result<()> {
        let inventory = self.check()?;
        let expected = match state
            .views
            .get(state.view)
            .context("grown callback view")?
            .pose
        {
            CapturePose::GrownFull(camera) => camera,
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
        let draws = self
            .draws
            .as_ref()
            .context("grown submitted draw observer missing")?;
        draws.check_inventory(state.view, &inventory.members, &inventory.visible)?;
        draws.check_background(state.view, &inventory.background)?;
        Ok(())
    }
}

impl PartIdentity {
    pub(super) fn is_leaf(self) -> bool {
        self.role == ForestPart::Leaves
    }
}

impl CurrentInventory {
    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"whole_inventory_verified":self.valid,"owned_grown_roots":1,"members":self.members.iter().map(|member|serde_json::json!({"entity_bits":member.entity.to_bits(),"root_bits":member.root.to_bits(),"part":member.part,"mesh":format!("{:?}",member.mesh),"material":format!("{:?}",member.material),"world_from_mesh":member.transform.to_cols_array()})).collect::<Vec<_>>(),"draw_policy":"instrumented_singletons","limits":"retained CPU content and immutable source asset lifecycle; no GPU storage-byte readback or performance claim"})
    }
}

fn material_id(material: &GrownMaterialHandle) -> UntypedAssetId {
    match material {
        GrownMaterialHandle::Wood(handle) => handle.id().untyped(),
        GrownMaterialHandle::Leaves(handle) => handle.id().untyped(),
    }
}
