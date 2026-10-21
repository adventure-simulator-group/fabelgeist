//! Immutable native census, joined to producer-owned attachments and assets.
use super::super::super::obstacle_setup::SpawnedObstacle;
use super::super::ordinary_batches::inventory::{Member, PartIdentity, PartOrdinal, SourceBinding};
use super::*;
use crate::presentation::{GrownPartBinding, GrownRootAttachment, GrownSceneAssets};
use adventuresim_environment::render::grown::GrownMaterialHandle;
use adventuresim_tactical_core::prelude::SceneObstacle;
use bevy::camera::primitives::Aabb;

#[derive(Resource)]
pub(super) struct HeldScene {
    pub assets: GrownSceneAssets,
    pub obstacles: Vec<SpawnedObstacle>,
    pub roots: Vec<Root>,
    pub buffers: Vec<SourceBinding>,
    pub vista: vista::HeldVista,
}

pub(super) struct Root {
    pub attachment: GrownRootAttachment,
    pub groups: Vec<Group>,
}

pub(super) struct Group {
    pub entity: Entity,
    pub level: Level,
    pub parts: Vec<Part>,
}

pub(super) struct Part {
    pub parent: Entity,
    pub member: Member,
    pub binding: GrownPartBinding,
    pub bounds: Aabb,
}

impl HeldScene {
    pub(super) fn admit(world: &mut World, assets: GrownSceneAssets) -> Result<Self> {
        let digest = world.resource::<SceneCaptureState>().digest.clone();
        CapturedObstacleBatch::check(world, &digest)?;
        let obstacles = world
            .resource::<CapturedObstacleBatch>()
            .members()
            .copied()
            .collect::<Vec<_>>();
        let attachments = {
            let mut state = SystemState::<TreeCacheInspection>::new(world);
            let inspection = state.get(world)?;
            obstacles
                .iter()
                .filter(|root| root.kind == SceneObstacle::Tree)
                .map(|root| inspection.grown_root(root.entity))
                .collect::<Result<Vec<_>>>()?
        };
        let roots = attachments
            .into_iter()
            .map(|attachment| Root::admit(world, &assets, attachment))
            .collect::<Result<Vec<_>>>()?;
        let Subject::Generated {
            bank,
            sources,
            selection,
            trees,
        } = *world.resource::<Subject>()
        else {
            anyhow::bail!("generated census requires its explicit subject");
        };
        ensure!(
            roots.len() == trees
                && assets.selection() == selection
                && assets.catalogue().bank_assets().bank().id() == bank,
            "generated subject differs from actual scene census"
        );
        ensure!(
            sources
                .iter()
                .all(|source| roots.iter().any(|root| root.attachment.source == *source)),
            "both genuine sources must be used by original roots"
        );
        let mut buffers = None;
        for source in assets.catalogue().sources() {
            for level in [Level::Full, Level::Middle, Level::Coarse] {
                let closure = assets.catalogue().assets(*source, level)?;
                ensure!(
                    Arc::ptr_eq(closure.bank_assets(), assets.catalogue().bank_assets()),
                    "catalogue closures do not share their real bank owner"
                );
                let actual = SourceBinding::from_assets(closure);
                let roles = adventuresim_environment::render::grown::GrownBufferRole::ALL;
                ensure!(
                    actual.len() == roles.len()
                        && roles.iter().all(|role| actual
                            .iter()
                            .filter(|binding| binding.role == *role)
                            .count()
                            == 1)
                        && actual
                            .iter()
                            .enumerate()
                            .all(|(index, binding)| actual[..index]
                                .iter()
                                .all(|earlier| earlier.handle.id() != binding.handle.id())),
                    "generated source roles and asset handles must be a bijection"
                );
                if let Some(expected) = &buffers {
                    ensure!(expected == &actual, "catalogue source handles differ");
                } else {
                    buffers = Some(actual);
                }
            }
        }
        let vista = vista::HeldVista::admit(world, &assets)?;
        Ok(Self {
            vista,
            assets,
            obstacles,
            roots,
            buffers: buffers.context("empty generated catalogue")?,
        })
    }

    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"scene_entity_bits":self.assets.scene().to_bits(),
            "selection":self.assets.selection(),"physical_obstacles":self.obstacles.len(),
            "physical_tree_roots":self.roots.len(),"vista":self.vista.report(),"selected_source_count":self.assets.catalogue().sources().len(),
            "owned_levels":[Level::Full,Level::Middle,Level::Coarse],
            "catalogue_logical_mesh_bytes":self.assets.catalogue().logical_mesh_bytes().bytes(),
            "source_root_counts":self.assets.catalogue().sources().iter().map(|source|
                serde_json::json!({"source":source,"roots":self.roots.iter().filter(|root|root.attachment.source==*source).count()})).collect::<Vec<_>>(),
            "parts":self.roots.iter().map(|root|root.parts().count()).sum::<usize>(),
            "installed_groups":self.roots.iter().map(|root|root.groups.len()).sum::<usize>(),
            "root_hierarchies":self.roots.iter().map(|root|serde_json::json!({"root_bits":root.attachment.root.to_bits(),"visual_bits":root.attachment.visual.to_bits(),"source":root.attachment.source,"groups":root.groups.iter().map(|group|serde_json::json!({"entity_bits":group.entity.to_bits(),"level":group.level,"parts":group.parts.iter().map(|part|serde_json::json!({"entity_bits":part.member.entity.to_bits(),"parent_bits":part.parent.to_bits(),"identity":part.member.part,"mesh":format!("{:?}",part.member.mesh),"material":format!("{:?}",part.member.material),"world_from_mesh":part.member.transform.to_cols_array(),"bounds":{"centre":part.bounds.center.to_array(),"half_extents":part.bounds.half_extents.to_array()}})).collect::<Vec<_>>()})).collect::<Vec<_>>()})).collect::<Vec<_>>(),
            "source_buffers":self.buffers.iter().map(|binding|serde_json::json!({"role":format!("{:?}",binding.role),"handle":format!("{:?}",binding.handle.id())})).collect::<Vec<_>>(),
            "limits":"immutable original physical identities and live source/asset joins; no collider-byte seal, GPU source-byte readback or physical-memory measurement"})
    }
}

