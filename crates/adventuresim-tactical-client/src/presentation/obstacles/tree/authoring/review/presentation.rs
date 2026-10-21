//! Actual source geometry attachment and the exact authored asset identities.
use super::*;
use bevy::{asset::UntypedAssetId, pbr::Material};

pub(crate) struct TreeSourcePresentation {
    pub(crate) root: Entity,
    pub(crate) parts: Vec<TreeSourceMeshBinding>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TreeSourceMeshBinding {
    pub(crate) part: TreeSourcePart,
    pub(crate) mesh: AssetId<Mesh>,
    pub(crate) material: UntypedAssetId,
}

struct SourceMaterials {
    bark: Handle<TacticalTreeBarkMaterial>,
    leaf: Handle<TacticalTreeLeafCardMaterial>,
}

impl TreeSourceGeometry {
    /// Root is actual ground in area-world metres; source vertices stay collider-centred.
    pub(crate) fn spawn(
        self,
        world: &mut World,
        root_metres: Vec3,
    ) -> Result<TreeSourcePresentation> {
        let materials = SourceMaterials::bind(world, self.species)?;
        let root = world
            .spawn((
                Transform::from_translation(root_metres + source_normalization()),
                Visibility::Inherited,
            ))
            .id();
        let mut parts = vec![
            TreeSourceMeshBinding::spawn(
                world,
                root,
                TreeSourcePart::Wood,
                self.wood,
                materials.bark,
            ),
            TreeSourceMeshBinding::spawn(
                world,
                root,
                TreeSourcePart::Foliage,
                self.foliage,
                materials.leaf,
            ),
        ];
        if let Some(mesh) = self.buds {
            let material = world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(tree_bud_material(self.species));
            parts.push(TreeSourceMeshBinding::spawn(
                world,
                root,
                TreeSourcePart::Buds,
                mesh,
                material,
            ));
        }
        Ok(TreeSourcePresentation { root, parts })
    }
}

impl SourceMaterials {
    fn bind(world: &mut World, species: TreePresentationSpecies) -> Result<Self> {
        let terrain = world
            .query::<&SceneTerrain>()
            .single(world)
            .context("one actual production terrain for source contact")?
            .clone();
        let textures = world.resource::<ProceduralTextureAssets>().clone();
        let heightmap = world
            .resource_mut::<Assets<Image>>()
            .add(terrain_heightmap_image(&terrain));
        let range = Vec2::new(terrain.minimum_height(), terrain.maximum_height());
        let bark = match species {
            TreePresentationSpecies::EnglishOak => {
                oak_bark_material(&textures, heightmap, range, &terrain)
            }
            TreePresentationSpecies::CommonBeech => {
                beech_bark_material(&textures, heightmap, range, &terrain)
            }
        };
        let leaf = match species {
            TreePresentationSpecies::EnglishOak => oak_leaf_material(&textures),
            TreePresentationSpecies::CommonBeech => beech_leaf_material(&textures),
        };
        Ok(Self {
            bark: world
                .resource_mut::<Assets<TacticalTreeBarkMaterial>>()
                .add(bark),
            leaf: world
                .resource_mut::<Assets<TacticalTreeLeafCardMaterial>>()
                .add(leaf),
        })
    }
}

impl TreeSourceMeshBinding {
    fn spawn<M: Material>(
        world: &mut World,
        root: Entity,
        part: TreeSourcePart,
        mesh: Mesh,
        material: Handle<M>,
    ) -> Self {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let binding = Self {
            part,
            mesh: mesh.id(),
            material: material.id().untyped(),
        };
        world.spawn((
            part,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            ChildOf(root),
            Transform::IDENTITY,
            Visibility::Inherited,
        ));
        binding
    }
}
