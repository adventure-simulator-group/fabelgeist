//! Whole owned specimen inventory, separate from the camera-visible draw subset.
use super::*;
use crate::presentation::{
    InstalledTreeMaterial, InstalledTreePartRole, InstalledTreeSpecimen, InstalledTreeVariant,
    PresentedTree, TacticalTreeAggregateBarkMaterial, TacticalTreeBarkMaterial,
    TacticalTreeImpostorMaterial, TacticalTreeLeafCardMaterial, TreeAssetOrigin,
    TreeCacheInspection, TreeLodRenderOverride,
    interior_lighting::{InteriorMaterial, InteriorMaterialSource},
};
use bevy::{
    asset::UntypedAssetId, camera::visibility::VisibleEntities, ecs::system::SystemParam,
    render::batching::NoAutomaticBatching,
};
use std::sync::Mutex;

mod assets;
mod roles;
mod roots;

#[derive(Clone, Copy)]
struct Binding {
    part: InstalledTreePartRole,
    mesh: AssetId<Mesh>,
    material: UntypedAssetId,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Member {
    pub entity: Entity,
    pub root: Entity,
    pub part: InstalledTreePartRole,
    pub mesh: AssetId<Mesh>,
    pub material: UntypedAssetId,
    authored_material: UntypedAssetId,
    transform: Mat4,
    visibility: Visibility,
}

#[derive(Resource)]
pub(super) struct OwnedTrees {
    roots: Vec<OwnedRoot>,
    variant: Arc<InstalledTreeVariant>,
    bindings: Vec<Binding>,
    sealed: Mutex<Option<Seal>>,
}

struct OwnedRoot {
    cache: InstalledTreeSpecimen,
    transform: Transform,
}

struct Seal {
    members: Vec<Member>,
    assets: assets::Snapshot,
}

#[derive(Resource, Default)]
pub(super) struct CurrentInventory {
    pub members: Vec<Member>,
    pub visible: Vec<Entity>,
    pub valid: bool,
    pub background: Vec<background::Member>,
}

#[derive(bevy::ecs::query::QueryData)]
struct SpecimenPart {
    entity: Entity,
    mesh: &'static Mesh3d,
    local: &'static Transform,
    global: &'static GlobalTransform,
    visibility: &'static Visibility,
    in_view: &'static ViewVisibility,
    bark: Option<&'static MeshMaterial3d<TacticalTreeBarkMaterial>>,
    leaf: Option<&'static MeshMaterial3d<TacticalTreeLeafCardMaterial>>,
    aggregate: Option<&'static MeshMaterial3d<TacticalTreeAggregateBarkMaterial>>,
    card: Option<&'static MeshMaterial3d<TacticalTreeImpostorMaterial>>,
    standard: Option<&'static MeshMaterial3d<StandardMaterial>>,
    interior: Option<&'static MeshMaterial3d<InteriorMaterial>>,
    original: Option<&'static InteriorMaterialSource>,
    parent: &'static ChildOf,
    descendants: Option<&'static Children>,
    unbatchable: Has<NoAutomaticBatching>,
}

#[derive(SystemParam)]
#[expect(
    clippy::type_complexity,
    reason = "the capture joins exact root and child material components without broad mutable world access"
)]
pub(in super::super) struct Inspector<'w, 's> {
    owned: Option<Res<'w, OwnedTrees>>,
    presented_roots: Query<'w, 's, Entity, With<PresentedTree>>,
    obstacles: Query<'w, 's, &'static adventuresim_tactical_core::prelude::SceneObstacle>,
    selection: Res<'w, TreeLodRenderOverride>,
    review: Res<'w, Review>,
    draws: Res<'w, material_draws::SpecimenDraws>,
    background: background::Inspector<'w, 's>,
    depth: Option<Res<'w, cameras::PerspectiveDepth>>,
    cameras: Query<
        'w,
        's,
        (
            &'static Camera,
            &'static GlobalTransform,
            &'static Projection,
            &'static VisibleEntities,
        ),
        With<crate::presentation::TacticalGameplayCamera>,
    >,
    roots: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static GlobalTransform,
            &'static Visibility,
            Option<&'static Children>,
            Has<PresentedTree>,
        ),
    >,
    parts: Query<'w, 's, SpecimenPart>,
    assets: assets::Stores<'w>,
    cache: TreeCacheInspection<'w, 's>,
    roles: roles::RoleInspector<'w, 's>,
}

