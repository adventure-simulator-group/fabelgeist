//! Select the actual covering above an unobstructed vertical smoke shaft.
use super::placement::Placement;
use crate::{BuildingPlan, RoofAssemblyId, RoofFace};
use bevy::math::{Vec2, Vec3};
use geo::{Area, BooleanOps, Intersects};

const OVERLAP_AREA_TOLERANCE: f32 = 0.00001;

pub(super) fn find(
    plan: &BuildingPlan,
    placement: Placement,
) -> Option<(RoofAssemblyId, &RoofFace)> {
    let probe = placement.centre + placement.kitchen_axis * placement.section.shaft_offset();
    plan.roof_assemblies.iter().find_map(|roof| {
        roof.faces
            .iter()
            .find(|face| fits(face, probe) && clear_other_roofs(plan, placement, face))
            .map(|face| (roof.id, face))
    })
}

fn ring(points: &[Vec3], offset: Vec3) -> geo::LineString<f32> {
    geo::LineString::new(
        points
            .iter()
            .map(|p| geo::Coord {
                x: p.x + offset.x,
                y: p.z + offset.z,
            })
            .collect(),
    )
}

fn footprint(min: Vec2, max: Vec2) -> geo::Polygon<f32> {
    geo::Rect::new(geo::coord! {x:min.x,y:min.y}, geo::coord! {x:max.x,y:max.y}).to_polygon()
}

fn fits(face: &RoofFace, probe: Vec2) -> bool {
    let weather = super::roof::PenetrationFootprint::new(face, probe);
    let rect = footprint(weather.weather_min, weather.weather_max);
    rect.difference(&geo::Polygon::new(ring(&face.polygon, Vec3::ZERO), vec![]))
        .unsigned_area()
        < OVERLAP_AREA_TOLERANCE
        && !face
            .cutouts
            .iter()
            .any(|cut| rect.intersects(&geo::Polygon::new(ring(cut, Vec3::ZERO), vec![])))
}

fn clear_other_roofs(plan: &BuildingPlan, placement: Placement, target: &RoofFace) -> bool {
    let shaft = placement.shaft(placement.flue_top(target));
    let rect = footprint(
        Vec2::new(shaft.min.x, shaft.min.z),
        Vec2::new(shaft.max.x, shaft.max.z),
    );
    for roof in &plan.roof_assemblies {
        for face in roof.faces.iter().filter(|face| face.id != target.id) {
            for offset in [
                Vec3::ZERO,
                -face.plane.normal.normalize() * face.thickness_metres,
            ] {
                let solid = geo::Polygon::new(
                    ring(&face.polygon, offset),
                    face.cutouts.iter().map(|cut| ring(cut, offset)).collect(),
                );
                if rect.intersection(&solid).unsigned_area() > OVERLAP_AREA_TOLERANCE {
                    return false;
                }
            }
        }
        for cheek in &roof.enclosure_faces {
            if cheek.polygon.iter().all(|p| p.y > shaft.max.y)
                || cheek.polygon.iter().all(|p| p.y < shaft.min.y)
            {
                continue;
            }
            let margin = Vec2::splat(
                crate::ROOF_ENCLOSURE_THICKNESS_METRES + super::placement::TIMBER_CLEARANCE_METRES,
            );
            let enclosure_clearance = footprint(
                Vec2::new(shaft.min.x, shaft.min.z) - margin,
                Vec2::new(shaft.max.x, shaft.max.z) + margin,
            );
            if ring(&cheek.polygon, Vec3::ZERO).intersects(&enclosure_clearance) {
                return false;
            }
        }
    }
    true
}

pub(super) fn weather_clear(plan: &BuildingPlan, placement: Placement, face: &RoofFace) -> bool {
    let mut roofs = vec![
        plan.roof_assemblies
            .iter()
            .find(|roof| roof.id == placement.roof)
            .unwrap()
            .clone(),
    ];
    let mut geometry = crate::ResolvedGeometry::default();
    let owner = plan
        .wall_assemblies
        .iter()
        .find(|wall| wall.id == placement.wall)
        .unwrap()
        .owner;
    let mut assembly = super::assembly::Assembly::new(
        &mut geometry,
        placement,
        owner,
        crate::DomesticHeatingProgramme::HearthAndRearFedStove,
    );
    super::roof::penetrate(&mut assembly, &mut roofs);
    !super::audit::roof_route::obstructed(
        plan,
        face.id,
        placement.shaft(placement.flue_top(face)),
        &geometry.solids.iter().collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_flue_under_a_shed_uses_its_covering_and_requires_the_parent_cut() {
        let mut programme = crate::BuildingProgram::fixture(crate::BuildingArchetype::HallHouse, 2);
        programme.domestic_heating = None;
        let mut plan = crate::generate(&programme).unwrap();
        let placement =
            super::super::placement::find(&plan).expect("grounded route remains available");
        let roof = plan
            .roof_assemblies
            .iter()
            .find(|roof| roof.id == placement.roof)
            .unwrap();
        let parent = roof.parent.expect("this route exits the shed covering");
        assert!(find(&plan, placement).is_some());
        for face in &mut plan
            .roof_assemblies
            .iter_mut()
            .find(|roof| roof.id == parent)
            .unwrap()
            .faces
        {
            face.cutouts.clear();
        }
        assert!(find(&plan, placement).is_none());
    }
}
