//! Shed coverings rise inward and meet the parent slope beyond the cheeks.
use super::*;

// Authored geometry for these civilian fixtures, not a period-wide roof rule.
pub(super) const PITCH_DEGREES: f32 = 22.0;
const SEAM_SEARCH_STEP_METRES: f32 = 0.01;
const SEAM_SEARCH_STEPS: usize = 800;
const ROOF_BUILDUP_METRES: f32 = 0.13;

pub(super) fn seam_depth(
    parent: &RoofAssembly,
    dormer: RoofDormer,
    top: f32,
    eave: f32,
) -> Option<f32> {
    let outward = direction_vector(dormer.facing);
    let tangent = Vec2::new(-outward.y, outward.x);
    let slope = PITCH_DEGREES.to_radians().tan();
    // At the rear wall the parent weather surface must reach the child
    // underside. The remaining overhang seats the covering on the parent.
    let lining_drop = ROOF_BUILDUP_METRES / PITCH_DEGREES.to_radians().cos();
    (1..=SEAM_SEARCH_STEPS)
        .map(|step| step as f32 * SEAM_SEARCH_STEP_METRES)
        .find(|depth| {
            let underside = top + (depth + eave) * slope - lining_drop;
            [-1.0, 1.0].into_iter().all(|side| {
                let point =
                    dormer.centre - outward * *depth + tangent * side * dormer.width_metres * 0.5;
                roof_surface_height_at(parent, point).is_some_and(|height| height >= underside)
            })
        })
}

pub(super) fn underside_height(child: &RoofAssembly, point: Vec2) -> f32 {
    let face = &child.faces[0];
    roof_plane_height(face.plane, point) - face.thickness_metres / face.plane.normal.normalize().y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shed_layout_rotates_with_its_front_and_rejects_an_unreachable_parent() {
        let plan =
            crate::generate(&BuildingProgram::fixture(BuildingArchetype::HallHouse, 42)).unwrap();
        let source = plan.roof_dormers[0];
        for (quarter, facing) in [
            Direction::East,
            Direction::South,
            Direction::West,
            Direction::North,
        ]
        .into_iter()
        .enumerate()
        {
            let rotation =
                bevy::math::Quat::from_rotation_y(quarter as f32 * std::f32::consts::FRAC_PI_2);
            let mut parent = plan.roof_assemblies[0].clone();
            for face in &mut parent.faces {
                face.plane.normal = rotation * face.plane.normal;
                for point in &mut face.polygon {
                    *point = rotation * *point;
                }
            }
            let centre = rotation * Vec3::new(source.centre.x, 0.0, source.centre.y);
            let dormer = RoofDormer {
                centre: Vec2::new(centre.x, centre.z),
                facing,
                ..source
            };
            let layout = dormer_layout::DormerLayout::new(&parent, dormer).unwrap();
            let polygon = roof_face_polygons(layout.recipe, Some(facing.opposite())).remove(0);
            let normal = roof_plane(&polygon).normal;
            assert!(
                Vec2::new(normal.x, normal.z)
                    .normalize()
                    .dot(direction_vector(facing))
                    > 0.999
            );
            for face in &mut parent.faces {
                face.plane.constant += 100.0;
            }
            assert!(matches!(
                dormer_layout::DormerLayout::new(&parent, dormer),
                Err(GenerationError::InvalidRoofDormer)
            ));
        }
    }

    #[test]
    fn shed_covering_falls_outward_and_seats_its_cheeks_on_the_parent() {
        for archetype in [
            BuildingArchetype::FachwerkCottage,
            BuildingArchetype::HallHouse,
        ] {
            for seed in [42, 47, 101] {
                let plan = crate::generate(&BuildingProgram::fixture(archetype, seed)).unwrap();
                assert!(
                    plan.roof_dormers
                        .iter()
                        .any(|dormer| dormer.kind == DormerKind::Shed)
                );
                for (index, dormer) in plan
                    .roof_dormers
                    .iter()
                    .enumerate()
                    .filter(|(_, dormer)| dormer.kind == DormerKind::Shed)
                {
                    let child = plan
                        .roof_assemblies
                        .iter()
                        .find(|roof| roof.id == RoofAssemblyId(1_000 + index as u64))
                        .unwrap();
                    let parent = plan
                        .roof_assemblies
                        .iter()
                        .find(|roof| Some(roof.id) == child.parent)
                        .unwrap();
                    let outward = direction_vector(dormer.facing);
                    let tangent = Vec2::new(-outward.y, outward.x);
                    let front = dormer.centre;
                    assert!(
                        underside_height(child, front - outward) > underside_height(child, front)
                    );
                    assert!(
                        (underside_height(child, front + tangent * 0.5)
                            - underside_height(child, front - tangent * 0.5))
                        .abs()
                            < 0.001
                    );
                    for cheek in child
                        .enclosure_faces
                        .iter()
                        .filter(|face| [0x4101, 0x4102].contains(&(face.id.0 & 0xffff)))
                    {
                        let rear = cheek
                            .polygon
                            .iter()
                            .min_by(|a, b| {
                                Vec2::new(a.x, a.z)
                                    .dot(outward)
                                    .total_cmp(&Vec2::new(b.x, b.z).dot(outward))
                            })
                            .unwrap();
                        let rear_plan = Vec2::new(rear.x, rear.z);
                        let gap = roof_surface_height_at(parent, rear_plan).unwrap()
                            - underside_height(child, rear_plan);
                        assert!((0.0..0.03).contains(&gap), "unseated rear cheek: {gap}");
                        for point in &cheek.polygon {
                            let p = Vec2::new(point.x, point.z);
                            assert!(point.y <= underside_height(child, p) + 0.03);
                        }
                    }
                    assert!(!crate::tessellate_roof_face(&child.faces[0]).is_empty());
                }
            }
        }
    }
}
