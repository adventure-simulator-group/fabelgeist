//! What cloth, thread and beads are made of, and how each takes the light.
//!
//! One fabric carries many things: a velvet ground, a band of gold cloth,
//! black silk lines, wire rings, pearls. Each is named for what it is, and
//! the name fixes its roughness and how far it is metal. A recipe says what
//! a thing is made of; it does not set numbers that no material has.
use crate::TextileFinish;
use serde::{Deserialize, Serialize};

/// What a thread worked on a cloth is made of.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileThread {
    Linen,
    Wool,
    Silk,
    /// A strip of gilt metal wound on a silk core: the gold thread of cloth
    /// of gold and couched work. The silk shows between the turns.
    MetalThread,
    /// Solid drawn wire, as in chains, rings and spangles.
    MetalWire,
}

/// How a surface takes the light, in the terms of a metallic-roughness
/// material: both run from nought to one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextileSurface {
    pub roughness: f32,
    pub metallic: f32,
}

impl TextileSurface {
    /// Nacre: smooth and not a metal.
    pub const PEARL: Self = Self {
        roughness: 0.28,
        metallic: 0.,
    };

    /// The surface as one texel of a glTF metallic-roughness map: roughness
    /// in green, metalness in blue. Red is left for ambient visibility.
    pub(crate) fn texel(self) -> [u8; 4] {
        let byte = |share: f32| (share.clamp(0., 1.) * 255.).round() as u8;
        [255, byte(self.roughness), byte(self.metallic), 255]
    }

    /// This surface where `cover` of a texel is `over`, as a thread covers
    /// part of the cloth under it.
    pub(crate) fn covered(self, over: Self, cover: f32) -> Self {
        let cover = cover.clamp(0., 1.);
        Self {
            roughness: self.roughness + (over.roughness - self.roughness) * cover,
            metallic: self.metallic + (over.metallic - self.metallic) * cover,
        }
    }
}

impl TextileThread {
    pub fn surface(self) -> TextileSurface {
        match self {
            Self::Linen => TextileFinish::Linen.surface(),
            Self::Wool => TextileFinish::Wool.surface(),
            Self::Silk => TextileFinish::Silk.surface(),
            Self::MetalThread => TextileSurface {
                roughness: 0.5,
                metallic: 0.85,
            },
            Self::MetalWire => TextileSurface {
                roughness: 0.4,
                metallic: 1.,
            },
        }
    }
}

impl TextileFinish {
    /// The surface of cloth woven in this finish. Cloth of metallic thread
    /// is rougher and less wholly metal than one thread of it: its weave
    /// scatters the light and its silk shows between the threads.
    pub fn surface(self) -> TextileSurface {
        let (roughness, metallic) = match self {
            Self::Linen => (0.88, 0.),
            Self::Wool => (0.92, 0.),
            Self::Silk => (0.46, 0.),
            Self::Velvet => (0.78, 0.),
            Self::MetallicThread => (0.65, 0.8),
        };
        TextileSurface {
            roughness,
            metallic,
        }
    }
}
