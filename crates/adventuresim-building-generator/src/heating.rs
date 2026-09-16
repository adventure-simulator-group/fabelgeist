//! Room-owned domestic heating with physical smoke routes and weathering.
mod appliances;
mod audit;
pub(crate) use audit::audit;
mod assembly;
mod contact;
mod model;
mod partition;
mod placement;
mod roof;
mod weathering;
use crate::*;
pub use model::*;

pub(crate) fn resolve(
    program: &BuildingProgram,
    plan: &mut BuildingPlan,
) -> Result<(), GenerationError> {
    let Some(programme) = program.domestic_heating else {
        return Ok(());
    };
    let placement = placement::find(plan).ok_or(GenerationError::InvalidDomesticHeating)?;
    let wall = plan
        .wall_assemblies
        .iter()
        .find(|w| w.id == placement.wall)
        .unwrap();
    let owner = wall.owner;
    let face = plan
        .roof_assemblies
        .iter()
        .flat_map(|r| &r.faces)
        .find(|f| f.id == placement.face)
        .unwrap();
    let top = placement.flue_top(face);
    partition::cut(plan, placement);
    let mut assembly = assembly::Assembly {
        placement,
        geometry: &mut plan.resolved_geometry,
        plan: DomesticHeatingPlan {
            programme,
            owner,
            kitchen: HeatingRoom {
                storey_level: 0,
                room_id: placement.kitchen,
            },
            heated_room: HeatingRoom {
                storey_level: 0,
                room_id: placement.stube,
            },
            fire_wall: placement.wall,
            centre_metres: placement.centre,
            kitchen_axis: placement.kitchen_axis,
            floor_height_metres: 0.0,
            ground_support: StructuralNodeId(0),
            parts: vec![],
            passages: vec![],
            operating_space: placement.operating_space(),
            roof: HeatingRoofPenetration {
                roof: placement.roof,
                face: placement.face,
                cutout_index: 0,
                edges: vec![],
                flashing: vec![],
            },
        },
    };
    appliances::build(&mut assembly, top);
    roof::penetrate(&mut assembly, &mut plan.roof_assemblies);
    let wall = plan
        .wall_assemblies
        .iter_mut()
        .find(|w| w.id == placement.wall)
        .unwrap();
    wall.host_solids.extend(
        assembly
            .plan
            .parts
            .iter()
            .filter(|p| p.kind == HeatingPartKind::FireWall)
            .map(|p| p.solid),
    );
    plan.domestic_heating = Some(assembly.plan);

    Ok(())
}

#[cfg(test)]
mod tests;
