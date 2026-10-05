//! Exact occupied solids distinguish a bearing conflict from a broad envelope.
use super::*;
use bevy::math::Vec3Swizzles;

pub(super) fn failed_property(
    input: &TacticalSceneInput,
    layout: &CitySceneLayout,
    diagnostic: SupportDiagnostic,
) -> Result<Value, Box<dyn std::error::Error>> {
    let Some(property) = layout
        .single_properties
        .iter()
        .find(|p| p.id == diagnostic.property_id)
    else {
        return Ok(json!({"rejection":diagnostic}));
    };
    let placement = input
        .buildings
        .iter()
        .find(|b| b.id == property.building_id)
        .cloned()
        .or_else(|| {
            input
                .distant_buildings
                .iter()
                .find(|b| b.id == property.building_id)
                .copied()
                .map(TacticalBuildingPlacement::from)
        })
        .ok_or("diagnostic member absent")?;
    let recipe = GeneratedBuildingRecipe::generate(placement.program.clone())?;
    let origin = recipe.collision.bounds.centre()?.metres().xz();
    let contact = recipe
        .collision
        .ground_floor_contact_bounds()?
        .ok_or("missing bearing")?;
    let bearing = CityPlotBounds {
        centre_metres: placement.centre_metres
            + placement
                .orientation
                .local_to_world(contact.centre()?.metres().xz() - origin),
        dimensions_metres: contact.plan_half_extents()?.metres() * 2.0,
        orientation: placement.orientation,
    };
    let entries: Vec<_> = adventuresim_building_generator::compile_ground_entrances(&recipe.plan)?.into_iter().map(|entry| {
        json!({"id":entry.id,"support":entry.support,"threshold_m":placement.centre_metres+placement.orientation.local_to_world(entry.threshold_metres.metres()-origin),"outward":placement.orientation.local_to_world(entry.outward.vector())})
    }).collect();
    let point = diagnostic.location_metres;
    let mut streets: Vec<_> = layout
        .streets
        .iter()
        .map(|street| {
            let distance = match *street {
                CityStreetPatch::Corridor {
                    start_metres,
                    end_metres,
                    ..
                } => {
                    let delta = end_metres - start_metres;
                    let t = ((point - start_metres).dot(delta) / delta.length_squared())
                        .clamp(0.0, 1.0);
                    point.distance(start_metres + delta * t)
                }
                CityStreetPatch::Market { corners_metres, .. } => corners_metres
                    .into_iter()
                    .map(|p| point.distance(p))
                    .fold(f32::INFINITY, f32::min),
            };
            (distance, street)
        })
        .collect();
    streets.sort_by(|a, b| a.0.total_cmp(&b.0));
    let nearest: Vec<_> = streets
        .into_iter()
        .take(4)
        .map(|(distance, street)| json!({"distance_m":distance,"street":street}))
        .collect();
    let mut contacts = Vec::new();
    for solid in &recipe.collision.cuboids {
        let contact = solid.ground_contact()?;
        contacts.extend(contact.points().map(|point| {
            let world=placement.centre_metres+placement.orientation.local_to_world(point.metres()-origin);
            let local=property.plot.orientation.world_to_local(world-property.plot.centre_metres);
            let outside=(local.abs()-property.plot.dimensions_metres*0.5).max(Vec2::ZERO).length();
            let adjacent:Vec<_>=layout.single_properties.iter().filter(|p|p.id!=property.id&&p.plot.contains(world)).map(|p|p.id)
                .chain(layout.compounds.iter().filter(|p|p.plot.contains(world)).map(|p|p.id)).collect();
            json!({"solid_id":solid.source,"point_m":world,"outside_reservation_m":outside,"adjacent_reservations":adjacent})
        }));
    }
    let maximum = contacts
        .iter()
        .filter_map(|p| p["outside_reservation_m"].as_f64())
        .fold(0.0_f64, f64::max);
    Ok(
        json!({"rejection":diagnostic,"single_property":property,"bearing":bearing,"entrances":entries,"nearest_streets":nearest,"exact_contact_maximum_outside_m":maximum,"exact_ground_contacts":contacts}),
    )
}
