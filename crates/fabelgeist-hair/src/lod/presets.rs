//! Authored screen-size and shading policies for the three render targets.

use super::{AccumulationFrames, Band, CardLevelIndex, Form, HairShading, LodPolicy};
use super::{PixelSpan, Quality, ShadowMapSize, ShadowQuality, SimBand, StrandLevelIndex};
use crate::sim::SimLevel;
use fabelgeist_math::Ratio;

impl LodPolicy {
    /// Every strand at every point, always.
    pub fn cinematic() -> Self {
        Self {
            name: "Cinematic",
            bands: vec![Band {
                min_pixels: PixelSpan::from_pixels(0.0),
                form: Form::Strands {
                    level: StrandLevelIndex::from_index(0),
                    share: Ratio::from_ratio(1.0),
                },
            }],
            quality: Quality {
                shading: HairShading::Marschner,
                shadows: ShadowQuality::DeepOpacity {
                    size: ShadowMapSize::from_texels(2048),
                },
                accumulate: AccumulationFrames::from_count(64),
                min_pixels: PixelSpan::from_pixels(0.5),
            },
            shadow_level: StrandLevelIndex::from_index(0),
            shadow_share: Ratio::from_ratio(1.0),
            blend: Ratio::from_ratio(0.0),
            sim: vec![SimBand {
                min_pixels: PixelSpan::from_pixels(0.0),
                level: SimLevel::HIGH,
            }],
        }
    }

    /// Strands close up, thinning with distance, cards far away. Every
    /// strand until the head is under 300 pixels tall -- a head in a
    /// medium shot at 1080p -- so what is coarser only shows where too
    /// little of it is on screen to tell.
    pub fn desktop() -> Self {
        Self {
            name: "Desktop",
            bands: vec![
                Band {
                    min_pixels: PixelSpan::from_pixels(300.0),
                    form: Form::Strands {
                        level: StrandLevelIndex::from_index(0),
                        share: Ratio::from_ratio(1.0),
                    },
                },
                Band {
                    min_pixels: PixelSpan::from_pixels(150.0),
                    form: Form::Strands {
                        level: StrandLevelIndex::from_index(1),
                        share: Ratio::from_ratio(0.5),
                    },
                },
                Band {
                    min_pixels: PixelSpan::from_pixels(70.0),
                    form: Form::Strands {
                        level: StrandLevelIndex::from_index(2),
                        share: Ratio::from_ratio(0.2),
                    },
                },
                Band {
                    min_pixels: PixelSpan::from_pixels(32.0),
                    form: Form::Cards {
                        level: CardLevelIndex::from_index(0),
                    },
                },
                Band {
                    min_pixels: PixelSpan::from_pixels(0.0),
                    form: Form::Cards {
                        level: CardLevelIndex::from_index(1),
                    },
                },
            ],
            quality: Quality {
                shading: HairShading::Marschner,
                shadows: ShadowQuality::DeepOpacity {
                    size: ShadowMapSize::from_texels(1024),
                },
                accumulate: AccumulationFrames::from_count(8),
                min_pixels: PixelSpan::from_pixels(1.0),
            },
            shadow_level: StrandLevelIndex::from_index(2),
            shadow_share: Ratio::from_ratio(0.25),
            blend: Ratio::from_ratio(0.4),
            sim: vec![
                SimBand {
                    min_pixels: PixelSpan::from_pixels(260.0),
                    level: SimLevel::HIGH,
                },
                SimBand {
                    min_pixels: PixelSpan::from_pixels(110.0),
                    level: SimLevel::MEDIUM,
                },
                SimBand {
                    min_pixels: PixelSpan::from_pixels(0.0),
                    level: SimLevel::LOW,
                },
            ],
        }
    }

    /// A fifth of the strands close up, a tenth further, cards far away;
    /// cheap shading and only baked shadowing.
    pub fn mobile() -> Self {
        Self {
            name: "Mobile",
            bands: vec![
                Band {
                    min_pixels: PixelSpan::from_pixels(300.0),
                    form: Form::Strands {
                        level: StrandLevelIndex::from_index(2),
                        share: Ratio::from_ratio(0.2),
                    },
                },
                Band {
                    min_pixels: PixelSpan::from_pixels(140.0),
                    form: Form::Strands {
                        level: StrandLevelIndex::from_index(3),
                        share: Ratio::from_ratio(0.1),
                    },
                },
                Band {
                    min_pixels: PixelSpan::from_pixels(55.0),
                    form: Form::Cards {
                        level: CardLevelIndex::from_index(0),
                    },
                },
                Band {
                    min_pixels: PixelSpan::from_pixels(0.0),
                    form: Form::Cards {
                        level: CardLevelIndex::from_index(1),
                    },
                },
            ],
            quality: Quality {
                shading: HairShading::KajiyaKay,
                shadows: ShadowQuality::Baked,
                accumulate: AccumulationFrames::from_count(1),
                min_pixels: PixelSpan::from_pixels(1.0),
            },
            shadow_level: StrandLevelIndex::from_index(3),
            shadow_share: Ratio::from_ratio(0.1),
            blend: Ratio::from_ratio(0.4),
            sim: vec![
                SimBand {
                    min_pixels: PixelSpan::from_pixels(220.0),
                    level: SimLevel::MEDIUM,
                },
                SimBand {
                    min_pixels: PixelSpan::from_pixels(0.0),
                    level: SimLevel::LOW,
                },
            ],
        }
    }
}
