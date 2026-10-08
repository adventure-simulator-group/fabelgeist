//! Physics construction failures retain the owning geometry and cause.
use super::*;
#[derive(Clone, Debug, thiserror::Error)]
pub enum SupportColliderError {
    #[error("property {property:?}, members {members:?}, surface collider: {cause}")]
    Surface {
        property: CityPropertyId,
        members: Vec<crate::scene_input::SceneBuildingId>,
        #[source]
        cause: avian3d::collision::collider::TrimeshBuilderError,
    },
    #[error("property {property:?}, members {members:?}, foundation cell {cell}: {issue}")]
    Foundation {
        property: CityPropertyId,
        members: Vec<crate::scene_input::SceneBuildingId>,
        cell: usize,
        #[source]
        issue: SupportGeometryIssue,
    },
    #[error(
        "property {property:?}, members {members:?}, boundary {element:?}, cell {cell}: {issue}"
    )]
    Boundary {
        property: CityPropertyId,
        members: Vec<crate::scene_input::SceneBuildingId>,
        element: enclosure::BoundarySupportElement,
        cell: usize,
        #[source]
        issue: SupportGeometryIssue,
    },
    #[error(transparent)]
    Geometry(#[from] SupportGeometryIssue),
}

/// Contact cells are admitted geometry without a three-dimensional solid.
/// Physics bodies are created only for the `Solid` case.
#[derive(Clone, Debug)]
pub enum SupportCollision {
    Solid(avian3d::prelude::Collider),
    ContactOnly,
}
impl SupportCollision {
    pub fn into_solid(self) -> Option<avian3d::prelude::Collider> {
        match self {
            Self::Solid(collider) => Some(collider),
            Self::ContactOnly => None,
        }
    }
    pub(super) fn compound(
        parts: Vec<(
            bevy::math::Vec3,
            bevy::math::Quat,
            avian3d::prelude::Collider,
        )>,
    ) -> Self {
        if parts.is_empty() {
            Self::ContactOnly
        } else {
            Self::Solid(avian3d::prelude::Collider::compound(parts))
        }
    }
}
