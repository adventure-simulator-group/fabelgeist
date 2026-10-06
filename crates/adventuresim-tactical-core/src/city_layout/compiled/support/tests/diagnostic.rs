//! Optional bounded evidence export; ordinary tests have no filesystem output.
use super::*;
use bevy::math::Vec3Swizzles;

pub(super) fn save(
    population: u32,
    seed: u64,
    layout: &CitySceneLayout,
    source: &GeographicSurface,
    surfaces: &[PropertySupportSurface],
    error: &SettlementSupportError,
) {
    let Some(directory) = std::env::var_os("FABELGEIST_SUPPORT_DIAGNOSTIC_DIR") else {
        return;
    };
    let SettlementSupportError::OwnershipOverlap {
        first,
        second,
        first_members,
        second_members,
        ..
    } = error
    else {
        return;
    };
    let owners = [*first, *second];
    let members: Vec<_> = first_members
        .iter()
        .chain(second_members)
        .copied()
        .collect();
    let buildings: Vec<_> = layout
        .playable
        .iter()
        .cloned()
        .chain(
            layout
                .distant
                .iter()
                .copied()
                .map(TacticalBuildingPlacement::from),
        )
        .filter(|b| members.contains(&b.id))
        .collect();
    let mut contacts = Vec::new();
    let mut floor_points = Vec::new();
    let mut solids = Vec::new();
    for building in &buildings {
        let recipe =
            crate::scene_input::GeneratedBuildingRecipe::generate(building.program.clone())
                .unwrap();
        let origin = recipe.collision.bounds.centre().unwrap().metres().xz();
        let mut building_points = Vec::new();
        for solid in &recipe.collision.cuboids {
            let world: Vec<_> = solid
                .ground_contact()
                .unwrap()
                .points()
                .map(|p| {
                    building.centre_metres.metres()
                        + building.orientation.local_to_world(p.metres() - origin)
                })
                .collect();
            building_points.extend(world.iter().copied());
            if world.len() >= 3 {
                let rotation =
                    bevy::math::Quat::from_rotation_y(building.orientation.yaw_radians());
                let floor = surfaces
                    .iter()
                    .find(|p| p.member_building_ids().contains(&building.id))
                    .unwrap()
                    .mesh()
                    .elevations_at(
                        crate::scene_coordinates::ScenePlanPoint::from_metres(
                            building.centre_metres.metres(),
                        )
                        .unwrap(),
                    )
                    .iter()
                    .max_by(|a, b| a.metres().total_cmp(&b.metres()))
                    .unwrap()
                    .metres();
                let centre = rotation
                    * (solid.centre.metres() - bevy::math::Vec3::new(origin.x, 0.0, origin.y))
                    + bevy::math::Vec3::new(
                        building.centre_metres.metres().x,
                        floor,
                        building.centre_metres.metres().y,
                    );
                let world_solid = adventuresim_building_generator::CollisionCuboid {
                    centre:
                        adventuresim_building_generator::spatial_geometry::Position::from_metres(
                            centre,
                        )
                        .unwrap(),
                    yaw_radians: adventuresim_building_generator::spatial_geometry::Radians::new(
                        solid.yaw_radians.radians() + building.orientation.yaw_radians(),
                    )
                    .unwrap(),
                    ..*solid
                };
                solids.push((building.id, solid.source, world.clone(), world_solid));
            }
            for surface in surfaces.iter().filter(|p| {
                owners.contains(&p.property_id()) && !p.member_building_ids().contains(&building.id)
            }) {
                for region in surface.support_regions() {
                    let intersection = clip_to_region(&world, *region);
                    let area = intersection
                        .iter()
                        .zip(intersection.iter().cycle().skip(1))
                        .map(|(a, b)| a.as_dvec2().perp_dot(b.as_dvec2()))
                        .sum::<f64>()
                        .abs()
                        * 0.5;
                    if area > f64::EPSILON {
                        contacts.push(serde_json::json!({"building":building.id,"solid":solid.source,"owner":surface.property_id(),"region":region,"contact":world,"intersection":intersection,"area_square_metres":area}));
                    }
                }
            }
        }
        floor_points.push(serde_json::json!({"building":building.id,"points":building_points}));
    }
    let mut pairs = Vec::new();
    for (index, (first_building, first_solid, first_polygon, first_shape)) in
        solids.iter().enumerate()
    {
        for (second_building, second_solid, second_polygon, second_shape) in &solids[index + 1..] {
            if first_building == second_building {
                continue;
            }
            let intersection = clip_to_polygon(first_polygon, second_polygon);
            let area = intersection
                .iter()
                .zip(intersection.iter().cycle().skip(1))
                .map(|(a, b)| a.as_dvec2().perp_dot(b.as_dvec2()))
                .sum::<f64>()
                .abs()
                * 0.5;
            if area > f64::EPSILON {
                pairs.push(serde_json::json!({"first_building":first_building,"first_solid":first_solid,
                "second_building":second_building,"second_solid":second_solid,"first_contact":first_polygon,"second_contact":second_polygon,
                "intersection":intersection,"area_square_metres":area,"world_solids":[first_shape,second_shape],
                "fixed_solids_intersect_at_selected_floors":first_shape.intersects(*second_shape)}));
            }
        }
    }
    let value = serde_json::json!({"population":population,"seed":seed,"generation":crate::scene_input::TACTICAL_SCENE_GENERATION_VERSION,
        "error":error,"exact_contact_intersections":contacts,"floor_contact_points":floor_points,"actual_contact_pairs":pairs,"compounds":layout.compounds.iter().filter(|p|owners.contains(&p.id)).collect::<Vec<_>>(),
        "single_properties":layout.single_properties.iter().filter(|p|owners.contains(&p.id)).collect::<Vec<_>>(),
        "buildings":buildings,"streets":layout.streets,"source_triangles":source.triangles().collect::<Vec<_>>(),
        "support":surfaces.iter().filter(|p|owners.contains(&p.property_id())).map(|p|serde_json::json!({"owner":p.property_id(),"regions":p.support_regions(),"mesh":p.mesh()})).collect::<Vec<_>>()});
    let path = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join(format!("population-{population}-seed-{seed}.json")),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}

