//! Conservative ground-contact and fixed-post boundaries in one property frame.
//! These measurements select support topology; they do not certify a foundation.

use adventuresim_tactical_core::{
    city_layout::{CityCompound, CityPlotBounds},
    prelude::TacticalBuildingPlacement,
    scene_input::GeneratedBuildingRecipe,
};
use bevy::math::{Vec2, Vec3Swizzles};
use serde_json::{Value, json};
#[path = "support_regions/distance.rs"]
mod distance;

pub(super) fn describe(
    compound: &CityCompound,
    placement: &TacticalBuildingPlacement,
    recipe: &GeneratedBuildingRecipe,
) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    let Some(contact_region) = contact_region(placement, recipe)? else {
        return Ok(None);
    };
    let post = compound.boundary.gate.post(compound.boundary.gate.hinge)?;
    let post_region = CityPlotBounds::new(
        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            post.pose.plan_metres(),
        )?,
        adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
            post.size_metres.metres().xz(),
        )?,
        post.orientation,
    )?;
    let local_bounds = |region: CityPlotBounds| {
        let points = region.corners().map(|p| {
            compound
                .plot
                .orientation()
                .world_to_local(p - compound.plot.centre_metres())
        });
        (
            points
                .into_iter()
                .fold(Vec2::splat(f32::INFINITY), Vec2::min),
            points
                .into_iter()
                .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max),
        )
    };
    let (contact_min, contact_max) = local_bounds(contact_region);
    let (post_min, post_max) = local_bounds(post_region);
    let door = compound.boundary.gate.door(compound.id)?;
    let hinge_local = compound
        .plot
        .orientation()
        .world_to_local(door.hinge_centre.metres().xz() - compound.plot.centre_metres());
    let post_polygon = post_region.corners().map(|p| {
        compound
            .plot
            .orientation()
            .world_to_local(p - compound.plot.centre_metres())
    });
    let closest = nearest_post_contact(compound, placement, recipe, &post_polygon)?;
    Ok(Some(json!({
        "property_id":compound.id,
        "front_building_id":placement.id,
        "front_ground_contact_region":contact_region,
        "front_contact_bounds_in_property_frame_m":[contact_min,contact_max],
        "hinge_post_region":post_region,
        "hinge_post_bounds_in_property_frame_m":[post_min,post_max],
        "gate_hinge_in_property_frame_m":hinge_local,
        "projected_right_side_post_gap_m":post_min.x-contact_max.x,
        "projected_right_side_hinge_gap_m":hinge_local.x-contact_max.x,
        "post_and_contact_depth_intervals_overlap":post_min.y <= contact_max.y && contact_min.y <= post_max.y,
        "closest_actual_datum_contact_to_hinge_post":closest,
        "verification_scope":"Conservative projected envelopes in the property's orientation. Use actual bearing, retaining-solid and gate-sweep geometry for final support/collision acceptance. A hinge-line gap is not the clearance to its fixed post.",
    })))
}

pub(super) fn contact_region(
    placement: &TacticalBuildingPlacement,
    recipe: &GeneratedBuildingRecipe,
) -> Result<Option<CityPlotBounds>, Box<dyn std::error::Error>> {
    let Some(contact) = recipe.collision.ground_floor_contact_bounds()? else {
        return Ok(None);
    };
    Ok(Some(CityPlotBounds::new(
        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            placement.centre_metres.metres()
                + placement.orientation.local_to_world(
                    contact.centre()?.metres().xz()
                        - recipe.collision.bounds.centre()?.metres().xz(),
                ),
        )?,
        adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
            contact.plan_half_extents()?.metres() * 2.0,
        )?,
        placement.orientation,
    )?))
}

fn nearest_post_contact(
    compound: &CityCompound,
    placement: &TacticalBuildingPlacement,
    recipe: &GeneratedBuildingRecipe,
    post_polygon: &[Vec2],
) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    let origin = recipe.collision.bounds.centre()?.metres().xz();
    let contacts = recipe
        .collision
        .cuboids
        .iter()
        .map(|solid| {
            let polygon = solid
                .ground_contact()?
                .points()
                .map(|p| {
                    let world = placement.centre_metres.metres()
                        + placement.orientation.local_to_world(p.metres() - origin);
                    compound
                        .plot
                        .orientation()
                        .world_to_local(world - compound.plot.centre_metres())
                })
                .collect::<Vec<_>>();
            Ok(distance::between(&polygon, post_polygon).map(|gap| (solid.source, gap, polygon)))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let mut contacts: Vec<_> = contacts.into_iter().flatten().collect();
    contacts.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    Ok(contacts.first().map(|(source, gap, polygon)| {
        json!({
            "fixed_solid_id":source, "solid_role":recipe.plan.resolved_geometry.solids.iter().find(|s|s.id==*source).map(|s|s.role), "gap_m":gap,"contact_polygon_in_property_frame_m":polygon,
        })
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_tactical_core::prelude::DistantBuildingPlacement;

    #[test]
    fn goslar_1238_post_reaches_the_contact_envelope_despite_hinge_clearance() {
        let fixture: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/tactical-grounding/goslar-1238.json"
        )))
        .unwrap();
        let compound: CityCompound = serde_json::from_value(fixture["compound"].clone()).unwrap();
        let front: DistantBuildingPlacement =
            serde_json::from_value(fixture["front_distant_placement"].clone()).unwrap();
        let placement = TacticalBuildingPlacement::from(front);
        let recipe = GeneratedBuildingRecipe::generate(placement.program.clone()).unwrap();
        let report = describe(&compound, &placement, &recipe).unwrap().unwrap();
        assert_eq!(report["property_id"], 1238);
        assert_eq!(report["front_building_id"], 1238);
        assert_eq!(report["post_and_contact_depth_intervals_overlap"], true);
        assert!(
            report["projected_right_side_post_gap_m"]
                .as_f64()
                .unwrap()
                .abs()
                < 0.001
        );
        assert!(
            (0.29..0.31).contains(&report["projected_right_side_hinge_gap_m"].as_f64().unwrap())
        );
    }
}
