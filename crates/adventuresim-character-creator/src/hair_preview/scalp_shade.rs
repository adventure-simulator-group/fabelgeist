//! The scalp under the hair, painted the hair's colour.
//!
//! Between strands and cards the skin would show bare. The scalp cap is
//! drawn flat in the hair's own colour instead, unlit, so it neither shades
//! nor reflects the light: wherever the hair parts, the eye meets more hair
//! colour rather than lit skin. Only the cap is painted -- the body's own
//! skin, the neck and the shoulders hair falls over, keeps its colour -- and
//! the cap comes and goes with the hair it lies under. The cap is blended
//! over the skin, fading out towards its edge.
use super::*;

/// The material the scalp cap is drawn with: its vertex colours alone,
/// unlit, blended over the skin.
pub(super) fn material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }
}

/// The vertex colours that paint the scalp under hair of `design`, showing
/// by `opacity`.
pub(super) fn colors(design: HairDesign, opacity: &[f32]) -> Vec<[f32; 4]> {
    let [red, green, blue] = design.pigment.color();
    opacity
        .iter()
        .map(|&opacity| [red, green, blue, opacity])
        .collect()
}
