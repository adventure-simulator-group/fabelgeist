//! The scalp under the hair, painted the hair's colour in shade.
//!
//! Between strands and cards the skin would show bare. The scalp cap is
//! drawn flat in the hair's own colour instead, darkened as hair is deep in
//! its own shadow, unlit, so it neither shades nor reflects the light:
//! wherever the hair parts, the eye meets more shadowed hair rather than lit
//! skin, and the gaps between cards recede rather than glow. Only the cap is
//! painted -- the body's own skin, the neck and the shoulders hair falls
//! over, keeps its colour -- and the cap comes and goes with the hair it lies
//! under. The cap is blended over the skin, fading out towards its edge.
use super::*;

/// The share of the hair's colour the scalp is painted in: as dark as hair
/// lies in its own shadow, under the strands over it.
const SCALP_SHADE: f32 = 0.4;

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

/// The vertex colours that paint the scalp under hair of `design`, in
/// shade, showing by `opacity`.
pub(super) fn colors(design: HairDesign, opacity: &[f32]) -> Vec<[f32; 4]> {
    let [red, green, blue] = design.pigment.color().map(|channel| channel * SCALP_SHADE);
    opacity
        .iter()
        .map(|&opacity| [red, green, blue, opacity])
        .collect()
}
