//! Window records retain frame, leaf role and opening identity during decoding.
use super::WindowLeafKind;
use crate::spatial_geometry::GeometryResult as Result;
use crate::spatial_geometry::{
    Architectural, CuboidDimensions, GeometryError, GeometryFrame, LeafDimensions, PlanDirection,
    Position, Radians,
};
use crate::{OpeningAssemblyId, ResolvedItemId};
use serde::{Deserialize, Serialize};

/// Fixed bars accompany the opening independently of its movable casement.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WindowBarPresence {
    Absent,
    Present,
}

/// One operable casement in the declared coordinate frame.
/// Geometry leaves are checked individually; generator compilation chooses the
/// inward swing.
///
/// ```compile_fail
/// use adventuresim_building_generator::{WindowSpec, spatial_geometry::Architectural};
/// use adventuresim_building_generator::furniture::FurnitureLocal;
/// fn wrong_frame(window: WindowSpec<Architectural>) -> WindowSpec<FurnitureLocal> { window }
/// ```
/// ```compile_fail
/// use adventuresim_building_generator::{WindowSpec, spatial_geometry::{Architectural, CuboidDimensions}};
/// fn wrong_role(window: &mut WindowSpec<Architectural>) { window.size_metres = CuboidDimensions::ZERO; }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(bound = "")]
pub struct WindowSpec<F: GeometryFrame> {
    pub leaf: WindowLeafKind,
    pub opening: OpeningAssemblyId,
    pub source: ResolvedItemId,
    pub closed_centre: Position<F>,
    pub hinge_centre: Position<F>,
    pub size_metres: LeafDimensions,
    pub closed_yaw_radians: Radians,
    pub tangent: PlanDirection<F>,
    pub outward: PlanDirection<F>,
    pub open_angle_radians: Radians,
    pub bars: WindowBarPresence,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct WindowBarSpec {
    pub opening: OpeningAssemblyId,
    pub source: ResolvedItemId,
    pub centre: Position<Architectural>,
    pub size_metres: CuboidDimensions,
    pub yaw_radians: Radians,
}

/// Window compilation and conversion preserve opening/source identities.
pub type WindowResult<T> = std::result::Result<T, WindowError>;

#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
#[error("window {opening:?}, source {source_id:?}: {cause}")]
pub struct WindowError {
    pub opening: OpeningAssemblyId,
    pub source_id: Option<ResolvedItemId>,
    #[source]
    pub cause: WindowErrorCause,
}
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
pub enum WindowErrorCause {
    #[error("operable window has no closure among {sources:?}")]
    MissingClosure { sources: Vec<ResolvedItemId> },
    #[error(transparent)]
    Geometry(#[from] GeometryError),
}

impl<'de, F: GeometryFrame> Deserialize<'de> for WindowSpec<F> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        // Wire vectors carry F-frame metre positions, leaf-local width/height/
        // thickness, normalized F-plan directions and yaw angles in radians.
        // Admit them together so geometry errors retain opening/source identity.
        struct NativeWindow {
            leaf: WindowLeafKind,
            opening: OpeningAssemblyId,
            source: ResolvedItemId,
            closed_centre: bevy::math::Vec3,
            hinge_centre: bevy::math::Vec3,
            size_metres: bevy::math::Vec3,
            closed_yaw_radians: f32,
            tangent: bevy::math::Vec2,
            outward: bevy::math::Vec2,
            open_angle_radians: f32,
            bars: WindowBarPresence,
        }
        let v = NativeWindow::deserialize(d)?;
        let admit = || -> Result<Self> {
            Ok(Self {
                leaf: v.leaf,
                opening: v.opening,
                source: v.source,
                closed_centre: Position::from_metres(v.closed_centre)?,
                hinge_centre: Position::from_metres(v.hinge_centre)?,
                size_metres: LeafDimensions::from_metres(v.size_metres)?,
                closed_yaw_radians: Radians::new(v.closed_yaw_radians)?,
                tangent: PlanDirection::from_normalized(v.tangent)?,
                outward: PlanDirection::from_normalized(v.outward)?,
                open_angle_radians: Radians::new(v.open_angle_radians)?,
                bars: v.bars,
            })
        };
        admit().map_err(|cause| {
            serde::de::Error::custom(WindowError {
                opening: v.opening,
                source_id: Some(v.source),
                cause: cause.into(),
            })
        })
    }
}
impl<'de> Deserialize<'de> for WindowBarSpec {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct NativeBar {
            opening: OpeningAssemblyId,
            source: ResolvedItemId,
            centre: bevy::math::Vec3,
            size_metres: bevy::math::Vec3,
            yaw_radians: f32,
        }
        let v = NativeBar::deserialize(d)?;
        let admit = || -> Result<Self> {
            Ok(Self {
                opening: v.opening,
                source: v.source,
                centre: Position::from_metres(v.centre)?,
                size_metres: CuboidDimensions::from_metres(v.size_metres)?,
                yaw_radians: Radians::new(v.yaw_radians)?,
            })
        };
        admit().map_err(|cause| {
            serde::de::Error::custom(WindowError {
                opening: v.opening,
                source_id: Some(v.source),
                cause: cause.into(),
            })
        })
    }
}
