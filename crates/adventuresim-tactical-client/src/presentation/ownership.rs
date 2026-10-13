//! Actor scenes and focused map cities retain independent presentation assets.
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

pub(crate) const REGIONAL_MAP_LAYER: usize = 3;

/// Fixed owner slots bound residency to one actor scene and one map city.
#[derive(Clone, Default)]
pub(crate) struct PresentationOwners<T> {
    scene: T,
    regional_map: T,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PresentationOwner {
    Scene,
    RegionalMap,
}

impl<T> PresentationOwners<T> {
    pub(crate) const fn new(scene: T, regional_map: T) -> Self {
        Self {
            scene,
            regional_map,
        }
    }

    pub(crate) fn get(&self, owner: PresentationOwner) -> &T {
        match owner {
            PresentationOwner::Scene => &self.scene,
            PresentationOwner::RegionalMap => &self.regional_map,
        }
    }

    pub(crate) fn get_mut(&mut self, owner: PresentationOwner) -> &mut T {
        match owner {
            PresentationOwner::Scene => &mut self.scene,
            PresentationOwner::RegionalMap => &mut self.regional_map,
        }
    }
}

impl PresentationOwner {
    pub(crate) const ALL: [Self; 2] = [Self::Scene, Self::RegionalMap];

    pub(crate) fn render_layers(self) -> RenderLayers {
        match self {
            Self::Scene => RenderLayers::default(),
            Self::RegionalMap => RenderLayers::layer(REGIONAL_MAP_LAYER),
        }
    }
}
