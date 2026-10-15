//! The scalp under the hair, painted with the hair lying on it.
//!
//! Between strands and cards the skin would show bare. The scalp cap is
//! drawn in the hair's own colour instead, by the texture baked from the
//! groom -- the strands lying along the scalp, in shade -- and unlit, so it
//! neither shades nor reflects the light: wherever the hair parts, the eye
//! meets more hair rather than lit skin, and the gaps between cards recede
//! rather than glow. Only the cap is painted -- the body's own skin, the
//! neck and the shoulders hair falls over, keeps its colour -- and the cap
//! comes and goes with the hair it lies under. The cap is blended over the
//! skin, fading out towards its edge.
use super::*;

/// The material the scalp cap is drawn with: its vertex colours by the
/// baked `texture`, unlit, blended over the skin.
pub(super) fn material(texture: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(texture),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }
}

/// The vertex colours that paint the scalp under hair of `design`, showing
/// by `opacity`: the texture shades them.
pub(super) fn colors(design: HairDesign, opacity: &[f32]) -> Vec<[f32; 4]> {
    let [red, green, blue] = design.pigment.color();
    opacity
        .iter()
        .map(|&opacity| [red, green, blue, opacity])
        .collect()
}
