//! The scalp under the hair, shaded as the hair crate's own head is.
//!
//! Between strands and cards the skin would show bare and lit. Where roots
//! grow, the skin takes on the hair's colour at the root; where hair lies
//! over it, the skin falls into its shadow. Vertex colours carry both,
//! multiplying the skin colour: the scalp cap's, fine enough for a parting,
//! carry both; the body's, far coarser over the skull, only the shadow.
use super::*;
use adventuresim_character_creator::hair::HairPigment;

/// How dark hair is at its root, as a share of its colour.
const ROOT_SHADE: f32 = 0.7;
/// The most the colour of growing roots replaces the skin's.
const ROOT_TINT: f32 = 0.85;
/// The most hair lying over the skin darkens it.
const OVER_SHADOW: f32 = 0.6;

/// The body's scalp is shaded for hair of this pigment.
#[derive(Component)]
pub(crate) struct ScalpShade(HairPigment);

pub(crate) type Bodies<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Mesh3d, Option<&'static ScalpShade>),
    With<animation_preview::BodySkin>,
>;

/// Shade every body's scalp for `design`, unless it already is.
pub(super) fn apply(
    commands: &mut Commands,
    bodies: &Bodies<'_, '_>,
    meshes: &mut Assets<Mesh>,
    design: HairDesign,
    cover: &[SkinCover],
) {
    for (entity, mesh, shade) in bodies {
        if shade.is_some_and(|shade| shade.0 == design.pigment) {
            continue;
        }
        let Some(mut mesh) = meshes.get_mut(&mesh.0) else {
            continue;
        };
        if mesh.count_vertices() != cover.len() {
            continue;
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors(design, &shadow_only(cover)));
        commands.entity(entity).insert(ScalpShade(design.pigment));
    }
}

/// The vertex colours that shade the skin under hair of `design`, each
/// multiplying the skin colour, for skin `cover`ed as given.
pub(super) fn colors(design: HairDesign, cover: &[SkinCover]) -> Vec<[f32; 4]> {
    let skin = preview::SKIN_COLOR.to_linear().to_f32_array_no_alpha();
    let root = design.pigment.color().map(|channel| channel * ROOT_SHADE);
    cover
        .iter()
        .map(|cover| {
            let tint = cover.roots * ROOT_TINT;
            let shadow = 1.0 - OVER_SHADOW * cover.over;
            let [r, g, b] = std::array::from_fn(|k| {
                let shaded = skin[k] + (root[k] - skin[k]) * tint;
                shaded / skin[k].max(f32::EPSILON) * shadow
            });
            [r, g, b, 1.0]
        })
        .collect()
}

/// `cover` with no roots growing: what the body's coarse vertices carry,
/// leaving the roots' tint to the scalp cap.
pub(super) fn shadow_only(cover: &[SkinCover]) -> Vec<SkinCover> {
    cover
        .iter()
        .map(|cover| SkinCover {
            roots: 0.0,
            over: cover.over,
        })
        .collect()
}

/// Return every shaded body to bare skin.
pub(super) fn clear(commands: &mut Commands, bodies: &Bodies<'_, '_>, meshes: &mut Assets<Mesh>) {
    for (entity, mesh, shade) in bodies {
        if shade.is_none() {
            continue;
        }
        if let Some(mut mesh) = meshes.get_mut(&mesh.0) {
            mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR);
        }
        commands.entity(entity).remove::<ScalpShade>();
    }
}
