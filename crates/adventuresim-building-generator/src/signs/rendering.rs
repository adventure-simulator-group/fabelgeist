//! Shared native/browser sign rendering, also consumed by the GPU review fixture.
use super::*;
use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::VisibilityRange,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use std::collections::HashMap;
mod materials;

pub const LETTERING_FADE_START_METRES: f32 = 35.0;
pub const LETTERING_FADE_END_METRES: f32 = 45.0;
const MAX_CACHED_SIGN_TEXTURES: usize = 64;
const BRACKET_WIDTH_METRES: f32 = 0.045;

#[derive(Clone)]
pub struct SignRenderPart {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub lettering: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignDetail {
    Board,
    Lettering,
    Complete,
}

pub struct SignRenderAssets<'a> {
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
    pub images: &'a mut Assets<Image>,
}

impl SignRenderPart {
    pub fn visibility(&self) -> VisibilityRange {
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: if self.lettering {
                LETTERING_FADE_START_METRES..LETTERING_FADE_END_METRES
            } else {
                f32::MAX..f32::MAX
            },
            use_aabb: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct ShopSignRenderCache {
    paint: HashMap<(ShopName, SignFont, SignFinish, u32), Handle<StandardMaterial>>,
    backing: HashMap<SignFinish, Handle<StandardMaterial>>,
    iron: Option<Handle<StandardMaterial>>,
}

impl ShopSignRenderCache {
    pub fn compile(
        &mut self,
        sign: &ShopSign,
        site: SignSite,
        origin: Vec3,
        detail: SignDetail,
        assets: SignRenderAssets<'_>,
    ) -> Vec<SignRenderPart> {
        let SignRenderAssets {
            meshes,
            materials,
            images,
        } = assets;
        let board = site.board(sign.mount);
        let paint = self.painted_material(sign, board, detail, materials, images);
        let backing = self.backing_material(sign.finish, materials);
        let iron = self.iron_material(materials);
        let mut parts = Vec::new();
        if detail != SignDetail::Lettering {
            parts.push(SignRenderPart {
                mesh: meshes.add(Cuboid::new(
                    board.size.x,
                    board.size.y,
                    super::site::PANEL_THICKNESS_METRES,
                )),
                material: backing,
                transform: Transform::from_translation(board.centre - origin)
                    .with_rotation(board.rotation),
                lettering: false,
            });
        }
        let sides = if sign.mount == SignMount::Projecting {
            2
        } else {
            1
        };
        for side in 0..if paint.is_some() { sides } else { 0 } {
            let rotation =
                board.rotation * Quat::from_rotation_y(side as f32 * std::f32::consts::PI);
            let centre = board.centre + rotation * Vec3::Z * super::site::PAINT_OFFSET_METRES;
            parts.push(SignRenderPart {
                mesh: meshes.add(face_mesh(board.size)),
                material: paint.as_ref().unwrap().clone(),
                transform: Transform::from_translation(centre - origin).with_rotation(rotation),
                lettering: true,
            });
        }
        if detail == SignDetail::Lettering {
            return parts;
        }
        let mut metal = |centre: Vec3, size: Vec3, rotation: Quat| {
            parts.push(SignRenderPart {
                mesh: meshes.add(Cuboid::from_size(size)),
                material: iron.clone(),
                transform: Transform::from_translation(centre - origin).with_rotation(rotation),
                lettering: false,
            })
        };
        let top = site.mounting.contact + site.outward * MOUNTING_PLATE_THICKNESS_METRES;
        let hanger_height = top.y - board.centre.y - board.size.y * 0.5;
        let bracket_rotation = Quat::from_rotation_arc(Vec3::Z, site.outward);
        let length = (board.centre - top).dot(site.outward)
            + match sign.mount {
                SignMount::Wall => 0.0,
                SignMount::Projecting => board.size.x * 0.5 + BRACKET_WIDTH_METRES,
            };
        metal(
            top + site.outward * length * 0.5,
            Vec3::new(BRACKET_WIDTH_METRES, BRACKET_WIDTH_METRES, length),
            bracket_rotation,
        );
        if sign.mount == SignMount::Wall {
            metal(
                board.centre + Vec3::Y * (board.size.y * 0.5 + hanger_height),
                Vec3::new(board.size.x, BRACKET_WIDTH_METRES, BRACKET_WIDTH_METRES),
                board.rotation,
            );
        }
        metal(
            site.mounting.contact + site.outward * MOUNTING_PLATE_THICKNESS_METRES * 0.5,
            Vec3::new(
                site.mounting.size.x,
                site.mounting.size.y,
                MOUNTING_PLATE_THICKNESS_METRES,
            ),
            bracket_rotation,
        );
        for side in [-0.35, 0.35] {
            let position = board.centre
                + board.rotation * Vec3::X * (board.size.x * side)
                + Vec3::Y * (board.size.y * 0.5 + hanger_height * 0.5);
            metal(
                position,
                Vec3::new(0.025, hanger_height, 0.025),
                Quat::IDENTITY,
            );
        }
        parts
    }
}

fn face_mesh(size: Vec2) -> Mesh {
    let half = size * 0.5;
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-half.x, -half.y, 0.0],
            [half.x, -half.y, 0.0],
            [half.x, half.y, 0.0],
            [-half.x, half.y, 0.0],
        ],
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 4]);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_world_schema::settlement_buildings::BuildingUse;

    #[test]
    fn distant_boards_allocate_no_textures_and_lettering_is_shared_between_readable_faces() {
        let mut cache = ShopSignRenderCache::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let mut images = Assets::<Image>::default();
        let site = SignSite {
            wall: crate::WallAssemblyId(1),
            attachment: Vec3::new(0.0, 2.7, 0.0),
            outward: Vec3::NEG_Z,
            panel_size: Vec2::new(1.35, 0.62),
            mounting: SignMounting {
                contact: Vec3::new(0.0, 3.13, 0.0),
                size: Vec2::new(0.09, 0.32),
                support: crate::ResolvedItemId(1),
            },
        };
        let mut sign = ShopSign::for_establishment(EstablishmentId(15), BuildingUse::Inn).unwrap();
        sign.mount = SignMount::Projecting;
        let board = cache.compile(
            &sign,
            site,
            Vec3::ZERO,
            SignDetail::Board,
            SignRenderAssets {
                meshes: &mut meshes,
                materials: &mut materials,
                images: &mut images,
            },
        );
        assert!(board.iter().all(|part| !part.lettering));
        assert_eq!(images.len(), 0);
        let letters = cache.compile(
            &sign,
            site,
            Vec3::ZERO,
            SignDetail::Lettering,
            SignRenderAssets {
                meshes: &mut meshes,
                materials: &mut materials,
                images: &mut images,
            },
        );
        assert_eq!(letters.len(), 2);
        assert_eq!(images.len(), 1);
        assert_eq!(letters[0].material, letters[1].material);
        let first = letters[0].transform.rotation * Vec3::Z;
        let second = letters[1].transform.rotation * Vec3::Z;
        assert!(first.dot(second) < -0.99);
        cache.compile(
            &sign,
            site,
            Vec3::ZERO,
            SignDetail::Lettering,
            SignRenderAssets {
                meshes: &mut meshes,
                materials: &mut materials,
                images: &mut images,
            },
        );
        assert_eq!(images.len(), 1);
        sign.name = ShopName::for_establishment(EstablishmentId(16), BuildingUse::Inn).unwrap();
        cache.compile(
            &sign,
            site,
            Vec3::ZERO,
            SignDetail::Lettering,
            SignRenderAssets {
                meshes: &mut meshes,
                materials: &mut materials,
                images: &mut images,
            },
        );
        assert_eq!(
            images.len(),
            2,
            "separate establishments must not share a painted name"
        );
    }
}
