//! Public grid routing and bounded search.

use crate::{
    BASIS_POINTS_PER_WHOLE, Cell, Error, MAX_ROUTE_NODES, Result, RoutingWeather, Surface,
    TerrainSkillProfile, TerrainWeights, check_deadline,
};
use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap},
    ops::{Add, AddAssign},
    time::Instant,
};

#[cfg(test)]
pub(super) use plan::compact_spans;
use plan::reconstruct;
pub(super) use plan::simplify_route;
use position::GridPosition;

mod plan;
mod position;

#[derive(Clone, Debug, PartialEq)]
pub struct RoutePoint {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TerrainSpan {
    pub surface: Surface,
    pub terrain: TerrainWeights,
    pub training_multiplier_permille: u16,
    pub check_millirank: u16,
    pub start_minute: u64,
    pub duration_minutes: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoutePlan {
    pub points: Vec<RoutePoint>,
    pub spans: Vec<TerrainSpan>,
    pub distance_m: u64,
    pub minutes: u64,
}

// A bounded search has at most MAX_ROUTE_NODES predecessor edges. Even at
// u32::MAX metres per cell, their exact seconds and heuristic fit in u128.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct RouteSeconds(u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Queue {
    f: RouteSeconds,
    g: RouteSeconds,
    position: GridPosition,
}
/// Small deterministic A* kernel used by the pack planner and exhaustive fixtures.
pub trait RoutingGrid {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn cell(&self, x: u32, y: u32) -> Option<Cell>;
    fn metres_per_cell(&self) -> u32;
}

impl RouteSeconds {
    const MINIMUM_STEP: u128 = 1;

    fn rounded_minutes(self) -> Result<u64> {
        u64::try_from(self.0.div_ceil(60))
            .map_err(|_| Error::Validation("route minutes exceed the supported range".into()))
    }
}

impl Add for RouteSeconds {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl AddAssign for RouteSeconds {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Ord for Queue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.f
            .cmp(&other.f)
            .then(self.g.cmp(&other.g))
            .then(self.position.cmp(&other.position))
    }
}
impl PartialOrd for Queue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn astar<G: RoutingGrid>(grid: &G, start: (u32, u32), goal: (u32, u32)) -> Result<RoutePlan> {
    astar_with_profile(grid, start, goal, TerrainSkillProfile::default())
}

pub fn astar_with_profile<G: RoutingGrid>(
    grid: &G,
    start: (u32, u32),
    goal: (u32, u32),
    profile: TerrainSkillProfile,
) -> Result<RoutePlan> {
    if !profile.is_valid() {
        return Err(Error::Validation(
            "terrain skill profile exceeds rank five".into(),
        ));
    }
    astar_with_profile_and_deadline(grid, start, goal, profile, RoutingWeather::default(), None)
}

pub fn astar_with_profile_and_weather<G: RoutingGrid>(
    grid: &G,
    start: (u32, u32),
    goal: (u32, u32),
    profile: TerrainSkillProfile,
    weather: RoutingWeather,
) -> Result<RoutePlan> {
    if !profile.is_valid() || !weather.is_valid() {
        return Err(Error::Validation(
            "terrain skill or weather profile exceeds bounded values".into(),
        ));
    }
    astar_with_profile_and_deadline(grid, start, goal, profile, weather, None)
}

pub(super) fn astar_with_profile_and_deadline<G: RoutingGrid>(
    grid: &G,
    start: (u32, u32),
    goal: (u32, u32),
    profile: TerrainSkillProfile,
    weather: RoutingWeather,
    deadline: Option<Instant>,
) -> Result<RoutePlan> {
    if start.0 >= grid.width()
        || start.1 >= grid.height()
        || goal.0 >= grid.width()
        || goal.1 >= grid.height()
    {
        return Err(Error::NoRoute);
    }
    let start = GridPosition::from(start);
    let goal = GridPosition::from(goal);
    let mut open = BinaryHeap::new();
    let mut best = HashMap::new();
    let mut parent = HashMap::new();
    best.insert(start, RouteSeconds::default());
    open.push(Reverse(Queue {
        f: heuristic(start, goal, grid.metres_per_cell()),
        g: RouteSeconds::default(),
        position: start,
    }));
    let mut visited = 0;
    while let Some(Reverse(current)) = open.pop() {
        if best.get(&current.position) != Some(&current.g) {
            continue;
        }
        if current.position == goal {
            return reconstruct(grid, &parent, start, goal, profile, weather);
        }
        visited += 1;
        if visited % 256 == 0 {
            check_deadline(deadline)?;
        }
        if visited > MAX_ROUTE_NODES {
            return Err(Error::NoRoute);
        }
        for dy in -1_i32..=1 {
            for dx in -1_i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let Some(next) = current.position.offset(dx, dy) else {
                    continue;
                };
                if next.column() >= grid.width() || next.row() >= grid.height() {
                    continue;
                }
                let Some(from) = grid.cell(current.position.column(), current.position.row())
                else {
                    continue;
                };
                let Some(to) = grid.cell(next.column(), next.row()) else {
                    continue;
                };
                if to.surface == Surface::Water && !to.crossing {
                    continue;
                }
                if dx != 0
                    && dy != 0
                    && (!grid
                        .cell(next.column(), current.position.row())
                        .is_some_and(passable)
                        || !grid
                            .cell(current.position.column(), next.row())
                            .is_some_and(passable))
                {
                    continue;
                }
                let step = step_seconds(
                    from,
                    to,
                    grid.metres_per_cell(),
                    dx != 0 && dy != 0,
                    profile,
                    weather,
                );
                let next_g = current.g + step;
                if best.get(&next).is_none_or(|known| next_g < *known) {
                    best.insert(next, next_g);
                    parent.insert(next, current.position);
                    open.push(Reverse(Queue {
                        f: next_g + heuristic(next, goal, grid.metres_per_cell()),
                        g: next_g,
                        position: next,
                    }));
                }
            }
        }
    }
    Err(Error::NoRoute)
}

