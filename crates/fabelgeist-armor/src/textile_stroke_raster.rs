//! Physical two-dimensional stroke kernel, shared by textile color and relief.
use crate::{
    TextileAxis, TextileColor, TextileFoldProfile, TextileNeedlework, TextilePathClosure,
    TextileRegion, TextileStroke, TextileStrokeStyle,
};
use std::f32::consts::TAU;

/// Prepared chart-local metre segments; repeated paths are clipped to a region.
pub(crate) struct NeedleworkTile {
    strokes: Vec<StrokeTile>,
}
struct StrokeTile {
    design: TextileStroke,
    size: [f32; 2],
    segments: Vec<Segment>,
    length: f32,
}
struct Segment {
    start: [f32; 2],
    delta: [f32; 2],
    length: f32,
    along: f32,
}
pub(crate) struct NeedleworkSample {
    pub height: f32,
    pub color: Option<TextileColor>,
    pub coverage: f32,
}
struct StrokeDistance {
    signed: f32,
    along: f32,
}
impl NeedleworkTile {
    pub fn new(design: &TextileNeedlework) -> Self {
        Self {
            strokes: design
                .strokes
                .iter()
                .map(|stroke| {
                    let span = match stroke.region {
                        TextileRegion::Whole => [1., 1.],
                        TextileRegion::Band { axis, start, end } => {
                            let mut span = [1., 1.];
                            span[usize::from(axis == TextileAxis::Along)] =
                                end.unit() - start.unit();
                            span
                        }
                        TextileRegion::Panel { across, along } => [
                            across.end.unit() - across.start.unit(),
                            along.end.unit() - along.start.unit(),
                        ],
                    };
                    let size = [
                        design.chart.across.metres() * span[0] / f32::from(stroke.across.0),
                        design.chart.along.metres() * span[1] / f32::from(stroke.along.0),
                    ];
                    let mut points = stroke
                        .points
                        .iter()
                        .map(|p| [p.across.unit() * size[0], p.along.unit() * size[1]])
                        .collect::<Vec<_>>();
                    if stroke.closure == TextilePathClosure::Closed {
                        points.push(points[0]);
                    }
                    let mut length = 0.;
                    let segments = points
                        .windows(2)
                        .map(|p| {
                            let delta = [p[1][0] - p[0][0], p[1][1] - p[0][1]];
                            let reach = delta[0].hypot(delta[1]);
                            let segment = Segment {
                                start: p[0],
                                delta,
                                length: reach,
                                along: length,
                            };
                            length += reach;
                            segment
                        })
                        .collect();
                    StrokeTile {
                        design: stroke.clone(),
                        size,
                        segments,
                        length,
                    }
                })
                .collect(),
        }
    }
    pub fn sample(&self, chart: [f32; 2], footprint: f32) -> NeedleworkSample {
        let mut result = NeedleworkSample {
            height: 0.,
            color: None,
            coverage: 0.,
        };
        for stroke in &self.strokes {
            let Some(local) = stroke.design.region.chart(chart) else {
                continue;
            };
            let point = [
                (local[0] * f32::from(stroke.design.across.0)).rem_euclid(1.) * stroke.size[0],
                (local[1] * f32::from(stroke.design.along.0)).rem_euclid(1.) * stroke.size[1],
            ];
            let sample = stroke.sample(point, footprint);
            result.height = result.height.max(sample.height);
            if sample.color.is_some() && sample.coverage > 0. {
                result.color = sample.color;
                result.coverage = sample.coverage;
            }
        }
        result
    }
}
impl StrokeTile {
    fn sample(&self, point: [f32; 2], footprint: f32) -> NeedleworkSample {
        let distance = self.nearest(point);
        let (radius, height, color, radial) = match &self.design.style {
            TextileStrokeStyle::Cord { diameter, color } => {
                let radius = diameter.metres() * 0.5;
                (radius, radius, Some(*color), distance.signed.abs())
            }
            TextileStrokeStyle::Stitches {
                width,
                length,
                gap,
                relief,
                color,
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
                width,
                pitch,
                relief,
                color,
            } => {
                let phase = distance.along / pitch.metres() * TAU;
                let strands = f32::from(TextileStrokeStyle::BRAID_STRANDS);
                let radius = width.metres() / (strands * 2.);
                let radial = (0..TextileStrokeStyle::BRAID_STRANDS)
                    .map(|strand| {
                        (distance.signed
                            - width.metres() / strands
                                * (phase + TAU * f32::from(strand) / strands).sin())
                        .abs()
                    })
                    .fold(f32::INFINITY, f32::min);
                (radius, relief.metres(), Some(*color), radial)
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
        NeedleworkSample {
            height: height * shape,
            color,
            coverage,
        }
    }
    fn nearest(&self, point: [f32; 2]) -> StrokeDistance {
        self.segments
            .iter()
            .map(|s| s.project(point))
            .min_by(|a, b| a.signed.abs().total_cmp(&b.signed.abs()))
            .unwrap_or(StrokeDistance {
                signed: f32::INFINITY,
                along: 0.,
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
        }
    }
}
