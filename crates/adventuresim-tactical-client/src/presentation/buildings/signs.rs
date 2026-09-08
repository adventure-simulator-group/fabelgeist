//! Per-establishment signs reuse building anchor geometry; lettering is allocated only nearby.
use super::*;
use adventuresim_building_generator::signs::{
    EstablishmentId, ShopSign, ShopSignRenderCache, SignDetail, SignMount, SignRenderAssets,
    SignRenderPart, SignSite,
};
use bevy::ecs::system::SystemParam;

const LETTERING_LOAD_DISTANCE_METRES: f32 = 48.0;
const LETTERING_RELEASE_DISTANCE_METRES: f32 = 60.0;

pub(in crate::presentation) struct BuildingPresentationPlugin;
impl Plugin for BuildingPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopSignRenderCache>()
            .add_observer(super::on_scene_building_added)
            .add_observer(super::on_scene_vista_buildings)
            .add_systems(Update, update_lettering);
    }
}

#[derive(SystemParam)]
pub(super) struct SignAssets<'w> {
    cache: ResMut<'w, ShopSignRenderCache>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    images: ResMut<'w, Assets<Image>>,
}

#[derive(Component)]
struct PresentedSign {
    sign: ShopSign,
    site: SignSite,
    origin: Vec3,
    lettering: Option<Entity>,
}

impl SignAssets<'_> {
    pub(super) fn spawn(
        &mut self,
        parent: &mut ChildSpawnerCommands,
        id: u64,
        compiled: &CompiledBuildingLevels,
        meshes: &mut Assets<Mesh>,
    ) {
        let Some(usage) = compiled.program.usage else {
            return;
        };
        let Some(mut sign) = ShopSign::for_establishment(EstablishmentId(id), usage) else {
            return;
        };
        let Some(&(mount, site)) = compiled
            .sign_sites
            .iter()
            .find(|(mount, _)| *mount == sign.mount)
            .or_else(|| compiled.sign_sites.first())
        else {
            return;
        };
        sign.mount = mount;
        let parts = self.cache.compile(
            &sign,
            site,
            compiled.local_origin,
            SignDetail::Board,
            SignRenderAssets {
                meshes,
                materials: &mut self.materials,
                images: &mut self.images,
            },
        );
        parent
            .spawn((
                Name::new(sign.name.text()),
                Transform::default(),
                Visibility::default(),
                PresentedSign {
                    sign,
                    site,
                    origin: compiled.local_origin,
                    lettering: None,
                },
            ))
            .with_children(|parent| spawn_parts(parent, parts));
    }
}

fn spawn_parts(parent: &mut ChildSpawnerCommands, parts: Vec<SignRenderPart>) {
    for part in parts {
        let visibility = part.visibility();
        parent.spawn((
            Mesh3d(part.mesh),
            MeshMaterial3d(part.material),
            part.transform,
            visibility,
        ));
    }
}

fn update_lettering(
    mut commands: Commands,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut signs: Query<(Entity, &GlobalTransform, &mut PresentedSign)>,
    mut assets: SignAssets,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, transform, mut presented) in &mut signs {
        let position = transform
            .transform_point(presented.site.board(presented.sign.mount).centre - presented.origin);
        let distance = cameras
            .iter()
            .filter(|(camera, _)| camera.is_active)
            .map(|(_, camera)| camera.translation().distance(position))
            .fold(f32::MAX, f32::min);
        if distance < LETTERING_LOAD_DISTANCE_METRES && presented.lettering.is_none() {
            let SignAssets {
                ref mut cache,
                ref mut materials,
                ref mut images,
            } = assets;
            let parts = cache.compile(
                &presented.sign,
                presented.site,
                presented.origin,
                SignDetail::Lettering,
                SignRenderAssets {
                    meshes: &mut meshes,
                    materials,
                    images,
                },
            );
            let child = commands
                .spawn((Transform::default(), Visibility::default()))
                .with_children(|parent| spawn_parts(parent, parts))
                .id();
            commands.entity(entity).add_child(child);
            presented.lettering = Some(child);
        } else if distance > LETTERING_RELEASE_DISTANCE_METRES {
            if let Some(child) = presented.lettering.take() {
                commands.entity(child).despawn();
            }
        }
    }
}

pub(super) fn sites(
    plan: &adventuresim_building_generator::BuildingPlan,
) -> Vec<(SignMount, SignSite)> {
    let Some(site) = SignSite::for_plan(plan) else {
        return Vec::new();
    };
    [SignMount::Wall, SignMount::Projecting]
        .into_iter()
        .filter(|mount| site.supports(plan, *mount))
        .map(|mount| (mount, site))
        .collect()
}