pub(super) fn passable(cell: Cell) -> bool {
    cell.surface != Surface::Water || cell.crossing
}
pub(super) fn snap<G: RoutingGrid>(
    grid: &G,
    origin: (u32, u32),
    radius: i32,
) -> Option<(u32, u32)> {
    for distance in 0..=radius {
        for dy in -distance..=distance {
            for dx in -distance..=distance {
                if dx.abs().max(dy.abs()) != distance {
                    continue;
                }
                let x = origin.0 as i32 + dx;
                let y = origin.1 as i32 + dy;
                if x >= 0
                    && y >= 0
                    && x < grid.width() as i32
                    && y < grid.height() as i32
                    && grid
                        .cell(x as u32, y as u32)
                        .is_some_and(|c| c.surface != Surface::Water || c.crossing)
                {
                    return Some((x as u32, y as u32));
                }
            }
        }
    }
    None
}
// The same truncated per-edge distance is used by search and reconstruction.
fn step_distance(metres: u32, diagonal: bool) -> u64 {
    const DIAGONAL_PER_THOUSAND: u64 = 1_414;
    const DISTANCE_SCALE: u64 = 1_000;
    u64::from(metres)
        * if diagonal {
            DIAGONAL_PER_THOUSAND
        } else {
            DISTANCE_SCALE
        }
        / DISTANCE_SCALE
}

fn minimum_step_seconds(distance: u64) -> RouteSeconds {
    let whole_bps = u128::from(BASIS_POINTS_PER_WHOLE);
    let base =
        (u128::from(distance) * 3_600).div_ceil(u128::from(Surface::MAX_SPEED_METRES_PER_HOUR));
    let seconds = (base * whole_bps)
        .div_ceil(whole_bps + u128::from(TerrainSkillProfile::MAX))
        .max(RouteSeconds::MINIMUM_STEP);
    RouteSeconds(seconds)
}

fn heuristic(a: GridPosition, b: GridPosition, metres: u32) -> RouteSeconds {
    let dx = a.column().abs_diff(b.column());
    let dy = a.row().abs_diff(b.row());
    let diagonal = u128::from(dx.min(dy));
    let straight = u128::from(dx.max(dy) - dx.min(dy));
    let diagonal_cost = minimum_step_seconds(step_distance(metres, true));
    let straight_cost = minimum_step_seconds(step_distance(metres, false));
    RouteSeconds(diagonal * diagonal_cost.0 + straight * straight_cost.0)
}
fn step_seconds(
    from: Cell,
    to: Cell,
    metres: u32,
    diagonal: bool,
    profile: TerrainSkillProfile,
    weather: RoutingWeather,
) -> RouteSeconds {
    let distance = step_distance(metres, diagonal);
    let base = (u128::from(distance) * 3_600)
        .div_ceil(u128::from(from.surface.speed_metres_per_hour().max(1)));
    let rise = i32::from(to.elevation_m) - i32::from(from.elevation_m);
    let uphill = if rise > 0 {
        1000 + (u64::try_from(rise).unwrap_or(0) * 6).min(1500)
    } else {
        1000
    };
    let weathered = from
        .terrain_weights()
        .with_ground_moisture(weather.ground_moisture_bps);
    let underlying_check = weathered.weights.dot(profile);
    let whole_bps = u32::from(BASIS_POINTS_PER_WHOLE);
    let cover = u32::from(weather.snow_cover_bps);
    let check = u64::from(
        ((u32::from(underlying_check) * (whole_bps - cover))
            + u32::from(weather.snow_check_millirank) * cover)
            / whole_bps,
    );
    let whole_bps = u64::from(BASIS_POINTS_PER_WHOLE);
    let saturation_factor = whole_bps + u64::from(weathered.saturation_bps);
    let snow_cost_bps = u64::from(weather.snow_cover_bps)
        * u64::from(5_000u16.saturating_sub(weather.snow_check_millirank))
        / whole_bps;
    // check is milli-rank: 1 + rank/10 => (10_000 + check) / 10_000.
    let seconds = (base
        * u128::from(uphill)
        * u128::from(saturation_factor)
        * u128::from(whole_bps + snow_cost_bps))
    .div_ceil(u128::from(1000 * whole_bps * (whole_bps + check)))
    .max(RouteSeconds::MINIMUM_STEP);
    RouteSeconds(seconds)
}