impl OwnedTrees {
    pub(super) fn from_cache(caches: Vec<InstalledTreeSpecimen>, world: &World) -> Result<Self> {
        let review = world.resource::<Review>();
        let representation = review.representation;
        ensure!(
            caches.len() == review.layout.root_count(),
            "owned fixture root count drift"
        );
        let variant = Arc::clone(caches.first().context("empty owned tree group")?.variant());
        let bindings: Vec<_> = variant
            .parts()
            .iter()
            .filter(|binding| representation.owns(binding.role))
            .map(|binding| Binding {
                part: binding.role,
                mesh: binding.mesh,
                material: match binding.material {
                    InstalledTreeMaterial::Bark(id) => id.untyped(),
                    InstalledTreeMaterial::Leaves(id) => id.untyped(),
                    InstalledTreeMaterial::Buds(id) => id.untyped(),
                    InstalledTreeMaterial::Aggregate(id) => id.untyped(),
                    InstalledTreeMaterial::Card { material, .. } => material.untyped(),
                },
            })
            .collect();
        ensure!(!bindings.is_empty(), "empty owned part bindings");
        for (index, binding) in bindings.iter().enumerate() {
            ensure!(
                bindings[..index]
                    .iter()
                    .all(|earlier| earlier.mesh != binding.mesh && earlier.part != binding.part),
                "ambiguous owned mesh/part mapping"
            );
        }
        let mut roots = Vec::with_capacity(caches.len());
        for cache in caches {
            ensure!(
                Arc::ptr_eq(&variant, cache.variant()),
                "roots do not share one installed variant"
            );
            ensure!(
                roots
                    .iter()
                    .all(|root: &OwnedRoot| root.cache.root() != cache.root()),
                "duplicate owned root entity"
            );
            roots.push(OwnedRoot {
                transform: *world
                    .get::<Transform>(cache.root())
                    .context("spawned root transform")?,
                cache,
            });
        }
        Ok(Self {
            roots,
            variant,
            bindings,
            sealed: Mutex::new(None),
        })
    }

    pub(super) fn retained_report(&self) -> serde_json::Value {
        serde_json::json!({"origin": format!("{:?}", self.variant.origin()),
            "cache_key": self.variant.cache_key().to_u64(),
            "parts": self.variant.parts().iter().map(|part| serde_json::json!({
                "role": part.role, "mesh": format!("{:?}", part.mesh),
                "material": format!("{:?}", part.material),
            })).collect::<Vec<_>>(),
            "roots": self.roots.iter().map(|root| serde_json::json!({
                "entity_bits": root.cache.root().to_bits(), "transform": root.transform,
            })).collect::<Vec<_>>(),
            "scope": "one retained variant; distinct roots share corresponding part identities; no automatic GPU batching claim"})
    }

    pub(super) fn retain_cpu_data(&self, world: &mut World) -> Result<()> {
        assets::retain_cpu_data(
            world,
            self.bindings
                .iter()
                .map(|binding| (binding.mesh, binding.material)),
        )
    }