fn clip_to_region(points: &[Vec2], region: crate::city_layout::CityPlotBounds) -> Vec<Vec2> {
    clip_to_polygon(points, &region.corners())
}

fn clip_to_polygon(points: &[Vec2], outline: &[Vec2]) -> Vec<Vec2> {
    let mut polygon: Vec<_> = points.iter().map(|p| p.as_dvec2()).collect();
    let corners: Vec<_> = outline.iter().map(|p| p.as_dvec2()).collect();
    for i in 0..corners.len() {
        let origin = corners[i];
        let edge = corners[(i + 1) % corners.len()] - origin;
        let mut result = Vec::new();
        let Some(mut previous) = polygon.last().copied() else {
            return Vec::new();
        };
        let mut before = edge.perp_dot(previous - origin);
        for point in polygon {
            let after = edge.perp_dot(point - origin);
            if (before >= 0.0) != (after >= 0.0) {
                result.push(previous.lerp(point, before / (before - after)));
            }
            if after >= 0.0 {
                result.push(point);
            }
            previous = point;
            before = after;
        }
        polygon = result;
    }
    polygon.into_iter().map(|p| p.as_vec2()).collect()
}

pub(super) fn save_property(
    population: u32,
    seed: u64,
    layout: &CitySceneLayout,
    source: &GeographicSurface,
    error: &CitySupportError,
) {
    let Some(directory) = std::env::var_os("FABELGEIST_SUPPORT_DIAGNOSTIC_DIR") else {
        return;
    };
    let CitySupportError::Support(diagnostic) = error else {
        return;
    };
    let building = layout
        .playable
        .iter()
        .cloned()
        .chain(
            layout
                .distant
                .iter()
                .copied()
                .map(TacticalBuildingPlacement::from),
        )
        .find(|b| diagnostic.member_building_ids.contains(&b.id))
        .unwrap();
    let recipe =
        crate::scene_input::GeneratedBuildingRecipe::generate(building.program.clone()).unwrap();
    let origin = recipe.collision.bounds.centre().unwrap().metres().xz();
    let value = serde_json::json!({"population":population,"seed":seed,"failure":diagnostic,
        "building":building,"property":layout.single_properties.iter().find(|p|p.id==diagnostic.property_id),
        "footprint":recipe.collision.ground_floor_footprint().unwrap().unwrap().vertices().iter().map(|p|building.centre_metres.metres()+building.orientation.local_to_world(p.metres()-origin)).collect::<Vec<_>>(),
        "entrances":adventuresim_building_generator::compile_ground_entrances(&recipe.plan).unwrap().iter().map(|e|serde_json::json!({"id":e.id,"support":e.support,"threshold":building.centre_metres.metres()+building.orientation.local_to_world(e.threshold_metres.metres()-origin),"outward":building.orientation.local_to_world(e.outward.vector())})).collect::<Vec<_>>(),
        "streets":layout.streets.iter().filter(|s|s.contains(diagnostic.location_metres.attempted_metres())).collect::<Vec<_>>(),
        "source_triangles":source.triangles().collect::<Vec<_>>()});
    let path = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join(format!(
            "population-{population}-seed-{seed}-property-{}.json",
            diagnostic.property_id.0
        )),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