impl Root {
    fn admit(
        world: &World,
        scene: &GrownSceneAssets,
        attachment: GrownRootAttachment,
    ) -> Result<Self> {
        ensure!(
            attachment.scene == scene.scene()
                && attachment.bank == scene.catalogue().bank_assets().bank().id(),
            "generated attachment scene/bank mismatch"
        );
        let owned: Vec<_> = attachment.groups().collect();
        let expected = match scene.selection() {
            adventuresim_environment::render::LevelSelection::Fixed(level) => vec![level],
            adventuresim_environment::render::LevelSelection::Adaptive => {
                vec![Level::Full, Level::Middle, Level::Coarse]
            }
        };
        ensure!(
            owned.iter().map(|group| group.level).eq(expected),
            "generated hierarchy differs from requested selection policy"
        );
        let groups = owned
            .into_iter()
            .map(|group| Group::admit(world, scene, &attachment, group.entity, group.level))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { attachment, groups })
    }

    pub(super) fn parts(&self) -> impl Iterator<Item = &Part> {
        self.groups.iter().flat_map(|group| &group.parts)
    }
}

impl Group {
    fn admit(
        world: &World,
        scene: &GrownSceneAssets,
        attachment: &GrownRootAttachment,
        entity: Entity,
        level: Level,
    ) -> Result<Self> {
        let assets = scene.catalogue().assets(attachment.source, level)?;
        let children = world
            .get::<Children>(entity)
            .context("generated visual children")?;
        ensure!(
            children.len() == assets.parts().len(),
            "generated source part count mismatch"
        );
        let parent = entity;
        let mut parts = Vec::new();
        for (ordinal, asset) in assets.parts().enumerate() {
            let binding = GrownPartBinding {
                scene: attachment.scene,
                root: attachment.root,
                bank: attachment.bank,
                source: attachment.source,
                level,
                role: asset.role(),
                records: asset.records(),
            };
            let mut matches = children
                .iter()
                .filter(|entity| world.get::<GrownPartBinding>(*entity) == Some(&binding));
            let entity = matches
                .next()
                .context("generated source partition missing")?;
            ensure!(
                matches.next().is_none(),
                "duplicate generated source partition"
            );
            let material = match asset.material() {
                GrownMaterialHandle::Wood(handle) => handle.id().untyped(),
                GrownMaterialHandle::Leaves(handle) => handle.id().untyped(),
            };
            let source_frame = Transform::from_translation(
                attachment.frame.translation
                    - Vec3::Y
                        * (adventuresim_tactical_core::prelude::TREE_TRUNK_HEIGHT_METRES * 0.5),
            );
            parts.push(Part {
                parent,
                binding,
                bounds: asset.bounds(),
                member: Member {
                    entity,
                    root: attachment.root,
                    part: PartIdentity {
                        bank: attachment.bank,
                        source: attachment.source,
                        level,
                        encoding: assets.encoding(),
                        ordinal: PartOrdinal(ordinal),
                        role: asset.role(),
                        records: asset.records(),
                    },
                    mesh: asset.mesh().id(),
                    material,
                    transform: source_frame.to_matrix(),
                },
            });
        }
        Ok(Self {
            entity,
            level,
            parts,
        })
    }
}
