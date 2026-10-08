//! Checked metre geometry in the renderer-owned litter patch frame.
use adventuresim_building_generator::spatial_geometry::{
    Displacement, Elevation, GeometryFrame, PlanDirection, Position, PositiveLength,
};
use bevy::reflect::Reflect;

/// Patch-local X/Z on the ground plane, Y up; origin at the mesh instance anchor.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub(super) enum LitterPatch {}
impl GeometryFrame for LitterPatch {}

pub(super) struct RosetteLeaf {
    pub root: Position<LitterPatch>,
    pub direction: PlanDirection<LitterPatch>,
    pub length: PositiveLength,
    pub width: PositiveLength,
    pub rise: Elevation<LitterPatch>,
}

pub(super) struct CamberedLeaf {
    pub centre: Position<LitterPatch>,
    pub long: Displacement<LitterPatch>,
    pub side: Displacement<LitterPatch>,
    pub height: Elevation<LitterPatch>,
}

pub(super) struct TwigStation {
    pub centre: Position<LitterPatch>,
    pub radius: PositiveLength,
}

/// Authored radial profiles used by the litter catalogue.
#[derive(Clone, Copy)]
pub(super) enum TwigCrossSection {
    Pentagonal,
    Hexagonal,
}
impl TwigCrossSection {
    pub const fn sides(self) -> u32 {
        match self {
            Self::Pentagonal => 5,
            Self::Hexagonal => 6,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum TwigBase {
    Closed,
    ForkAttachment,
}

pub(super) struct BentTwig {
    pub stations: [TwigStation; 3],
    pub cross_section: TwigCrossSection,
    pub base: TwigBase,
    pub root: Position<LitterPatch>,
}
