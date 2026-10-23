//! Deterministic textile color maps, shared by native preview and export.
use crate::textile_botanical::BotanicalTile;
use crate::textile_sampling::TextileRasterSize;
use crate::textile_weave::{composite_yarn, weave_coverage};
use crate::{
    GenerateError, Permille, TextileAppearance, TextileAxis, TextileFill, TextileMotif,
    TextileRegion, TextileRepeatCount, TextileSpan,
};
use std::f32::consts::{PI, TAU};

pub struct TextilePixels {
    pub width: u32,
    pub height: u32,
    /// Straight-alpha sRGB RGBA8; row zero is the proximal fabric-chart edge.
    pub color: Vec<u8>,
    pub(crate) needlework_heights: Vec<f32>,
    pub(crate) chart_boundary: crate::TextileChartBoundary,
}

impl TextileAppearance {
    pub fn pixels(&self) -> Result<TextilePixels, GenerateError> {
        self.validate()?;
        let tiles = self
            .layers
            .0
            .iter()
            .map(|layer| match &layer.fill {
                TextileFill::Motifs {
                    motif: TextileMotif::Scrollwork { branches, curl },
                    ..
                } => Some(BotanicalTile::new(branches.0, curl.unit())),
                TextileFill::Motifs {
                    motif: TextileMotif::Damask { leaf_pairs, spread },
                    ..
                } => Some(BotanicalTile::damask(leaf_pairs.0, spread.unit())),
                _ => None,
            })
            .collect::<Vec<_>>();
        let size = TextileRasterSize::for_appearance(self);
        let needlework = self
            .needlework
            .as_ref()
            .map(|n| crate::textile_stroke_raster::NeedleworkTile::new(n));
        let mut needlework_heights = Vec::new();
        let mut color = Vec::with_capacity((size.width * size.height * 4) as usize);
        for row in 0..size.height {
            for column in 0..size.width {
                let uv = [
                    (column as f32 + 0.5) / size.width as f32,
                    (row as f32 + 0.5) / size.height as f32,
                ];
                let mut pixel = self.color_at(
                    uv,
                    [1. / size.width as f32, 1. / size.height as f32],
                    &tiles,
                );
                if let Some(tile) = &needlework {
                    let chart = self
                        .needlework
                        .as_ref()
                        .map(|n| n.chart)
                        .ok_or(crate::GenerateError::InvalidSurface)?;
                    let footprint = (chart.across.metres() / size.width as f32)
                        .max(chart.along.metres() / size.height as f32);
                    let sample = tile.sample(uv, footprint);
                    if let Some(thread) = sample.color {
                        pixel = composite_yarn(pixel, thread, sample.coverage);
                    }
                    needlework_heights.push(sample.height);
                }
                color.extend(pixel);
            }
        }
        Ok(TextilePixels {
            width: size.width,
            height: size.height,
            color,
            needlework_heights,
            chart_boundary: self.chart_boundary,
        })
    }

    /// Native normalized material-chart boundary. This samples appearance,
    /// never body positions or skin correspondence; callers freeze the chart.
    fn color_at(
        &self,
        chart: [f32; 2],
        footprint: [f32; 2],
        tiles: &[Option<BotanicalTile>],
    ) -> [u8; 4] {
        let mut color = [
            self.color.0[0],
            self.color.0[1],
            self.color.0[2],
            (self.opacity.unit() * 255.).round() as u8,
        ];
        for (layer, tile) in self.layers.0.iter().zip(tiles) {
            if let Some(local) = layer.region.chart(chart) {
                let coverage = layer.fill.coverage(local, chart, footprint, tile.as_ref());
                color = composite_yarn(color, layer.color, coverage);
            }
        }
        color
    }
}

impl TextileAxis {
    fn coordinate(self, point: [f32; 2]) -> f32 {
        match self {
            Self::Across => point[0],
            Self::Along => point[1],
        }
    }
}

impl TextileRegion {
    pub(crate) fn chart(&self, point: [f32; 2]) -> Option<[f32; 2]> {
        match *self {
            Self::Whole => Some(point),
            Self::Panel { across, along } => {
                Some([across.coordinate(point[0])?, along.coordinate(point[1])?])
            }
            Self::Band { axis, start, end } => {
                let position = axis.coordinate(point);
                if position < start.unit() || position > end.unit() {
                    return None;
                }
                let local = (position - start.unit()) / (end.unit() - start.unit());
                Some(match axis {
                    TextileAxis::Across => [local, point[1]],
                    TextileAxis::Along => [point[0], local],
                })
            }
        }
    }
}

impl TextileSpan {
    fn coordinate(self, value: f32) -> Option<f32> {
        (value >= self.start.unit() && value <= self.end.unit())
            .then(|| (value - self.start.unit()) / (self.end.unit() - self.start.unit()))
    }
}

