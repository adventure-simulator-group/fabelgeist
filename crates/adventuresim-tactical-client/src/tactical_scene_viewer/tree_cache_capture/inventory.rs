//! Whole owned specimen inventory, separate from the camera-visible draw subset.
use super::*;
use crate::presentation::{
    InstalledCacheSpecimen, InstalledTreeMaterial, InstalledTreePartRole, PreparedCacheInspection,
    PresentedTree, TacticalTreeBarkMaterial, TacticalTreeLeafCardMaterial, TreeLeafRepresentation,
    TreeLodRenderOverride, TreeSourcePart, TreeSourcePresentation,
    interior_lighting::{InteriorMaterial, InteriorMaterialSource},
};
use bevy::{
    asset::UntypedAssetId, ecs::system::SystemParam, render::batching::NoAutomaticBatching,
};
use std::sync::Mutex;

mod assets;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Part {
    Source(TreeSourcePart),
    Cache(InstalledTreePartRole),
}

#[derive(Clone, Copy)]
struct Binding {
    part: Part,
    mesh: AssetId<Mesh>,
    material: UntypedAssetId,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Member {
    pub entity: Entity,
    pub part: Part,
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
    cache: Option<InstalledCacheSpecimen>,
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
    depth: Option<Res<'w, cameras::PerspectiveDepth>>,
    cameras: Query<
        'w,
        's,
        (
            &'static Camera,
            &'static GlobalTransform,
            &'static Projection,
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
    parts: Query<
        'w,
        's,
        (
            Entity,
            &'static Mesh3d,
            &'static Transform,
            &'static GlobalTransform,
            &'static Visibility,
            &'static ViewVisibility,
            Option<&'static MeshMaterial3d<TacticalTreeBarkMaterial>>,
            Option<&'static MeshMaterial3d<TacticalTreeLeafCardMaterial>>,
            Option<&'static MeshMaterial3d<StandardMaterial>>,
            Option<&'static MeshMaterial3d<InteriorMaterial>>,
            Option<&'static InteriorMaterialSource>,
            Option<&'static TreeSourcePart>,
            &'static ChildOf,
            Option<&'static Children>,
            Has<NoAutomaticBatching>,
        ),
    >,
    assets: assets::Stores<'w>,
    cache: PreparedCacheInspection<'w, 's>,
}

impl OwnedSpecimen {
    pub(super) fn from_source(source: TreeSourcePresentation, world: &World) -> Result<Self> {
        let bindings = source
            .parts
            .into_iter()
            .map(|binding| Binding {
                part: Part::Source(binding.part),
                mesh: binding.mesh,
                material: binding.material,
            })
            .collect();
        Self::new(source.root, bindings, None, world)
    }

    pub(super) fn from_cache(cache: InstalledCacheSpecimen, world: &World) -> Result<Self> {
        let bindings = cache
            .parts
            .iter()
            .filter_map(|binding| {
                let material = match binding.material {
                    InstalledTreeMaterial::Bark(id) => id.untyped(),
                    InstalledTreeMaterial::Leaves(id) => id.untyped(),
                    InstalledTreeMaterial::Buds(id) => id.untyped(),
                    InstalledTreeMaterial::Card { .. } => return None,
                };
                Some(Binding {
                    part: Part::Cache(binding.role),
                    mesh: binding.mesh,
                    material,
                })
            })
            .collect();
        Self::new(cache.root, bindings, Some(cache), world)
    }

    fn new(
        root: Entity,
        bindings: Vec<Binding>,
        cache: Option<InstalledCacheSpecimen>,
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
        let (camera, transform, projection) = self.cameras.single()?;
        ensure!(
            expected.matches(
                camera,
                transform,
                projection,
                self.depth.as_deref().context("capture perspective depth")?
            ),
            "screenshot callback camera drift"
        );
        self.check()?;
        Ok(())
    }

    /// Used both after visibility and inside the actual screenshot callback.
    pub(super) fn check(&self) -> Result<CurrentInventory> {
        let owned = self.owned.as_ref().context("owned review specimen")?;
        if let Some(cache) = &owned.cache {
            self.cache.verify(cache)?;
            ensure!(
                self.presented_roots.single()? == owned.root,
                "unexpected selected/cache root count"
            );
        } else {
            ensure!(
                self.presented_roots.is_empty(),
                "unexpected production root in source capture"
            );
        }
        ensure!(
            self.obstacles.is_empty(),
            "unexpected retained scene obstacle/collider owner"
        );
        ensure!(
            self.selection.lod == Some(0)
                && self.selection.leaf == Some(TreeLeafRepresentation::TexturedMesh)
                && self.selection.projected_scale.is_none(),
            "actual fixed Full selection drift"
        );
        let (local, global, visibility, children, presented) = self.roots.get(owned.root)?;
        ensure!(
            *local == owned.root_transform
                && global.to_matrix() == local.to_matrix()
                && *visibility == Visibility::Inherited
                && presented == owned.cache.is_some(),
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
            let (
                entity,
                mesh,
                local,
                global,
                visibility,
                in_view,
                bark,
                leaf,
                standard,
                interior,
                original,
                source_part,
                parent,
                descendants,
                unbatchable,
            ) = self.parts.get(child)?;
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
            let (authored, effective) = match (bark, leaf, standard, interior, original) {
                (Some(material), None, None, None, None) => {
                    (material.0.id().untyped(), material.0.id().untyped())
                }
                (None, Some(material), None, None, None) => {
                    (material.0.id().untyped(), material.0.id().untyped())
                }
                (None, None, None, Some(material), Some(original)) => {
                    (original.0.id().untyped(), material.0.id().untyped())
                }
                _ => anyhow::bail!("owned material absent, ambiguous or not converted"),
            };
            let expected_visibility =
                if binding.part == Part::Cache(InstalledTreePartRole::MidTrunkDependency) {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                };
            ensure!(
                authored == binding.material
                    && *local == Transform::IDENTITY
                    && global.to_matrix() == owned.root_transform.to_matrix()
                    && *visibility == expected_visibility,
                "owned binding/transform/selection drift"
            );
            match binding.part {
                Part::Source(expected) => {
                    ensure!(source_part == Some(&expected), "source role drift")
                }
                Part::Cache(_) => ensure!(source_part.is_none(), "cache cannot impersonate source"),
            }
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
        Ok(CurrentInventory {
            members,
            visible,
            valid: true,
        })
    }
}

impl Part {
    pub(super) fn is_wood(self) -> bool {
        matches!(
            self,
            Self::Source(TreeSourcePart::Wood)
                | Self::Cache(InstalledTreePartRole::DetailedTrunk)
                | Self::Cache(InstalledTreePartRole::Wood { .. })
        )
    }
    pub(super) fn is_foliage(self) -> bool {
        matches!(
            self,
            Self::Source(TreeSourcePart::Foliage)
                | Self::Cache(InstalledTreePartRole::CamberedLeaves { .. })
        )
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
