//! Showing the hair and the scalp under it apart, to look at either alone:
//! the scalp's painted strands without the hair over them, or the hair
//! without the scalp filling its gaps. Only what the studio draws changes;
//! the recipe keeps its hair.
use super::*;

/// What a hair entity draws.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HairLayer {
    /// The strands or cards.
    Hair,
    /// The painted scalp cap under them.
    Scalp,
}

/// Which hair layers the studio draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShownLayers {
    pub(crate) hair: bool,
    pub(crate) scalp: bool,
}

impl Default for ShownLayers {
    fn default() -> Self {
        Self {
            hair: true,
            scalp: true,
        }
    }
}

impl ShownLayers {
    /// How an entity drawing `layer` is shown.
    pub(in crate::hair_preview) fn visibility(self, layer: HairLayer) -> Visibility {
        let shown = match layer {
            HairLayer::Hair => self.hair,
            HairLayer::Scalp => self.scalp,
        };
        if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    }
}

/// Show or hide each hair entity as the studio asks.
pub(crate) fn show(studio: Res<Studio>, mut layers: Query<(&HairLayer, &mut Visibility)>) {
    for (&layer, mut visibility) in &mut layers {
        visibility.set_if_neq(studio.hair_layers.visibility(layer));
    }
}
