//! Bound raster detail per motif in each logical cloth-chart direction.
use crate::{
    TextileAppearance, TextileAxis, TextileFill, TextileMotif, TextilePatternSpace, TextileRegion,
};

const MIN_TEXTURE_SIDE: u32 = 256;
const MAX_TEXTURE_SIDE: u32 = 4096;
const PIXELS_PER_REPEAT: f32 = 128.;
const MAX_TEXTURE_PIXELS: u32 = 2_097_152;
const MIN_FEATURE_PIXELS: f32 = 2.;
const SAMPLES_PER_THREAD_PERIOD: f32 = 32.;

pub(crate) struct TextileRasterSize {
    pub width: u32,
    pub height: u32,
}

impl TextileRasterSize {
    pub fn for_appearance(fabric: &TextileAppearance) -> Self {
        let mut repeats = [1_f32; 2];
        if let Some(relief) = &fabric.relief {
            repeats = relief_density(relief);
        }
        for layer in &fabric.layers.0 {
            let span = match layer.region {
                TextileRegion::Whole => [1., 1.],
                TextileRegion::Band { axis, start, end } => {
                    let mut span = [1., 1.];
                    span[axis_index(axis)] = end.unit() - start.unit();
                    span
                }
                TextileRegion::Panel { across, along } => [
                    across.end.unit() - across.start.unit(),
                    along.end.unit() - along.start.unit(),
                ],
            };
            let density = match layer.fill {
                TextileFill::Solid => [1., 1.],
                TextileFill::Weave { warp, weft, width } => {
                    let detail = (MIN_FEATURE_PIXELS / (width.unit() * PIXELS_PER_REPEAT)).max(1.);
                    [f32::from(warp.0) * detail, f32::from(weft.0) * detail]
                }
                TextileFill::Stripes {
                    axis,
                    repeats,
                    width,
                    ..
                } => {
                    let mut density = [1., 1.];
                    let index = axis_index(axis);
                    if width.0 < 1000 {
                        let feature = width.unit().min(1. - width.unit());
                        density[index] = f32::from(repeats.0) / span[index]
                            * (MIN_FEATURE_PIXELS / (feature * PIXELS_PER_REPEAT)).max(1.);
                    }
                    density
                }
                TextileFill::Motifs {
                    space,
                    ref motif,
                    across,
                    along,
                    scale,
                    ..
                } => {
                    let span = if space == TextilePatternSpace::Region {
                        span
                    } else {
                        [1., 1.]
                    };
                    let detail = match motif {
                        TextileMotif::Medallion {
                            threads, stroke, ..
                        } => {
                            (f32::from(threads.0) * SAMPLES_PER_THREAD_PERIOD)
                                .max(MIN_FEATURE_PIXELS / stroke.unit())
                                / PIXELS_PER_REPEAT
                        }
                        _ => 1.,
                    };
                    [
                        f32::from(across.0) * detail / (span[0] * scale.unit()),
                        f32::from(along.0) * detail / (span[1] * scale.unit()),
                    ]
                }
            };
            for axis in 0..2 {
                repeats[axis] = repeats[axis]
                    .max(density[axis])
                    .max(MIN_FEATURE_PIXELS / (span[axis] * PIXELS_PER_REPEAT));
            }
        }
        if let Some(needlework) = &fabric.needlework {
            let density = needlework_density(needlework);
            for axis in 0..2 {
                repeats[axis] = repeats[axis].max(density[axis]);
            }
        }
        let requested = repeats.map(|repeat| {
            (repeat * PIXELS_PER_REPEAT)
                .max(MIN_TEXTURE_SIDE as f32)
                .log2()
                .ceil()
                .exp2()
        });
        // Scale both axes together before applying the minimum side. Independent
        // per-axis caps would destroy a narrow trim's sampling aspect ratio.
        let scale = (MAX_TEXTURE_SIDE as f32 / requested[0].max(requested[1]))
            .min((MAX_TEXTURE_PIXELS as f32 / (requested[0] * requested[1])).sqrt())
            .min(1.);
        let sides = requested.map(|side| ((side * scale).floor() as u32).max(MIN_TEXTURE_SIDE));
        Self {
            width: sides[0],
            height: sides[1],
        }
    }
}

/// Native raster adapter: repeat density on each normalized cloth-chart axis.
fn relief_density(relief: &crate::TextilePleatRelief) -> [f32; 2] {
    let mut repeats = [1_f32; 2];
    let span = match relief.layout {
        crate::TextilePleatLayout::FreeEdge | crate::TextilePleatLayout::Uniform => 1.,
        crate::TextilePleatLayout::Waist { front_panel, .. } => 1. - front_panel.unit(),
        crate::TextilePleatLayout::Shirred(crate::TextileShirring { band, rows, .. }) => {
            repeats[1] = (f32::from(rows.0) * 16.
                / ((band.end.unit() - band.start.unit()) * PIXELS_PER_REPEAT))
                .max(1.);
            1.
        }
    };
    repeats[0] = f32::from(relief.pleats.count.0) * 16. / (span * PIXELS_PER_REPEAT);
    repeats
}

fn axis_index(axis: TextileAxis) -> usize {
    match axis {
        TextileAxis::Across => 0,
        TextileAxis::Along => 1,
    }
}

/// Native physical chart dimensions and thinnest stroke request raster detail.
fn needlework_density(needlework: &crate::TextileNeedlework) -> [f32; 2] {
    let feature = needlework
        .strokes
        .iter()
        .map(|s| s.style.feature_metres())
        .fold(f32::INFINITY, f32::min);
    [
        needlework.chart.across.metres(),
        needlework.chart.along.metres(),
    ]
    .map(|size| size / feature * MIN_FEATURE_PIXELS / PIXELS_PER_REPEAT)
}