impl TextileFill {
    fn coverage(
        &self,
        point: [f32; 2],
        garment: [f32; 2],
        footprint: [f32; 2],
        tile: Option<&BotanicalTile>,
    ) -> f32 {
        let covered = match self {
            Self::Solid => true,
            Self::Weave { warp, weft, width } => {
                return weave_coverage(garment, footprint, *warp, *weft, *width);
            }
            Self::Stripes {
                axis,
                repeats,
                width,
                phase,
            } => {
                let position = axis.coordinate(point) * f32::from(repeats.0) + phase.unit();
                position.rem_euclid(1.0) < width.unit()
            }
            Self::Motifs {
                space,
                motif,
                across,
                along,
                scale,
                rotation,
                stagger,
                offset,
            } => {
                let point = match space {
                    crate::TextilePatternSpace::Region => point,
                    crate::TextilePatternSpace::Garment => garment,
                };
                let row = point[1] * f32::from(along.0) + offset.along.unit();
                let x = point[0] * f32::from(across.0)
                    + offset.across.unit()
                    + row.floor().rem_euclid(2.0) * stagger.unit();
                let local = [
                    (x.rem_euclid(1.0) - 0.5) / scale.unit(),
                    (row.rem_euclid(1.0) - 0.5) / scale.unit(),
                ];
                motif.covers(rotate(local, rotation.unit() * TAU), tile)
            }
        };
        if covered { 1. } else { 0. }
    }
}

impl TextileMotif {
    fn covers(&self, point: [f32; 2], tile: Option<&BotanicalTile>) -> bool {
        match self {
            Self::Lozenge => point[0].abs() + point[1].abs() < 0.42,
            Self::Rosette { petals } => {
                let angle = point[1].atan2(point[0]);
                let radius = 0.28 + 0.1 * (angle * f32::from(petals.0)).cos();
                point[0].hypot(point[1]) < radius
            }
            Self::Medallion {
                petals,
                rings,
                threads,
                stroke,
            } => medallion(point, *petals, *rings, *threads, *stroke),
            Self::Palmette { leaf_pairs, spread } => palmette(point, leaf_pairs.0, spread.unit()),
            Self::Scrollwork { .. } | Self::Damask { .. } => {
                tile.is_some_and(|tile| tile.covers(point))
            }
        }
    }
}

/// Native ornament kernel: normalized tile XY, with rounded stitch footprints
/// alternating warp/weft directions. Ring stitches follow their radial frame.
fn medallion(
    point: [f32; 2],
    petals: TextileRepeatCount,
    rings: TextileRepeatCount,
    threads: TextileRepeatCount,
    stroke: Permille,
) -> bool {
    let radius = point[0].hypot(point[1]);
    let angle = point[1].atan2(point[0]);
    let flower = (0..petals.0).any(|petal| {
        let direction = TAU * f32::from(petal) / f32::from(petals.0);
        ellipse(
            [
                point[0] - direction.cos() * 0.095,
                point[1] - direction.sin() * 0.095,
            ],
            [0.075, 0.075],
        )
    }) || ellipse(point, [0.055, 0.055]);
    let thread = point.map(|v| v * f32::from(threads.0));
    let local = thread.map(|v| v.rem_euclid(1.) - 0.5);
    let warp = (thread[0].floor() + thread[1].floor()).rem_euclid(2.) < 1.;
    let weave = ellipse(local, if warp { [0.46, 0.16] } else { [0.16, 0.46] });
    let rays = radius > 0.2
        && radius < 0.29
        && (angle * f32::from(threads.0) + radius * TAU * f32::from(rings.0))
            .sin()
            .abs()
            < 0.3;
    let border = (0..rings.0).any(|ring| {
        let thread_radius = 0.42 - f32::from(ring) * 0.055;
        (radius - thread_radius).abs() < stroke.unit() * 0.5
            && (angle * f32::from(threads.0) * 2.).sin() > -0.6
    });
    flower || weave || rays || border
}

/// Native motif kernel: normalized tile coordinates, with paired elliptical
/// leaves, a curved stem and a crowned blossom. These are authored geometry
/// ratios, independent of any museum image or garment dimensions.
fn palmette(point: [f32; 2], pairs: u8, spread: f32) -> bool {
    let stem_x = 0.045 * (point[1] * TAU).sin();
    if point[1].abs() < 0.34 && (point[0] - stem_x).abs() < 0.018 {
        return true;
    }
    for pair in 0..pairs {
        let level = f32::from(pair) / f32::from(pairs);
        let y = 0.24 - level * 0.48;
        let reach = (0.19 - level * 0.05) * spread;
        for side in [-1.0, 1.0] {
            let local = [point[0] - side * reach, point[1] - y];
            let leaf = rotate(local, side * (0.55 + level * 0.45));
            if ellipse(leaf, [0.145 * spread, 0.052]) {
                return true;
            }
            let branch = rotate(
                [point[0] - side * reach * 0.5, point[1] - y - 0.02],
                side * 0.25,
            );
            if ellipse(branch, [reach * 0.6, 0.014]) {
                return true;
            }
        }
    }
    let blossom = [point[0], point[1] + 0.32];
    if ellipse(blossom, [0.105, 0.085]) {
        return true;
    }
    for angle in [-PI / 3.0, 0.0, PI / 3.0] {
        let crown = [point[0] - angle.sin() * 0.08, point[1] + 0.4];
        if ellipse(rotate(crown, angle), [0.022, 0.05]) {
            return true;
        }
    }
    false
}

fn ellipse(point: [f32; 2], radii: [f32; 2]) -> bool {
    (point[0] / radii[0]).powi(2) + (point[1] / radii[1]).powi(2) < 1.0
}

fn rotate(point: [f32; 2], angle: f32) -> [f32; 2] {
    let (sin, cos) = angle.sin_cos();
    [
        point[0] * cos - point[1] * sin,
        point[0] * sin + point[1] * cos,
    ]
}
