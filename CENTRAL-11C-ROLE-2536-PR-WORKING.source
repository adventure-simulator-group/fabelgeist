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
        child.enclosure_faces.push(RoofEnclosureFace::new(
            ResolvedItemId((0xA_u64 << 60) | (child.id.0 << 16) | 0x4200 | slot as u64),
            vec![
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
            child.support_nodes.clone(),
        ));
    }
}

pub(super) fn bond_front_wall(
    geometry: &mut ResolvedGeometry,
    wall: &crate::WallAssembly,
    parent_owner: Option<GeometryOwnerId>,
    base: f32,
    top: f32,
) -> Result<(), GenerationError> {
    let owner = wall.owner;
    let origin = wall.frame.origin;
    let tangent = wall.frame.tangent;
    let outward = wall.frame.outward;
    let width = wall.length_metres;
    for (bond_slot, roof_owner) in parent_owner.into_iter().enumerate() {
        geometry.junction_bonds.push(JunctionBond {
            id: ResolvedItemId(
                (0x6_u64 << 60) | (u64::from(owner.0) << 16) | (1 + bond_slot as u64),
            ),
            owners: [roof_owner, owner],
            bounds: SpatialBounds::<Architectural>::from_metres(
                Vec3::new(
                    origin.x - tangent.x.abs() * width * 0.55 - outward.x.abs() * 0.30,
                    base - 0.12,
                    origin.y - tangent.y.abs() * width * 0.55 - outward.y.abs() * 0.30,
                ),
                Vec3::new(
                    origin.x + tangent.x.abs() * width * 0.55 + outward.x.abs() * 0.30,
                    top + 0.18,
                    origin.y + tangent.y.abs() * width * 0.55 + outward.y.abs() * 0.30,
                ),
            )?,
            minimum_interface_area_square_metres: 0.005,
            maximum_penetration_metres: 0.18,
        });
    }
    Ok(())
}
