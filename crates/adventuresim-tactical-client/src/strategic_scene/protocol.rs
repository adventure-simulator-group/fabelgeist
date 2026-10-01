//! Presentation-only browser boundary. Rectangles are physical canvas pixels.
use bevy::prelude::*;
use serde::Deserialize;

pub(crate) const FORGE_LAYER: usize = 1;
pub(crate) const COMPOSITOR_LAYER: usize = 2;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct PlaceId(pub String);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String")]
pub(crate) struct PortraitId(pub u64);

impl TryFrom<String> for PortraitId {
    type Error = std::num::ParseIntError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse().map(Self)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PlaceKind {
    Square,
    Residence,
    Keep,
    Market,
    Smith,
    Armor,
    Tailor,
    Apothecary,
    Books,
    Inn,
    Church,
    Guild,
    Camp,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct Place {
    pub id: PlaceId,
    pub kind: PlaceKind,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub(crate) struct CanvasRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub full_width: u32,
    pub full_height: u32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl CanvasRect {
    pub(super) fn uncropped(self) -> Self {
        Self {
            x: 0,
            y: 0,
            width: self.full_width,
            height: self.full_height,
            offset_x: 0.0,
            offset_y: 0.0,
            ..self
        }
    }

    pub fn apply(self, camera: &mut Camera, size: UVec2) {
        let origin = UVec2::new(self.x, self.y).min(size);
        let extent = UVec2::new(self.width, self.height).min(size - origin);
        camera.is_active = extent.min_element() > 0;
        if !camera.is_active {
            return;
        }
        camera.viewport = Some(bevy::camera::Viewport {
            physical_position: origin,
            physical_size: extent,
            ..default()
        });
        camera.sub_camera_view = Some(bevy::camera::SubCameraView {
            full_size: UVec2::new(self.full_width.max(1), self.full_height.max(1)),
            offset: Vec2::new(self.offset_x, self.offset_y),
            size: extent,
        });
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_identity_survives_javascript_integer_limit() {
        let id: PortraitId = serde_json::from_str("\"18446744073709551615\"").unwrap();
        assert_eq!(id.0, u64::MAX);
        assert!(serde_json::from_str::<PortraitId>("42").is_err());
        assert!(serde_json::from_str::<PortraitId>("\"someone\"").is_err());
    }

    #[test]
    fn offscreen_views_do_not_create_invalid_gpu_viewports() {
        let mut camera = Camera::default();
        let rect = CanvasRect {
            x: 900,
            y: 50,
            width: 80,
            height: 80,
            full_width: 80,
            full_height: 80,
            offset_x: 0.0,
            offset_y: 0.0,
        };
        rect.apply(&mut camera, UVec2::new(800, 600));
        assert!(!camera.is_active);
        CanvasRect { x: 760, ..rect }.apply(&mut camera, UVec2::new(800, 600));
        assert_eq!(camera.viewport.unwrap().physical_size, UVec2::new(40, 80));
    }
}
