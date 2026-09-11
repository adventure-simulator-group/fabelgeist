//! Modest parish churches and chapels share canonical walls, apertures and roofs.
use super::*;
use crate::ServiceBuildingSize;
use crate::*;
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod appearance_tests;
mod fittings;
mod programme;
mod storey;
#[cfg(test)]
mod tests;
mod validation;

pub(crate) use validation::audit_small_church;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SmallChurchKind {
    Chapel,
    Parish,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmallChurchPlan {
    pub kind: SmallChurchKind,
    pub size: ServiceBuildingSize,
    pub nave_bays: u16,
    pub nave_depth_metres: f32,
    pub chancel_eave_metres: Option<f32>,
    pub belfry_stage: ResolvedBounds,
    pub fittings: Vec<ResolvedItemId>,
    pub bearing_walls: Vec<crate::WallAssemblyId>,
    pub public_route: ResolvedBounds,
}

impl SmallChurchKind {
    pub const fn from_use(usage: BuildingUse) -> Option<Self> {
        match usage {
            BuildingUse::Chapel => Some(Self::Chapel),
            BuildingUse::ParishChurch => Some(Self::Parish),
            _ => None,
        }
    }
}

pub(super) fn occupied_storey(program: &BuildingProgram) -> Option<StoreyPlan> {
    Some(storey::build(Dimensions::from_program(program)?))
}

pub(super) fn wall_height(program: &BuildingProgram, wall: crate::WallSegment) -> f32 {
    let Some(dimensions) = Dimensions::from_program(program) else {
        return program.storey_height_metres;
    };
    if wall.centre().y > dimensions.nave_depth() {
        dimensions.chancel_eave
    } else {
        dimensions.nave_eave
    }
}

pub(super) fn roofs(program: &BuildingProgram) -> Option<Vec<RoofPiece>> {
    Some(Dimensions::from_program(program)?.roofs(program.roof_pitch_degrees))
}

pub(super) fn belfry_enclosure_top(program: &BuildingProgram, index: usize) -> Option<f32> {
    let d = Dimensions::from_program(program)?;
    (index == if d.chancel_cells > 0 { 2 } else { 1 })
        .then(|| d.bell_floor(program.roof_pitch_degrees))
}

pub(super) fn resolve(
    program: &BuildingProgram,
    walls: &mut Vec<crate::WallAssembly>,
    geometry: &mut ResolvedGeometry,
) -> Option<SmallChurchPlan> {
    let dimensions = Dimensions::from_program(program)?;
    Some(fittings::assemble(
        dimensions,
        program.roof_pitch_degrees,
        walls,
        geometry,
    ))
}

#[derive(Clone, Copy)]
struct Dimensions {
    kind: SmallChurchKind,
    size: ServiceBuildingSize,
    width_cells: u16,
    nave_cells: u16,
    chancel_cells: u16,
    nave_eave: f32,
    chancel_eave: f32,
}

impl Dimensions {
    fn from_program(program: &BuildingProgram) -> Option<Self> {
        if program.archetype != BuildingArchetype::ParishChurch {
            return None;
        }
        let kind = SmallChurchKind::from_use(program.usage?)?;
        let size = program.service_size?;
        let extra = match size {
            ServiceBuildingSize::Small => 0,
            ServiceBuildingSize::Medium => 2,
            ServiceBuildingSize::Large => 4,
        };
        Some(match kind {
            SmallChurchKind::Chapel => Self {
                kind,
                size,
                width_cells: 5,
                nave_cells: 6 + extra,
                chancel_cells: 0,
                nave_eave: 3.9,
                chancel_eave: 3.9,
            },
            SmallChurchKind::Parish => Self {
                kind,
                size,
                width_cells: 7,
                nave_cells: 8 + extra,
                chancel_cells: 4,
                nave_eave: 5.1,
                chancel_eave: 4.2,
            },
        })
    }

    fn width(self) -> f32 {
        f32::from(self.width_cells) * CELL_SIZE_METRES
    }
    fn nave_depth(self) -> f32 {
        f32::from(self.nave_cells) * CELL_SIZE_METRES
    }
    fn depth(self) -> f32 {
        f32::from(self.nave_cells + self.chancel_cells) * CELL_SIZE_METRES
    }
    fn bell_centre(self) -> Vec2 {
        Vec2::new(self.width() * 0.5, 2.25)
    }
    fn bell_floor(self, pitch: f32) -> f32 {
        self.nave_eave + self.width() * 0.5 * pitch.to_radians().tan() + 0.25
    }
}
