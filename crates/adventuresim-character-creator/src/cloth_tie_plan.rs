//! Stable tape topology: attachment bands, symmetric loops, straight tails, knot.
use fabelgeist_armor::gpu::BodyClothLayer;
use fabelgeist_armor::{ArmorComponentRole, ClothTies};

const TIE_CURVE_SEGMENTS: u32 = 24;
const TIE_TAIL_SEGMENTS: u32 = 4;
pub(crate) const TIE_POINT_WORDS: u32 = 5;

/// Native source-point ordinals in the fitted cloth table.
pub(crate) struct ClothTieAnchors {
    pub front: u32,
    pub back: u32,
}

pub(crate) struct ClothTiePlan {
    pub first: u32,
    pub words: Vec<u32>,
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub(crate) enum TiePath {
    FrontBand,
    BackBand,
    LeftLoop,
    RightLoop,
    LeftTail,
    RightTail,
    Knot,
}

impl ClothTiePlan {
    pub fn new(
        anchors: &[ClothTieAnchors],
        design: &ClothTies,
        layers: &mut Vec<BodyClothLayer>,
    ) -> Self {
        let first = layers.iter().map(|l| l.point_count).sum();
        let mut words = Vec::new();
        let mut indices = Vec::new();
        for anchor in anchors {
            for path in [
                TiePath::FrontBand,
                TiePath::BackBand,
                TiePath::LeftLoop,
                TiePath::RightLoop,
                TiePath::LeftTail,
                TiePath::RightTail,
                TiePath::Knot,
            ] {
                let closed = matches!(path, TiePath::Knot);
                let segments = if matches!(path, TiePath::LeftTail | TiePath::RightTail) {
                    TIE_TAIL_SEGMENTS
                } else {
                    TIE_CURVE_SEGMENTS
                };
                let stations = segments + u32::from(!closed);
                let offset = words.len() as u32 / TIE_POINT_WORDS;
                for station in 0..stations {
                    let along = station as f32 / segments as f32;
                    for across in [-0.5_f32, 0.5_f32] {
                        words.extend([
                            anchor.front,
                            anchor.back,
                            path as u32,
                            along.to_bits(),
                            across.to_bits(),
                        ]);
                    }
                }
                for station in 0..segments {
                    let a = offset + station * 2;
                    let b = offset + ((station + 1) % stations) * 2;
                    indices.extend([a, b, a + 1, a + 1, b, b + 1]);
                }
            }
        }
        layers.push(BodyClothLayer {
            point_count: words.len() as u32 / TIE_POINT_WORDS,
            indices,
            role: ArmorComponentRole::ClothTies,
            offset_metres: design.thickness.metres(),
            thickness_metres: design.thickness.metres(),
        });
        Self { first, words }
    }
}
