//! Missing physical authorities are construction failures with their source identity.
use crate::{HeatingPartKind, ResolvedItemId, RoofAssemblyId, StoreyIndex, WallAssemblyId};

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum HeatingConstructionError {
    #[error("resolved heating programme is absent")]
    MissingProgramme,
    #[error("heating passage {kind:?} is absent")]
    MissingPassage { kind: crate::HeatingPassageKind },
    #[error("heating void {void:?} is absent")]
    MissingVoid { void: ResolvedItemId },
    #[error("heating wall {wall:?} is absent")]
    MissingWall { wall: WallAssemblyId },
    #[error("heating roof face {face:?} is absent")]
    MissingFace { face: ResolvedItemId },
    #[error("heating roof {roof:?} is absent")]
    MissingRoof { roof: RoofAssemblyId },
    #[error("heating floor on storey {storey} is absent")]
    MissingFloor { storey: StoreyIndex },
    #[error("heating solid {solid:?} is absent")]
    MissingSolid { solid: ResolvedItemId },
    #[error("heating solid {solid:?} has no bearing node")]
    MissingBearing { solid: ResolvedItemId },
    #[error("heating part {kind:?} is absent")]
    MissingPart { kind: HeatingPartKind },
    #[error("heating cut left no deck pieces on storey {storey}")]
    MissingDeck { storey: StoreyIndex },
}
