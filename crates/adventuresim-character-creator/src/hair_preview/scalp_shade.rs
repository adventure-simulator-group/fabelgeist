//! The scalp under the hair, shaded as the hair crate's own head is.
//!
//! Between strands and cards the skin would show bare and lit. Where roots
//! grow, the skin takes on the hair's colour at the root; where hair lies
//! over it, the skin falls into its shadow. The body's vertex colours carry
//! both, multiplying its skin colour.
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
    let skin = preview::SKIN_COLOR.to_linear().to_f32_array_no_alpha();
    let root = design.pigment.color().map(|channel| channel * ROOT_SHADE);
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
        let colors: Vec<[f32; 4]> = cover
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
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        commands.entity(entity).insert(ScalpShade(design.pigment));
    }
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
