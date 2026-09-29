//! The studio's item catalog, with the catalog default designs being edited.
use adventuresim_character_creator::equipment_catalog::ItemCatalog;
use bevy::prelude::{Deref, DerefMut, Resource};

#[derive(Resource, Deref, DerefMut)]
pub(super) struct EquipmentCatalog(pub ItemCatalog);
