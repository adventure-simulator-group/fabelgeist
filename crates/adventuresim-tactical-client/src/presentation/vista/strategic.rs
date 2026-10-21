//! Sampled strategic scenery never becomes tactical terrain or ground support.
use super::*;
use adventuresim_tactical_core::{
    city_layout::grounding::SupportElevation,
    scene::{TerrainSupportError, TerrainSupportResult},
    scene_coordinates::ScenePlanPoint,
};
mod colors;

pub(crate) type BackdropResult<T> = Result<T, BackdropError>;

#[derive(Component, Clone)]
pub(crate) struct StrategicBackdrop {
    terrain: SceneTerrain,
    lods: Vec<VistaLod>,
    foreground: Vec<EnvironmentalSample>,
}

pub(crate) struct PreparedBackdrop {
    backdrop: StrategicBackdrop,
    environment: SceneEnvironment,
    chunks: Vec<BackdropChunk>,
}

struct BackdropChunk {
    mesh: Mesh,
    origin: Vec3,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum BackdropError {
    #[error("Strategic backdrop height samples are invalid")]
    Heightmap,
    #[error(transparent)]
    VistaMesh(#[from] VistaMeshError),
}

impl StrategicBackdrop {
    pub(crate) fn new(
        input: &TacticalSceneInput,
        maximum_lods: usize,
    ) -> BackdropResult<Self> {
        let terrain = SceneTerrain::from_heightmap(
            usize::from(input.playable.width),
            usize::from(input.playable.depth),
            input.playable.spacing_metres,
            input.playable.heights_metres.clone(),
        )
        .ok_or(BackdropError::Heightmap)?;
        if input.playable.environment.len() != input.playable.heights_metres.len() {
            return Err(BackdropError::Heightmap);
        }
        Ok(Self {
            terrain,
            lods: input
                .vista
                .lods
                .iter()
                .take(maximum_lods)
                .cloned()
                .collect(),
            foreground: input.playable.environment.clone(),
        })
    }

    pub(crate) fn weather_surface(&self) -> &SceneTerrain {
        &self.terrain
    }

    pub(crate) fn required_height_at(
        &self,
        point: ScenePlanPoint,
    ) -> TerrainSupportResult<SupportElevation> {
        self.height_at(point)
            .ok_or(TerrainSupportError::MissingSupport { point })
    }

    pub(crate) fn height_at(&self, point: ScenePlanPoint) -> Option<SupportElevation> {
        if let Some(height) = self.terrain.height_at(point) {
            return Some(height);
        }
        let mut inner = Vec2::new(self.terrain.width(), self.terrain.depth()) * 0.5;
        for (index, lod) in self.lods.iter().enumerate() {
            let height = adventuresim_tactical_core::vista_surface::sampled_vista_triangle_height(
                lod,
                self.lods.get(index + 1),
                (index == 0).then_some(&self.terrain),
                inner,
                point,
            );
            if height.is_some() {
                return height;
            }
            inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres
                * 0.5;
        }
        None
    }
}

impl PreparedBackdrop {
    pub(crate) fn new(
        backdrop: StrategicBackdrop,
        environment: SceneEnvironment,
    ) -> BackdropResult<Self> {
        let mut chunks = Vec::new();
        let mut mesh = backdrop.terrain.mesh();
        mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0);
        let foreground = backdrop.foreground.iter()
            .map(|sample| pigment::vista_sample_color(*sample, environment.weather).to_array())
            .collect::<Vec<_>>();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, foreground.clone());
        chunks.push(BackdropChunk { mesh, origin: Vec3::ZERO });
        let lods = backdrop.lods.iter().collect::<Vec<_>>();
        let mut inner = Vec2::new(backdrop.terrain.width(), backdrop.terrain.depth()) * 0.5;
        for (index, lod) in lods.iter().copied().enumerate() {
            for mut mesh in natural::sampled_vista_lod_meshes_with_morph(
                lod,
                inner,
                lods.get(index + 1).copied(),
                (index == 0).then_some(&backdrop.terrain),
                None,
                environment.weather,
            )? {
                if index == 0 {
                    backdrop.stitch_colors(&mut mesh, &foreground, lod.spacing_metres);
                }
                let origin = Vec3::new(
                    lod.origin_east_metres as f32, 0.0, lod.origin_north_metres as f32,
                );
                chunks.push(BackdropChunk { mesh, origin });
            }
            inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres * 0.5;
        }
        Ok(Self { backdrop, environment, chunks })
    }

    pub(crate) fn backdrop(&self) -> &StrategicBackdrop {
        &self.backdrop
    }

    pub(crate) fn spawn(self, commands: &mut Commands, root: Entity) {
        let Self { backdrop, environment, chunks } = self;
        commands.spawn((environment.clone(), backdrop, ChildOf(root)));
        commands.queue(move |world: &mut World| {
            let material = world.resource_mut::<Assets<TacticalVistaMaterial>>().add(vista_material(
                environment.weather, grass_terminal_pigment(&environment),
            ));
            for chunk in chunks {
                spawn_mesh(world, root, chunk.mesh, material.clone(), chunk.origin);
            }
        });
    }
}

fn spawn_mesh(
    world: &mut World,
    root: Entity,
    mesh: Mesh,
    material: Handle<TacticalVistaMaterial>,
    origin: Vec3,
) {
    let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
    world.spawn((
        NotShadowCaster,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(origin),
        ChildOf(root),
    ));
}
