//! A stroke's path in chart-local metres: its course between the authored
//! points, and the repeats that reach into one tile.
use crate::{TextilePathClosure, TextilePathCourse, TextileRepeatCount, TextileStroke};

/// Straight pieces drawn for each span of a curved course.
const CURVE_STEPS: u8 = 6;
/// Uses of a repeat tile's fixed variation values.
pub(crate) const TURN_CHANNEL: u8 = 0;
pub(crate) const SHIFT_CHANNELS: [u8; 2] = [1, 2];

/// One repeat of a path that overlaps the sampled tile.
pub(crate) struct StrokeCopy {
    /// Whole tiles from the sampled tile to the repeat's own.
    pub tiles: [i8; 2],
    /// The same displacement in metres.
    pub offset: [f32; 2],
}

impl StrokeCopy {
    /// Repeats whose path, widened by its thread and by any variation,
    /// overlaps the tile. `joins` says along which axes a tile has neighbours.
    pub fn reaching(
        stroke: &TextileStroke,
        size: [f32; 2],
        points: &[[f32; 2]],
        joins: [bool; 2],
    ) -> Vec<Self> {
        let center = middle(points);
        let mut reach = stroke.style.reach_metres();
        // A turned repeat stays within the circle about its middle.
        let turned = stroke.variation.map(|variation| {
            reach += size[0].min(size[1]) * variation.shift.unit();
            points
                .iter()
                .map(|p| (p[0] - center[0]).hypot(p[1] - center[1]))
                .fold(0., f32::max)
        });
        let shifts = |axis: usize| {
            let extent = |bound: fn(f32, f32) -> f32, start: f32| {
                points.iter().map(|p| p[axis]).fold(start, bound)
            };
            let (low, high) = turned.map_or_else(
                || {
                    (
                        extent(f32::min, f32::INFINITY),
                        extent(f32::max, f32::NEG_INFINITY),
                    )
                },
                |radius| (center[axis] - radius, center[axis] + radius),
            );
            let tiles = if joins[axis] {
                TextileStroke::NEIGHBOR_TILES
            } else {
                0..=0
            };
            tiles
                .filter(|&tile| {
                    let shift = f32::from(tile) * size[axis];
                    high + reach + shift >= 0. && low - reach + shift <= size[axis]
                })
                .collect::<Vec<_>>()
        };
        let along = shifts(1);
        shifts(0)
            .into_iter()
            .flat_map(|across| along.iter().map(move |&along| [across, along]))
            .map(|tiles| Self {
                tiles,
                offset: [f32::from(tiles[0]) * size[0], f32::from(tiles[1]) * size[1]],
            })
            .collect()
    }

    /// Index of this repeat's tile, wrapped so a closed chart joins. `cell`
    /// holds the sampled tile's whole-number indices as native floats.
    pub fn tile(&self, cell: [f32; 2], repeats: [TextileRepeatCount; 2]) -> [u8; 2] {
        std::array::from_fn(|axis| {
            let count = f32::from(repeats[axis].0);
            // Exact: a whole number below the repeat count, at most 32.
            (cell[axis] + f32::from(self.tiles[axis])).rem_euclid(count) as u8
        })
    }
}

/// The path's points in metres, closed and curved as authored.
pub(crate) fn course(stroke: &TextileStroke, size: [f32; 2]) -> Vec<[f32; 2]> {
    run(
        stroke,
        stroke
            .points
            .iter()
            .map(|p| [p.across.unit() * size[0], p.along.unit() * size[1]])
            .collect(),
    )
}

/// Height in metres of the thread's middle line at each point of the path's
/// [`course`].
pub(crate) fn lifts(stroke: &TextileStroke) -> Vec<f32> {
    run(
        stroke,
        stroke.points.iter().map(|p| [p.lift.metres()]).collect(),
    )
    .into_iter()
    .map(|[lift]| lift)
    .collect()
}

/// Values held at the authored points, carried along the path as it is
/// closed and curved.
fn run<const N: usize>(stroke: &TextileStroke, points: Vec<[f32; N]>) -> Vec<[f32; N]> {
    let closed = stroke.closure == TextilePathClosure::Closed;
    let count = points.len();
    let spans = if closed { count } else { count - 1 };
    // Beyond an open path's ends the curve continues its last direction.
    let at = |index: isize| {
        if closed {
            points[index.rem_euclid(count as isize) as usize]
        } else {
            points[index.clamp(0, count as isize - 1) as usize]
        }
    };
    let steps = match stroke.course {
        TextilePathCourse::Straight => 1,
        TextilePathCourse::Curved => CURVE_STEPS,
    };
    let mut path = Vec::with_capacity(spans * usize::from(steps) + 1);
    for span in 0..spans as isize {
        let control = [at(span - 1), at(span), at(span + 1), at(span + 2)];
        path.extend((0..steps).map(|step| {
            let t = f32::from(step) / f32::from(steps);
            std::array::from_fn(|axis| catmull_rom(control.map(|p| p[axis]), t))
        }));
    }
    path.push(at(spans as isize));
    path
}

/// Middle of the box around the path.
pub(crate) fn middle(points: &[[f32; 2]]) -> [f32; 2] {
    let [low, high] = bounds(points, 0.);
    std::array::from_fn(|axis| (low[axis] + high[axis]) * 0.5)
}

/// Lowest and highest corner of the box around the path, widened by `reach`.
pub(crate) fn bounds(points: &[[f32; 2]], reach: f32) -> [[f32; 2]; 2] {
    let corner = |bound: fn(f32, f32) -> f32, start: f32, reach: f32| {
        std::array::from_fn(|axis| points.iter().map(|p| p[axis]).fold(start, bound) + reach)
    };
    [
        corner(f32::min, f32::INFINITY, -reach),
        corner(f32::max, f32::NEG_INFINITY, reach),
    ]
}

/// Uniform Catmull-Rom spline through `p[1]` at 0 and `p[2]` at 1.
fn catmull_rom(p: [f32; 4], t: f32) -> f32 {
    let a = -p[0] + 3. * p[1] - 3. * p[2] + p[3];
    let b = 2. * p[0] - 5. * p[1] + 4. * p[2] - p[3];
    let c = p[2] - p[0];
    p[1] + 0.5 * t * (c + t * (b + t * a))
}
