//! The scalp under the hair, shaded as the hair crate shades its own head.
//!
//! Between strands and cards the skin would show bare and lit. Where roots
//! grow, the skin takes on the hair's colour at the root; where hair lies
//! over it, the skin falls into its shadow. Only the scalp cap is shaded --
//! the body's own skin, the neck and the shoulders hair falls over, keeps its
//! colour -- and the cap comes and goes with the hair it lies under. The cap
//! is blended over the skin, fading out towards its edge.
use super::*;

/// The material the scalp cap is drawn with: the skin's, blended over it.
pub(super) fn material() -> StandardMaterial {
    StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        ..preview::skin_material()
    }
}

/// How dark hair is at its root, as a share of its colour.
const ROOT_SHADE: f32 = 0.7;
/// The most the colour of growing roots replaces the skin's.
const ROOT_TINT: f32 = 0.95;
/// The root density at which the scalp takes the roots' colour fully: a
/// scalp is hidden under its hair wherever hair grows at all densely, as
/// between an afro's open curls.
const DENSE_ROOTS: f32 = 0.35;
/// The most hair lying over the skin darkens it.
const OVER_SHADOW: f32 = 0.6;

/// The vertex colours that shade the skin under hair of `design`, each
/// multiplying the skin colour, for skin `cover`ed as given and showing by
/// `opacity`.
pub(super) fn colors(design: HairDesign, cover: &[SkinCover], opacity: &[f32]) -> Vec<[f32; 4]> {
    let skin = preview::SKIN_COLOR.to_linear().to_f32_array_no_alpha();
    let root = design.pigment.color().map(|channel| channel * ROOT_SHADE);
    cover
        .iter()
        .zip(opacity)
        .map(|(cover, &opacity)| {
            let dense = (cover.roots / DENSE_ROOTS).clamp(0.0, 1.0);
            let tint = dense * dense * (3.0 - 2.0 * dense) * ROOT_TINT;
            let shadow = 1.0 - OVER_SHADOW * cover.over;
            let [r, g, b] = std::array::from_fn(|k| {
                let shaded = skin[k] + (root[k] - skin[k]) * tint;
                shaded / skin[k].max(f32::EPSILON) * shadow
            });
            [r, g, b, opacity]
        })
        .collect()
}
