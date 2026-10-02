//! Ground-cover categories and interaction markers.
use bevy::prelude::Component;
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GroundScatterLayer {
    Grass,
    Understory,
    BotanicalPlants,
    DryLeaves,
    Twigs,
    LooseStone,
}

#[derive(Component)]
pub(in crate::presentation) struct GroundScatterPresented;

/// Marks the locally controlled character whose movement bends nearby grass.
#[derive(Component)]
pub(crate) struct GrassInteractor;
