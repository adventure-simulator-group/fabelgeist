//! Scene-bound tree assets and producer census for the visible vista rings.
use super::tree_sites::VistaTreeBand;
use super::*;
use bevy::ecs::system::SystemParam;

#[derive(Component, Default)]
pub(crate) struct VistaTreeCensus {
    pub(crate) roots: Vec<Entity>,
    pub(crate) parts: Vec<Entity>,
}

#[derive(bevy::ecs::query::QueryData)]
pub(in crate::presentation) struct VistaScene {
    pub entity: Entity,
    pub terrain: &'static SceneTerrain,
    pub ground: &'static SceneGround,
    pub environment: &'static SceneEnvironment,
    pub landform: Option<&'static TerrainLandformRecipe>,
    pub content: &'static TreeContentRequest,
}

#[derive(SystemParam)]
pub(in crate::presentation) struct VistaTrees<'w> {
    pub cache: Res<'w, TreePresentationCache>,
    materials: ResMut<'w, Assets<TacticalTreeImpostorMaterial>>,
    variants: ResMut<'w, VistaTreePresentationCache>,
    prepared: PreparedTreeImpostors<'w>,
}

impl VistaTreeCensus {
    pub(super) fn extend(&mut self, band: Self) {
        self.roots.extend(band.roots);
        self.parts.extend(band.parts);
    }
}

impl VistaTrees<'_> {
    pub(super) fn ready_for_scene(
        &self,
        scene: Option<&VistaSceneItem<'_, '_>>,
    ) -> anyhow::Result<bool> {
        scene
            .map(|scene| {
                self.cache
                    .ready_for_vista(scene.entity, scene.environment, scene.content)
            })
            .transpose()
            .map(|ready| ready.unwrap_or(false))
    }

    pub(super) fn spawn_bands(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        images: &mut Assets<Image>,
        scene: Entity,
        content: &TreeContentRequest,
        bands: Vec<VistaTreeBand<'_>>,
    ) -> anyhow::Result<VistaTreeCensus> {
        let mut census = VistaTreeCensus::default();
        for band in bands {
            census.extend(self.spawn_band(commands, meshes, images, scene, content, band)?);
        }
        Ok(census)
    }

    #[cfg_attr(
        not(feature = "grown-trees"),
        expect(
            unused_variables,
            reason = "the shared family boundary uses the scene ownership inputs only when the optional grown runtime is compiled"
        )
    )]
    pub(super) fn spawn_band(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        images: &mut Assets<Image>,
        scene: Entity,
        content: &TreeContentRequest,
        band: VistaTreeBand<'_>,
    ) -> anyhow::Result<VistaTreeCensus> {
        let sites = super::tree_sites::collect(&band);
        match content {
            TreeContentRequest::Recipe => Ok(super::recipe_trees::spawn(
                commands,
                &band,
                &sites,
                meshes,
                &mut self.materials,
                images,
                &mut self.variants,
                self.prepared.get(),
            )),
            #[cfg(feature = "grown-trees")]
            TreeContentRequest::Grown(_) => {
                let catalogue = self.cache.lease_grown_catalogue(scene, band.environment)?;
                super::grown_trees::spawn(commands, scene, &band, &sites, &catalogue)
            }
        }
    }
}
