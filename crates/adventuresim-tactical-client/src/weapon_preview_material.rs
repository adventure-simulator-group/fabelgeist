//! Material palette shared by the browser forge and art showcase.
use adventuresim_weapon_model::MaterialClass;
use bevy::prelude::*;

pub(crate) fn preview_material(class: MaterialClass) -> StandardMaterial {
    let base_color = match class {
        MaterialClass::Wood => Color::srgb(0.30, 0.18, 0.09),
        MaterialClass::Leather => Color::srgb(0.16, 0.09, 0.05),
        MaterialClass::DarkLeather => Color::srgb(0.055, 0.045, 0.038),
        MaterialClass::Brass => Color::srgb(0.68, 0.50, 0.18),
        MaterialClass::Steel => Color::srgb(0.68, 0.72, 0.76),
        MaterialClass::DarkSteel => Color::srgb(0.30, 0.33, 0.37),
    };
    let metallic = matches!(
        class,
        MaterialClass::Brass | MaterialClass::Steel | MaterialClass::DarkSteel
    );
    StandardMaterial {
        base_color,
        metallic: if metallic { 0.35 } else { 0.0 },
        perceptual_roughness: if metallic { 0.48 } else { 0.76 },
        ..default()
    }
}
