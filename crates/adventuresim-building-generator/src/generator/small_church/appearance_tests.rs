use super::*;

#[test]
fn pointed_window_crowns_remain_open_in_detail_and_collision() {
    for usage in [BuildingUse::Chapel, BuildingUse::ParishChurch] {
        let plan = crate::generate(&BuildingProgram::settlement(
            BuildingArchetype::ParishChurch,
            Some(usage),
            42,
        ))
        .unwrap();
        let collision = crate::compile_building_collision(&plan);
        for opening in plan
            .opening_assemblies
            .iter()
            .filter(|opening| opening.use_kind == crate::OpeningUse::Window)
        {
            let crate::OpeningProfile::PointedTwoCentred {
                width_metres,
                spring_height_metres,
                apex_height_metres,
                ..
            } = opening.profile
            else {
                panic!("sacred window must have a pointed head");
            };
            let solid = plan
                .resolved_geometry
                .solids
                .iter()
                .find(|solid| solid.id == opening.head_solid)
                .unwrap();
            let detail = crate::detail::compile_solid_detail(&plan, solid);
            let tangent = Vec3::new(opening.frame.tangent.x, 0.0, opening.frame.tangent.y);
            let clear = Vec3::new(
                opening.frame.origin.x,
                opening.sill_elevation_metres + (spring_height_metres + apex_height_metres) * 0.5,
                opening.frame.origin.y,
            );
            assert!(
                !detail_covers(&detail, tangent, clear),
                "arch crown rendered as a solid box"
            );
            assert!(
                !collision
                    .cuboids
                    .iter()
                    .any(|cuboid| { (clear - cuboid.centre).abs().cmplt(cuboid.size * 0.5).all() }),
                "arch crown blocks tactical collision"
            );
            let bearing = clear + tangent * (width_metres * 0.5 + 0.04);
            assert!(
                detail_covers(&detail, tangent, bearing),
                "arch bearing disappeared with the cut"
            );
            let crown = Vec3::new(
                clear.x,
                opening.sill_elevation_metres + apex_height_metres,
                clear.z,
            );
            assert!(
                detail
                    .meshes
                    .iter()
                    .flat_map(|mesh| &mesh.vertices)
                    .any(|vertex| {
                        (vertex.position - crown).dot(tangent).abs() < 0.001
                            && (vertex.position.y - crown.y).abs() < 0.001
                    }),
                "visible tessellation does not reach the authored pointed apex"
            );
        }
    }
}

fn detail_covers(detail: &crate::BuildingDetail, tangent: Vec3, point: Vec3) -> bool {
    let project = |point: Vec3| Vec2::new(point.dot(tangent), point.y);
    let point = project(point);
    detail.meshes.iter().any(|mesh| {
        mesh.indices.as_chunks::<3>().0.iter().any(|indices| {
            let [a, b, c] = std::array::from_fn(|index| {
                project(mesh.vertices[indices[index] as usize].position)
            });
            let area = (b - a).perp_dot(c - a);
            if area.abs() < 0.000_001 {
                return false;
            }
            let first = (b - a).perp_dot(point - a) / area;
            let second = (c - b).perp_dot(point - b) / area;
            let third = (a - c).perp_dot(point - c) / area;
            first >= 0.0 && second >= 0.0 && third >= 0.0
        })
    })
}

#[test]
fn pointed_and_segmental_panels_follow_the_authored_spring_and_crown() {
    let plan = crate::generate(&BuildingProgram::fixture(
        BuildingArchetype::ParishChurch,
        42,
    ))
    .unwrap();
    let source = &plan.opening_assemblies[0];
    let mut panel = plan
        .resolved_geometry
        .solids
        .iter()
        .find(|solid| solid.id == source.head_solid)
        .unwrap()
        .clone();
    // Isolate both supported arch profiles from the wall's host ownership:
    // their clear span and spring heights define the actual panel silhouette.
    panel.id = ResolvedItemId(u64::MAX);
    panel.owner = GeometryOwnerId(u32::MAX);
    panel.role = SolidRole::LeadedGlazing;
    panel.centre = Vec3::ZERO;
    for pointed in [false, true] {
        let rise = if pointed { 1.4 } else { 0.5 };
        panel.size = Vec3::new(2.0, 2.0 + rise, 0.05);
        panel.shape = if pointed {
            crate::ResolvedSolidShape::PointedArchRing {
                clear_span_metres: 2.0,
                spring_height_metres: 2.0,
                apex_height_metres: 2.0 + rise,
                arc_radius_metres: (1.0 + rise * rise) * 0.5,
                ring_depth_metres: 0.2,
            }
        } else {
            crate::ResolvedSolidShape::SegmentalArchRing {
                clear_span_metres: 2.0,
                spring_height_metres: 2.0,
                rise_metres: rise,
                ring_depth_metres: 0.2,
            }
        };
        let detail = crate::detail::compile_solid_detail(&plan, &panel);
        let spring = 2.0 - panel.size.y * 0.5;
        let crown = panel.size.y * 0.5;
        assert!(detail_covers(
            &detail,
            Vec3::X,
            Vec3::new(0.0, crown - 0.05, 0.0)
        ));
        assert!(!detail_covers(
            &detail,
            Vec3::X,
            Vec3::new(0.95, crown - 0.05, 0.0)
        ));
        for expected in [Vec2::new(1.0, spring), Vec2::new(0.0, crown)] {
            assert!(
                detail
                    .meshes
                    .iter()
                    .flat_map(|mesh| &mesh.vertices)
                    .any(|vertex| {
                        Vec2::new(vertex.position.x, vertex.position.y).distance(expected) < 0.001
                    }),
                "panel does not retain its authored spring/crown"
            );
        }
    }
}

#[test]
fn lower_chancel_verge_abuts_the_nave_and_gables_keep_wall_material() {
    for size in [
        ServiceBuildingSize::Small,
        ServiceBuildingSize::Medium,
        ServiceBuildingSize::Large,
    ] {
        let plan = crate::generate(
            &BuildingProgram::fixture(BuildingArchetype::ParishChurch, 42).with_service_size(size),
        )
        .unwrap();
        let church = plan.small_church.as_ref().unwrap();
        let chancel = &plan.roof_assemblies[1];
        for face in &chancel.faces {
            assert!(
                face.polygon
                    .iter()
                    .all(|point| point.z >= church.nave_depth_metres - 0.001),
                "chancel tiles extend into nave"
            );
        }
        for roof in &plan.roof_assemblies[..2] {
            assert!(
                roof.enclosure_faces
                    .iter()
                    .all(|face| face.material == RoofMaterial::RubbleInfill)
            );
        }
        let belfry = plan.roof_assemblies.last().unwrap();
        assert!(
            belfry
                .enclosure_faces
                .iter()
                .any(|face| face.material == RoofMaterial::TimberInfill)
        );
    }
}
