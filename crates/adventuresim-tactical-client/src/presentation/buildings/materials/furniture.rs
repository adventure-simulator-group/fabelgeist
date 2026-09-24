//! Wood treatments share the ordinary furniture's metre-space grain maps.
use super::*;
use adventuresim_building_generator::furniture::FurnitureWoodSurface;

pub(super) fn materials(
    textures: &ProceduralTextureAssets,
    assets: &mut Assets<StandardMaterial>,
) -> [Handle<StandardMaterial>; 4] {
    [
        FurnitureWoodSurface::Handled,
        FurnitureWoodSurface::Replacement,
        FurnitureWoodSurface::ReplacementEndGrain,
        FurnitureWoodSurface::Painted,
    ]
    .map(|surface| {
        let mut material = surface_material(&textures.hewn_oak, HEWN_OAK_TILE_METRES);
        match surface {
            FurnitureWoodSurface::Handled => {
                material.base_color = Color::srgb(1.04, 1.02, 1.0);
                material.perceptual_roughness = 0.58;
                material.normal_map_texture = None;
            }
            FurnitureWoodSurface::Replacement => {
                material.base_color = Color::srgb(1.22, 1.16, 1.06);
                material.perceptual_roughness = 0.94;
            }
            FurnitureWoodSurface::ReplacementEndGrain | FurnitureWoodSurface::Painted => {
                let reference = BuildingLodMaterial::FurnitureWood(surface)
                    .workplace_surface()
                    .unwrap();
                material.base_color =
                    Color::srgb(reference.srgb[0], reference.srgb[1], reference.srgb[2]);
                material.base_color_texture = None;
                material.metallic_roughness_texture = None;
                material.metallic = 0.0;
                material.perceptual_roughness = reference.perceptual_roughness;
                if surface == FurnitureWoodSurface::ReplacementEndGrain {
                    material.normal_map_texture = None;
                }
            }
        }
        assets.add(material)
    })
}
