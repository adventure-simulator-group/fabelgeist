//! Room-owned domestic heating with physical smoke routes and weathering.
use crate::GenerationResult as Result;
mod appliances;
mod error;
pub use error::HeatingConstructionError;
mod audit;
pub(crate) use audit::audit;
mod assembly;
mod contact;
mod floor_bearings;
mod floors;
mod model;
mod partition;
mod placement;
mod roof;
mod roof_route;
mod weather_sections;
mod weathering;
use crate::*;
pub use model::*;

pub(crate) fn resolve(program: &BuildingProgram, plan: &mut BuildingPlan) -> Result<()> {
    let Some(programme) = program.domestic_heating else {
        return Ok(());
    };
    let placement = placement::find(plan)?.ok_or(GenerationError::InvalidDomesticHeating)?;
    let wall = plan
        .wall_assemblies
        .iter()
        .find(|w| w.id == placement.site.wall)
        .ok_or(HeatingConstructionError::MissingWall {
            wall: placement.site.wall,
        })?;
    let owner = wall.owner;
    let face = plan
        .roof_assemblies
        .iter()
        .flat_map(|r| &r.faces)
        .find(|f| f.id == placement.face)
        .ok_or(HeatingConstructionError::MissingFace {
            face: placement.face,
        })?;
    let top = placement.site.flue_top(face)?;
    partition::cut(plan, placement)?;
    let floors = floors::cut(plan, placement)?;
    let mut assembly =
        assembly::Assembly::new(&mut plan.resolved_geometry, placement, owner, programme)?;
    appliances::build(&mut assembly, top)?;
    floors::close(&mut assembly, floors)?;
    roof::penetrate(&mut assembly, &mut plan.roof_assemblies)?;
    let wall = plan
        .wall_assemblies
        .iter_mut()
        .find(|w| w.id == placement.site.wall)
        .ok_or(HeatingConstructionError::MissingWall {
            wall: placement.site.wall,
        })?;
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

#[cfg(test)]
mod upper_tests;