    pub(super) fn instrument_children(world: &mut World) -> Result<()> {
        let owned = world.resource::<Self>();
        if owned
            .sealed
            .lock()
            .map_err(|_| anyhow::anyhow!("inventory seal lock"))?
            .is_some()
        {
            return Ok(());
        }
        let mut children = Vec::new();
        for root in &owned.roots {
            children.extend(
                world
                    .get::<Children>(root.cache.root())
                    .context("instrumented tree children")?
                    .iter(),
            );
        }
        for child in children {
            if world.get::<NoAutomaticBatching>(child).is_none() {
                world.entity_mut(child).insert(NoAutomaticBatching);
            }
        }
        Ok(())
    }
}

impl Inspector<'_, '_> {
    pub(in super::super) fn check_capture(
        &self,
        state: &super::super::capture_state::SceneCaptureState,
    ) -> Result<()> {
        let view = state
            .views
            .get(state.view)
            .context("held screenshot view")?;
        let CapturePose::Cache(expected) = view.pose else {
            anyhow::bail!("held cache screenshot pose");
        };
        let (camera, transform, projection, _) = self.cameras.single()?;
        ensure!(
            expected.matches(
                camera,
                transform,
                projection,
                self.depth.as_deref().context("capture perspective depth")?
            ),
            "screenshot callback camera drift"
        );
        let current = self.check()?;
        self.draws
            .check_background(state.view, &current.background)?;
        Ok(())
    }

    /// Used both after visibility and inside the actual screenshot callback.
    pub(super) fn check(&self) -> Result<CurrentInventory> {
        let owned = self.owned.as_ref().context("owned review specimen")?;
        let mut actual_roots: Vec<_> = self.presented_roots.iter().collect();
        let mut expected_roots: Vec<_> = owned.roots.iter().map(|root| root.cache.root()).collect();
        actual_roots.sort_unstable();
        expected_roots.sort_unstable();
        ensure!(
            actual_roots == expected_roots,
            "unexpected selected/cache root set"
        );
        ensure!(
            matches!(
                (self.review.role, owned.variant.origin()),
                (Role::Reference, TreeAssetOrigin::Generated)
                    | (Role::Candidate, TreeAssetOrigin::Prepared(_))
            ),
            "review role/cache origin mismatch"
        );
        ensure!(
            self.obstacles.is_empty(),
            "unexpected retained scene obstacle/collider owner"
        );
        ensure!(
            self.review.representation.matches(&self.selection),
            "actual fixed production representation drift"
        );
        let mut members = Vec::new();
        let mut visible = Vec::new();
        for root in &owned.roots {
            let current = self.check_root(owned, root)?;
            members.extend(current.members);
            visible.extend(current.visible);
        }
        members.sort_by_key(|member| member.entity.to_bits());
        visible.sort_by_key(|entity| entity.to_bits());
        let mut seal = owned
            .sealed
            .lock()
            .map_err(|_| anyhow::anyhow!("owned inventory seal lock"))?;
        if let Some(seal) = &*seal {
            ensure!(
                seal.members == members,
                "sealed whole inventory/binding drift"
            );
            seal.assets.check(&self.assets)?;
        } else {
            *seal = Some(Seal {
                assets: assets::Snapshot::capture(&self.assets, &members)?,
                members: members.clone(),
            });
        }
        let (_, _, _, visible_background) = self.cameras.single()?;
        let background = self.background.collect(visible_background)?;
        Ok(CurrentInventory {
            members,
            visible,
            valid: true,
            background,
        })
    }
}

impl CurrentInventory {
    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"whole_inventory_verified": self.valid,
            "members": self.members.iter().map(|member| serde_json::json!({
                "entity_bits": member.entity.to_bits(), "root_bits": member.root.to_bits(), "part": member.part,
                "mesh": format!("{:?}", member.mesh),
                "authored_material": format!("{:?}", member.authored_material),
                "effective_material": format!("{:?}", member.material),
                "selected": member.visibility != Visibility::Hidden,
                "world_from_mesh": member.transform.to_cols_array(),
            })).collect::<Vec<_>>(),
            "limits": "CPU-retained selected meshes, referenced material images and interior field; no GPU uniform or texture-byte readback"})
    }
}
