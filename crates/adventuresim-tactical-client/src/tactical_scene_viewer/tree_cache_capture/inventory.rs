//! Whole owned specimen inventory, separate from the camera-visible draw subset.
use super::*;
use crate::presentation::{
    InstalledTreeMaterial, InstalledTreeOrigin, InstalledTreePartRole, InstalledTreeSpecimen,
    PresentedTree, TacticalTreeAggregateBarkMaterial, TacticalTreeBarkMaterial,
    TacticalTreeImpostorMaterial, TacticalTreeLeafCardMaterial, TreeCacheInspection,
    TreeLodRenderOverride,
    interior_lighting::{InteriorMaterial, InteriorMaterialSource},
};
use bevy::{
    asset::UntypedAssetId, camera::visibility::VisibleEntities, ecs::system::SystemParam,
    render::batching::NoAutomaticBatching,
};
use std::sync::Mutex;

mod assets;
mod roles;

#[derive(Clone, Copy)]
struct Binding {
    part: InstalledTreePartRole,
    mesh: AssetId<Mesh>,
    material: UntypedAssetId,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Member {
    pub entity: Entity,
    pub part: InstalledTreePartRole,
    pub mesh: AssetId<Mesh>,
    pub material: UntypedAssetId,
    authored_material: UntypedAssetId,
    transform: Mat4,
    visibility: Visibility,
}

#[derive(Resource)]
pub(super) struct OwnedSpecimen {
    pub root: Entity,
    bindings: Vec<Binding>,
    cache: InstalledTreeSpecimen,
    root_transform: Transform,
    sealed: Mutex<Option<Seal>>,
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
    owned: Option<Res<'w, OwnedSpecimen>>,
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

impl OwnedSpecimen {
    pub(super) fn from_cache(cache: InstalledTreeSpecimen, world: &World) -> Result<Self> {
        let representation = world.resource::<Review>().representation;
        let bindings = cache
            .parts
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
                    InstalledTreeMaterial::Card { material, .. } => material,
                },
            })
            .collect();
        Self::new(cache.root, bindings, cache, world)
    }

    fn new(
        root: Entity,
        bindings: Vec<Binding>,
        cache: InstalledTreeSpecimen,
        world: &World,
    ) -> Result<Self> {
        ensure!(!bindings.is_empty(), "empty owned specimen");
        for (index, binding) in bindings.iter().enumerate() {
            ensure!(
                bindings[..index]
                    .iter()
                    .all(|earlier| earlier.mesh != binding.mesh && earlier.part != binding.part),
                "ambiguous owned mesh/part mapping"
            );
        }
        Ok(Self {
            root,
            bindings,
            cache,
            root_transform: *world
                .get::<Transform>(root)
                .context("spawned specimen root transform")?,
            sealed: Mutex::new(None),
        })
    }

    pub(super) fn retained_report(&self) -> serde_json::Value {
        serde_json::json!({"origin": format!("{:?}", self.cache.origin),
            "cache_key": self.cache.cache_key.to_u64(),
            "parts": self.cache.parts.iter().map(|part| serde_json::json!({
                "role": part.role, "mesh": format!("{:?}", part.mesh),
                "material": format!("{:?}", part.material),
            })).collect::<Vec<_>>(),
            "scope": "complete retained part identities; selected child values have separate held snapshots"})
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
        let children: Vec<_> = world
            .get::<Children>(owned.root)
            .context("instrumented specimen children")?
            .iter()
            .collect();
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
        self.cache
            .verify(&owned.cache, self.review.representation)?;
        ensure!(
            self.presented_roots.single()? == owned.root,
            "unexpected selected/cache root count"
        );
        ensure!(
            matches!(
                (self.review.role, owned.cache.origin),
                (Role::Reference, InstalledTreeOrigin::Generated)
                    | (Role::Candidate, InstalledTreeOrigin::Prepared(_))
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
        let (local, global, visibility, children, presented) = self.roots.get(owned.root)?;
        ensure!(
            *local == owned.root_transform
                && global.to_matrix() == local.to_matrix()
                && *visibility == Visibility::Inherited
                && presented,
            "owned root transform/visibility/capability drift"
        );
        let children = children.context("owned specimen children")?;
        ensure!(
            children.len() == owned.bindings.len(),
            "incomplete or extra owned child inventory"
        );
        let mut members = Vec::with_capacity(children.len());
        let mut visible = Vec::new();
        for child in children.iter() {
            let SpecimenPartItem {
                entity,
                mesh,
                local,
                global,
                visibility,
                in_view,
                bark,
                leaf,
                aggregate,
                card,
                standard,
                interior,
                original,
                parent,
                descendants,
                unbatchable,
            } = self.parts.get(child)?;
            ensure!(
                unbatchable
                    && parent.parent() == owned.root
                    && descendants.is_none_or(|children| children.is_empty()),
                "unexpected nested or reparented specimen member"
            );
            let binding = owned
                .bindings
                .iter()
                .find(|binding| binding.mesh == mesh.0.id())
                .context("unadmitted owned mesh")?;
            let (authored, effective) =
                match (bark, leaf, aggregate, card, standard, interior, original) {
                    (Some(material), None, None, None, None, None, None) => {
                        (material.0.id().untyped(), material.0.id().untyped())
                    }
                    (None, Some(material), None, None, None, None, None) => {
                        (material.0.id().untyped(), material.0.id().untyped())
                    }
                    (None, None, Some(material), None, None, None, None) => {
                        (material.0.id().untyped(), material.0.id().untyped())
                    }
                    (None, None, None, Some(material), None, None, None) => {
                        (material.0.id().untyped(), material.0.id().untyped())
                    }
                    (None, None, None, None, None, Some(material), Some(original)) => {
                        (original.0.id().untyped(), material.0.id().untyped())
                    }
                    _ => anyhow::bail!("owned material absent, ambiguous or not converted"),
                };
            self.roles
                .check(entity, binding.part, &owned.cache, &self.cache)?;
            let expected_visibility = self.review.representation.visibility(binding.part);
            ensure!(
                authored == binding.material
                    && *local == Transform::IDENTITY
                    && global.to_matrix() == owned.root_transform.to_matrix()
                    && *visibility == expected_visibility,
                "owned binding/transform/selection drift"
            );
            if in_view.get() {
                visible.push(entity);
            }
            members.push(Member {
                entity,
                part: binding.part,
                mesh: mesh.0.id(),
                material: effective,
                authored_material: authored,
                transform: global.to_matrix(),
                visibility: *visibility,
            });
        }
        for binding in &owned.bindings {
            ensure!(
                members
                    .iter()
                    .filter(|member| member.mesh == binding.mesh)
                    .count()
                    == 1,
                "owned inventory is not a bijection with admitted parts"
            );
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
                "entity_bits": member.entity.to_bits(), "part": member.part,
                "mesh": format!("{:?}", member.mesh),
                "authored_material": format!("{:?}", member.authored_material),
                "effective_material": format!("{:?}", member.material),
                "selected": member.visibility != Visibility::Hidden,
                "world_from_mesh": member.transform.to_cols_array(),
            })).collect::<Vec<_>>(),
            "limits": "CPU-retained selected meshes, referenced material images and interior field; no GPU uniform or texture-byte readback"})
    }
}
