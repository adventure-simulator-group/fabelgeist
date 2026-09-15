//! Exhibit identities and catalog-backed framing for existing assets.

use adventuresim_weapon_model::{default_design, generate};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use super::{
    DemoEntity, StudioEntity,
    camera::{CameraSpace, OrbitView},
    scenery,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ExhibitId {
    Henry,
    Nuremberg,
    Longsword,
    ArmingSword,
    Rapier,
    Spear,
    Halberd,
    Dagger,
    City,
    Oak,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ExhibitKind {
    Armor,
    Weapon,
    Scenery,
}

#[derive(Deserialize)]
pub(super) struct Exhibit {
    id: ExhibitId,
    kind: ExhibitKind,
    asset: Option<String>,
    focus: [f32; 3],
    distance: f32,
}

impl Exhibit {
    pub fn is_studio(&self) -> bool {
        !matches!(self.kind, ExhibitKind::Scenery)
    }

    pub fn catalog() -> Vec<Self> {
        serde_json::from_str(include_str!("../../../../assets/art-demo/catalog.json"))
            .expect("shipped exhibit catalog must be valid")
    }

    pub fn id(&self) -> ExhibitId {
        self.id
    }

    pub fn get(id: ExhibitId) -> Self {
        Self::catalog()
            .into_iter()
            .find(|exhibit| exhibit.id == id)
            .expect("exhibit in catalog")
    }

    pub fn view(&self) -> OrbitView {
        OrbitView::new(
            Vec3::from_array(self.focus),
            self.distance,
            if matches!(self.kind, ExhibitKind::Scenery) {
                CameraSpace::Landscape
            } else {
                CameraSpace::Studio
            },
        )
    }

    pub fn spawn(&self, world: &mut World) -> Result<Option<Handle<WorldAsset>>, String> {
        match self.kind {
            ExhibitKind::Armor => {
                let asset = self
                    .asset
                    .as_ref()
                    .ok_or("armor exhibit requires an asset")?;
                let handle = world
                    .resource::<AssetServer>()
                    .load(GltfAssetLabel::Scene(0).from_asset(format!("art-demo/armor/{asset}")));
                world.spawn((
                    DemoEntity(self.id),
                    WorldAssetRoot(handle.clone()),
                    Transform::IDENTITY,
                    Visibility::default(),
                ));
                Ok(Some(handle))
            }
            ExhibitKind::Weapon => {
                let catalog_id = self
                    .asset
                    .as_deref()
                    .ok_or("weapon exhibit requires a chassis")?;
                weapon(world, self.id, catalog_id)?;
                Ok(None)
            }
            ExhibitKind::Scenery => {
                scenery::spawn(world, self.id)?;
                Ok(None)
            }
        }
    }
}

pub(super) fn studio(world: &mut World) {
    // The studio dome occludes the outdoor sky without replacing its lighting cache.
    const STUDIO_DOME_RADIUS_METRES: f32 = 50.0;
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(Sphere::new(STUDIO_DOME_RADIUS_METRES));
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb_u8(23, 27, 29),
            unlit: true,
            cull_mode: None,
            ..default()
        });
    world.spawn((
        StudioEntity,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::IDENTITY,
    ));
    for (position, illuminance) in [
        (Vec3::new(-3.0, 4.0, 5.0), 60_000.0),
        (Vec3::new(4.0, 1.0, -2.0), 25_000.0),
    ] {
        world.spawn((
            StudioEntity,
            DirectionalLight {
                illuminance,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_translation(position).looking_at(Vec3::Y, Vec3::Y),
        ));
    }
}

fn weapon(world: &mut World, id: ExhibitId, catalog_id: &str) -> Result<(), String> {
    let design = default_design(catalog_id).ok_or("unknown weapon chassis")?;
    let generated = generate(&design).map_err(|error| format!("weapon generation: {error:?}"))?;
    let center =
        (Vec3::from_array(generated.bounds.min) + Vec3::from_array(generated.bounds.max)) * 0.5;
    let extent = Vec3::from_array(generated.bounds.max) - Vec3::from_array(generated.bounds.min);
    const DISPLAY_LENGTH_METRES: f32 = 1.5;
    let scale = DISPLAY_LENGTH_METRES / extent.max_element();
    let root = world
        .spawn((
            DemoEntity(id),
            Transform::from_rotation(Quat::from_rotation_z(-0.35)).with_scale(Vec3::splat(scale)),
            Visibility::default(),
        ))
        .id();
    for part in generated.parts {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals);
        mesh.insert_indices(Indices::U32(part.indices));
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = world.resource_mut::<Assets<StandardMaterial>>().add(
            crate::weapon_preview_material::preview_material(part.material),
        );
        world.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(-center),
            ChildOf(root),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_resolves_shipped_assets_and_generatable_weapon_designs() {
        let catalog: Vec<Exhibit> =
            serde_json::from_str(include_str!("../../../../assets/art-demo/catalog.json")).unwrap();
        for exhibit in catalog {
            match exhibit.kind {
                ExhibitKind::Armor => assert!(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../assets/art-demo/armor")
                        .join(exhibit.asset.unwrap())
                        .is_file()
                ),
                ExhibitKind::Weapon => {
                    generate(&default_design(&exhibit.asset.unwrap()).unwrap()).unwrap();
                }
                ExhibitKind::Scenery => {}
            }
        }
    }
}
