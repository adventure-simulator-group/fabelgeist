use super::*;

#[derive(Component)]
pub struct ArmorSkin(pub BodySkin);

pub(super) fn posed_colliders(
    body: &BodySkin,
    armor: &Query<&ArmorSkin>,
    matrices: &[Mat4],
) -> (Vec<Vec3>, Vec<Vec3>, Vec<[u32; 3]>) {
    let mut rest = Vec::new();
    let mut end = Vec::new();
    let mut faces = Vec::new();
    for collider in std::iter::once(body).chain(armor.iter().map(|part| &part.0)) {
        let offset = rest.len() as u32;
        rest.extend(collider.positions.iter().copied().map(Vec3::from_array));
        end.extend(skinned_positions(
            &collider.positions,
            &collider.indices,
            &collider.weights,
            matrices,
        ));
        faces.extend(collider.faces.iter().map(|face| face.map(|i| i + offset)));
    }
    (rest, end, faces)
}
