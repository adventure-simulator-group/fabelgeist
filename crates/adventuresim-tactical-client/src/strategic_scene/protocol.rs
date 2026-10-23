//! Presentation-only browser boundary. Rectangles are physical canvas pixels.
use crate::camera::viewport::CanvasRect;
use bevy::prelude::Resource;
use serde::Deserialize;

pub(crate) const FORGE_LAYER: usize = 1;
pub(crate) const COMPOSITOR_LAYER: usize = 2;
use crate::presentation::ownership::REGIONAL_MAP_LAYER;
pub(crate) const FIRST_PERSON_LAYER: usize = REGIONAL_MAP_LAYER + 1;

pub(crate) use crate::presentation::venues::{Place, PlaceId, PlaceKind, PortraitId};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Person {
    pub id: PortraitId,
    pub place: PlaceId,
    pub presentation: PersonPresentation,
    pub equipment: Vec<adventuresim_core::equipment_presentation::EquipmentAppearance>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PersonPresentation {
    Scene,
    Portrait,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PortraitView {
    pub id: PortraitId,
    pub rect: CanvasRect,
}

#[derive(Clone, Debug, Deserialize, Resource)]
pub(crate) struct StrategicView {
    pub revision: u64,
    pub location: String,
    pub places: Vec<Place>,
    pub people: Vec<Person>,
    pub active_place: Option<PlaceId>,
    pub selected: Option<PortraitId>,
    pub street: Option<CanvasRect>,
    pub stage: Option<CanvasRect>,
    pub forge: Option<CanvasRect>,
    pub portraits: Vec<PortraitView>,
}

impl StrategicView {
    pub(super) fn stage_subject(&self) -> Option<PortraitId> {
        self.selected.or_else(|| {
            self.people
                .iter()
                .find(|person| {
                    Some(&person.place) == self.active_place.as_ref()
                        && person.presentation == PersonPresentation::Scene
                })
                .map(|person| person.id)
        })
    }

    pub(super) fn requires_person(&self, id: PortraitId) -> bool {
        self.portraits.iter().any(|portrait| portrait.id == id)
            || (self.stage.is_some() && self.stage_subject().is_none_or(|subject| subject == id))
    }
}
