//! Physical two-dimensional stroke kernel, shared by textile color and relief.
use crate::{
    TextileColor, TextileFoldProfile, TextileNeedlework, TextileRepeatCount, TextileStroke,
    TextileStrokeStyle, TextileSurface,
    textile_stroke_path::{self, StrokeCopy},
};
use std::f32::consts::{FRAC_PI_2, TAU};

/// Height of a braid strand where it passes under another, as a share of the
/// height where it passes over.
const BRAID_UNDER_HEIGHT: f32 = 0.45;

/// Prepared chart-local metre segments. A repeated path that leaves its tile
/// continues into the next one, so repeats join without a seam.
pub(crate) struct NeedleworkTile {
    strokes: Vec<StrokeTile>,
}
struct StrokeTile {
    design: TextileStroke,
    size: [f32; 2],
    /// Tiles that span the region of the stroke along each axis.
    repeats: [f32; 2],
    segments: Vec<Segment>,
    length: f32,
    /// The tile itself and every repeat that reaches into it.
    copies: Vec<StrokeCopy>,
    /// Middle of the path, about which a varied repeat turns.
    middle: [f32; 2],
    /// Lowest and highest corner of the box around the path and its thread.
    bounds: [[f32; 2]; 2],
}
struct Segment {
    start: [f32; 2],
    delta: [f32; 2],
    length: f32,
    along: f32,
    /// Height of the thread's middle line at the start and the end.
    lift: [f32; 2],
}
pub(crate) struct NeedleworkSample {
    pub height: f32,
    pub color: Option<TextileColor>,
    /// How the thread that shows takes the light.
    pub surface: Option<TextileSurface>,
    pub coverage: f32,
}
struct StrokeSample {
    height: f32,
    color: Option<TextileColor>,
    surface: Option<TextileSurface>,
    coverage: f32,
}
/// The thread that shows at one sample: the highest of those that cover
/// most of it. Threads of equal height join, and the later one supplies
/// the color.
#[derive(Default)]
struct ThreadTop {
    height: f32,
    color: Option<TextileColor>,
    surface: Option<TextileSurface>,
    coverage: f32,
}
struct StrokeDistance {
    signed: f32,
    along: f32,
    /// Height of the thread's middle line at the nearest point of the path.
    lift: f32,
}
/// The braid strand that lies on top at one sampled point.
struct BraidStrand {
    radius: f32,
    radial: f32,
    /// Crown height share, lowered where the strand passes under another.
    lift: f32,
}
impl NeedleworkTile {
    pub fn new(design: &TextileNeedlework) -> Self {
        let strokes = design
            .strokes
            .iter()
            .map(|stroke| StrokeTile::new(design, stroke))
            .collect();
        Self { strokes }
    }
    pub fn sample(&self, chart: [f32; 2], footprint: f32) -> NeedleworkSample {
        let mut top = ThreadTop::default();
        let mut fold = 0_f32;
        for stroke in &self.strokes {
            let Some(local) = stroke.design.region.chart(chart) else {
                continue;
            };
            let tiles = [local[0] * stroke.repeats[0], local[1] * stroke.repeats[1]];
            let cell = tiles.map(f32::floor);
            for copy in &stroke.copies {
                let point = [
                    (tiles[0] - cell[0]) * stroke.size[0] - copy.offset[0],
                    (tiles[1] - cell[1]) * stroke.size[1] - copy.offset[1],
                ];
                let point = stroke.varied(point, cell, copy);
                if !stroke.reaches(point, footprint) {
                    continue;
                }
                let sample = stroke.sample(point, footprint);
                if sample.color.is_none() {
                    // A fold shapes the cloth itself and joins what lies on it.
                    fold = fold.max(sample.height);
                } else {
                    top.lay(&sample);
                }
            }
        }
        NeedleworkSample {
            height: top.height.max(fold),
            color: top.color,
            surface: top.surface,
            coverage: top.coverage,
        }
    }
}
impl ThreadTop {
    fn lay(&mut self, sample: &StrokeSample) {
        if sample.coverage <= 0. {
            return;
        }
        // A thread that only grazes the sample does not hide one that
        // covers it, however high it stands.
        let covers = sample.coverage >= 0.5;
        if self.color.is_none() || (covers && sample.height >= self.height) {
            self.color = sample.color;
            self.surface = sample.surface;
        }
        if covers || self.coverage < 0.5 {
            self.height = self.height.max(sample.height);
        }
        self.coverage = self.coverage.max(sample.coverage);
    }
}
impl StrokeTile {
    fn new(design: &TextileNeedlework, stroke: &TextileStroke) -> Self {
        let extent = design.extent(&stroke.region);
        let repeats = [f32::from(stroke.across.0), f32::from(stroke.along.0)];
        let size = [extent[0] / repeats[0], extent[1] / repeats[1]];
        // An axis without repeats has no neighbouring tile.
        let joins = repeats.map(|count| count > 1.);
        let points = textile_stroke_path::course(stroke, size);
        let lifts = textile_stroke_path::lifts(stroke);
        let middle = textile_stroke_path::middle(&points);
        let bounds = textile_stroke_path::bounds(&points, stroke.style.reach_metres());
        let copies = StrokeCopy::reaching(stroke, size, &points, joins);
        let mut length = 0.;
        let segments = points
            .windows(2)
            .zip(lifts.windows(2))
            .map(|(p, lift)| {
                let delta = [p[1][0] - p[0][0], p[1][1] - p[0][1]];
                let reach = delta[0].hypot(delta[1]);
                let segment = Segment {
                    start: p[0],
                    delta,
                    length: reach,
                    along: length,
                    lift: [lift[0], lift[1]],
                };
                length += reach;
                segment
            })
            .collect();
        Self {
            design: stroke.clone(),
            size,
            repeats,
            segments,
            length,
            copies,
            middle,
            bounds,
        }
    }
    /// Whether a sample of this footprint can touch the thread at all.
    fn reaches(&self, point: [f32; 2], footprint: f32) -> bool {
        (0..2).all(|axis| {
            point[axis] >= self.bounds[0][axis] - footprint
                && point[axis] <= self.bounds[1][axis] + footprint
        })
    }
    /// The sampled point carried back into the frame of the unvaried path,
    /// for the repeat that `copy` brings into the tile `cell`.
    fn varied(&self, point: [f32; 2], cell: [f32; 2], copy: &StrokeCopy) -> [f32; 2] {
        let Some(variation) = self.design.variation else {
            return point;
        };
        let tile = copy.tile(cell, [self.design.across, self.design.along]);
        let reach = self.size[0].min(self.size[1]) * variation.shift.unit();
        let turn = FRAC_PI_2
            * variation.turn.unit()
            * variation.offset(tile, textile_stroke_path::TURN_CHANNEL);
        let from_middle = [
            point[0]
                - self.middle[0]
                - reach * variation.offset(tile, textile_stroke_path::SHIFT_CHANNELS[0]),
            point[1]
                - self.middle[1]
                - reach * variation.offset(tile, textile_stroke_path::SHIFT_CHANNELS[1]),
        ];
        let (sin, cos) = (-turn).sin_cos();
        [
            self.middle[0] + from_middle[0] * cos - from_middle[1] * sin,
            self.middle[1] + from_middle[0] * sin + from_middle[1] * cos,
        ]
    }
    fn sample(&self, point: [f32; 2], footprint: f32) -> StrokeSample {
        let distance = self.nearest(point);
        let (radius, height, color, radial) = match &self.design.style {
            TextileStrokeStyle::Cord {
                diameter, color, ..
            } => {
                let radius = diameter.metres() * 0.5;
                (radius, radius, Some(*color), distance.signed.abs())
            }
            TextileStrokeStyle::Stitches {
                width,
                length,
                gap,
                relief,
                color,
                ..
            } => {
                let phase = distance.along.rem_euclid(length.metres() + gap.metres());
                let end_distance = (phase - length.metres())
                    .max(0.)
                    .min((length.metres() + gap.metres() - phase).max(0.));
                (
                    width.metres() * 0.5,
                    relief.metres(),
                    Some(*color),
                    distance.signed.hypot(end_distance),
                )
            }
            TextileStrokeStyle::Braid {
                strands,
                cover,
                width,
                pitch,
                relief,
                color,
                ..
            } => {
                let radius = width.metres() * cover.unit() / (f32::from(strands.0) * 2.);
                let strand =
                    self.braid(&distance, *strands, width.metres(), radius, pitch.metres());
                (
                    strand.radius,
                    relief.metres() * strand.lift,
                    Some(*color),
                    strand.radial,
                )
            }
            TextileStrokeStyle::Pearls {
                diameter,
                spacing,
                color,
            } => {
                let mut radial = f32::INFINITY;
                // Candidate beads adjacent to the projection on every segment
                // also handle corners; distance is to an actual path point.
                for segment in &self.segments {
                    let station = segment.project(point).along;
                    let first = (station / spacing.metres()).floor();
                    for ordinal in [first, first + 1.] {
                        let at = ordinal * spacing.metres();
                        if at >= 0. && at <= self.length {
                            let center = self.at(at);
                            radial = radial.min((point[0] - center[0]).hypot(point[1] - center[1]));
                        }
                    }
                }
                let radius = diameter.metres() * 0.5;
                (radius, radius, Some(*color), radial)
            }
            TextileStrokeStyle::Fold {
                width,
                relief,
                profile,
            } => {
                let radius = width.metres() * 0.5;
                let radial = match profile {
                    TextileFoldProfile::Rolled => distance.signed.abs(),
                    TextileFoldProfile::Felled => (distance.signed.abs() - radius * 0.5).abs() * 2.,
                };
                (radius, relief.metres(), None, radial)
            }
        };
        let coverage = ((radius - radial) / footprint + 0.5).clamp(0., 1.);
        let shape = (1. - (radial / radius).powi(2)).max(0.).sqrt();
        // The thread's crown stands its own height above its middle line.
        let lift = if color.is_some() { distance.lift } else { 0. };
        StrokeSample {
            height: lift + height * shape,
            color,
            surface: self.design.style.surface(),
            coverage,
        }
    }
    /// Strand `k` of `n` runs at `sin(theta)` across the braid, with `theta`
    /// offset by `k / n` of a turn. Two strands meet where their phases mirror
    /// about a quarter turn, so the odd level `sin((n - 1)(theta - pi/2))`
    /// is opposite for the pair and alternates along each strand. The pitch
    /// is adjusted to a whole number of turns so a repeated path joins itself.
    fn braid(
        &self,
        distance: &StrokeDistance,
        strands: TextileRepeatCount,
        width: f32,
        radius: f32,
        pitch: f32,
    ) -> BraidStrand {
        let count = f32::from(strands.0);
        let amplitude = width * 0.5 - radius;
        let rate = (self.length / pitch).round().max(1.) * TAU / self.length;
        let mut top = BraidStrand {
            radius,
            radial: f32::INFINITY,
            lift: 1.,
        };
        let mut top_height = 0.;
        for strand in 0..strands.0 {
            let theta = distance.along * rate + TAU * f32::from(strand) / count;
            // First-order distance to the curve from the offset across the path.
            let slope = amplitude * rate * theta.cos();
            let radial = (distance.signed - amplitude * theta.sin()).abs() / slope.hypot(1.);
            let level = ((count - 1.) * (theta - FRAC_PI_2)).sin();
            let lift = BRAID_UNDER_HEIGHT + (1. - BRAID_UNDER_HEIGHT) * (level + 1.) * 0.5;
            let height = lift * (1. - (radial / radius).powi(2)).max(0.).sqrt();
            if height > top_height || (top_height == 0. && radial < top.radial) {
                top_height = height;
                top = BraidStrand {
                    radius,
                    radial,
                    lift,
                };
            }
        }
        top
    }
    fn nearest(&self, point: [f32; 2]) -> StrokeDistance {
        self.segments
            .iter()
            .map(|s| s.project(point))
            .min_by(|a, b| a.signed.abs().total_cmp(&b.signed.abs()))
            .unwrap_or(StrokeDistance {
                signed: f32::INFINITY,
                along: 0.,
                lift: 0.,
            })
    }
    fn at(&self, distance: f32) -> [f32; 2] {
        let index = self
            .segments
            .partition_point(|s| s.along + s.length < distance)
            .min(self.segments.len() - 1);
        let segment = &self.segments[index];
        let t = ((distance - segment.along) / segment.length).clamp(0., 1.);
        [
            segment.start[0] + segment.delta[0] * t,
            segment.start[1] + segment.delta[1] * t,
        ]
    }
}
impl Segment {
    fn project(&self, point: [f32; 2]) -> StrokeDistance {
        let p = [point[0] - self.start[0], point[1] - self.start[1]];
        let t = ((p[0] * self.delta[0] + p[1] * self.delta[1]) / (self.length * self.length))
            .clamp(0., 1.);
        let vector = [p[0] - self.delta[0] * t, p[1] - self.delta[1] * t];
        let sign = (self.delta[0] * p[1] - self.delta[1] * p[0]).signum();
        // Endcaps use Euclidean distance too; signum(0) must not erase it.
        StrokeDistance {
            signed: vector[0].hypot(vector[1]) * if sign == 0. { 1. } else { sign },
            along: self.along + t * self.length,
            lift: self.lift[0] + (self.lift[1] - self.lift[0]) * t,
        }
    }
}
