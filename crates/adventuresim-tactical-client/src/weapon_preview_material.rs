//! Material palette shared by the browser forge and art showcase.
use adventuresim_weapon_model::Material as WeaponMaterial;
use bevy::prelude::*;

pub(crate) fn preview_material(class: WeaponMaterial) -> StandardMaterial {
    let base_color = match class {
        WeaponMaterial::Wood => Color::srgb(0.30, 0.18, 0.09),
        WeaponMaterial::Leather => Color::srgb(0.16, 0.09, 0.05),
        WeaponMaterial::DarkLeather => Color::srgb(0.055, 0.045, 0.038),
        WeaponMaterial::Brass => Color::srgb(0.68, 0.50, 0.18),
        WeaponMaterial::Steel => Color::srgb(0.68, 0.72, 0.76),
        WeaponMaterial::DarkSteel => Color::srgb(0.30, 0.33, 0.37),
        material => {
            let [r, g, b] = material.color().map(|n| n as f32);
            Color::srgb(r, g, b)
        }
    };
    let metallic = class.is_metal();
    StandardMaterial {
        base_color,
        metallic: if metallic { 0.35 } else { 0.0 },
        perceptual_roughness: if metallic { 0.48 } else { 0.76 },
        ..default()
    }
}
