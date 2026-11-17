//! Environment-facing authority used by equipment actions.

use bevy::ecs::system::SystemParam;

#[derive(SystemParam)]
pub(super) struct EquipmentEnvironment<'w, 's> {
    pub(super) doors: crate::openings::DoorGrabber<'w, 's>,
    pub(super) windows: crate::openings::WindowGrabber<'w, 's>,
}
