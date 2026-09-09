//! Four weather faces from an attached roof body down to its actual parent slopes.
use super::*;

pub(super) fn append(
    child: &mut RoofAssembly,
    parent: &RoofAssembly,
    recipe: RoofPiece,
    parent_base: f32,
    top: f32,
    material: RoofMaterial,
) {
    let min = recipe.centre - recipe.size * 0.5;
    let max = recipe.centre + recipe.size * 0.5;
    let corners = [
        [Vec2::new(min.x, min.y), Vec2::new(max.x, min.y)],
        [Vec2::new(max.x, max.y), Vec2::new(min.x, max.y)],
        [Vec2::new(min.x, max.y), Vec2::new(min.x, min.y)],
        [Vec2::new(max.x, min.y), Vec2::new(max.x, max.y)],
    ];
    for (slot, [a, b]) in corners.into_iter().enumerate() {
        child.enclosure_faces.push(RoofEnclosureFace {
            id: ResolvedItemId((0xA_u64 << 60) | (child.id.0 << 16) | 0x4200 | slot as u64),
            polygon: vec![
                Vec3::new(
                    a.x,
                    roof_surface_height_at(parent, a).unwrap_or(parent_base),
                    a.y,
                ),
                Vec3::new(
                    b.x,
                    roof_surface_height_at(parent, b).unwrap_or(parent_base),
                    b.y,
                ),
                Vec3::new(b.x, top, b.y),
                Vec3::new(a.x, top, a.y),
            ],
            material,
            support_nodes: child.support_nodes.clone(),
        });
    }
}
