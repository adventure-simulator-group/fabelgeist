//! Route reconstruction and terrain-span presentation.

use super::{
    GridPosition, RoutePlan, RoutePoint, RouteSeconds, RoutingGrid, TerrainSpan, step_distance,
    step_seconds,
};
use crate::{
    BASIS_POINTS_PER_WHOLE, Error, MAX_ROUTE_NODES, MAX_ROUTE_POINTS, MAX_TERRAIN_SPANS, Result,
    RoutingWeather, Surface, TerrainSkillProfile, TerrainWeights,
};
use std::{cmp::Reverse, collections::HashMap};

// Spans retain exact seconds until each cumulative endpoint is rounded.
struct SecondSpan {
    surface: Surface,
    terrain: TerrainWeights,
    training_multiplier_permille: u16,
    check_millirank: u16,
    duration: RouteSeconds,
}

struct RouteMeasurements {
    distance_m: u64,
    seconds: RouteSeconds,
    spans: Vec<SecondSpan>,
}

pub(crate) fn simplify_route(points: &mut Vec<RoutePoint>, cap: usize) {
    if points.len() <= cap {
        return;
    }
    let stride = points.len().div_ceil(cap - 1);
    let mut simplified = points.iter().step_by(stride).cloned().collect::<Vec<_>>();
    if simplified.last() != points.last() {
        simplified.push(points[points.len() - 1].clone())
    }
    *points = simplified;
}

pub(super) fn reconstruct<G: RoutingGrid>(
    grid: &G,
    parent: &HashMap<GridPosition, GridPosition>,
    start: GridPosition,
    goal: GridPosition,
    profile: TerrainSkillProfile,
    weather: RoutingWeather,
) -> Result<RoutePlan> {
    let nodes = route_nodes(parent, start, goal)?;
    let measurements = measure_route(grid, &nodes, profile, weather)?;
    let minutes = measurements.seconds.rounded_minutes()?;
    let mut spans = minute_spans(measurements.spans)?;
    let mut points = nodes
        .into_iter()
        .map(|position| RoutePoint {
            longitude: f64::from(position.column()),
            latitude: f64::from(position.row()),
        })
        .collect::<Vec<_>>();
    retain_grid_turns(&mut points);
    simplify_route(&mut points, MAX_ROUTE_POINTS);
    compact_spans(&mut spans, minutes, MAX_TERRAIN_SPANS);
    Ok(RoutePlan {
        points,
        spans,
        distance_m: measurements.distance_m,
        minutes,
    })
}

pub(crate) fn compact_spans(spans: &mut Vec<TerrainSpan>, total_minutes: u64, cap: usize) {
    if spans.len() <= cap || cap == 0 || total_minutes == 0 {
        return;
    }
    let source = std::mem::take(spans);
    let mut compacted: Vec<TerrainSpan> = Vec::with_capacity(cap);
    for bucket in 0..cap {
        // Each quotient is at most total_minutes; the products need u128.
        let start = (u128::from(total_minutes) * bucket as u128 / cap as u128) as u64;
        let end = (u128::from(total_minutes) * (bucket as u128 + 1) / cap as u128) as u64;
        if start == end {
            continue;
        }
        let mut durations = [0_u64; 6];
        let mut training_mass = 0_u128;
        let mut check_mass = 0_u128;
        let mut terrain_training_mass = [0_u128; 5];
        for span in &source {
            let span_end = span.start_minute + span.duration_minutes;
            let overlap = end
                .min(span_end)
                .saturating_sub(start.max(span.start_minute));
            durations[surface_index(span.surface)] += overlap;
            let overlap = u128::from(overlap);
            let training = u128::from(span.training_multiplier_permille);
            training_mass += overlap * training;
            check_mass += overlap * u128::from(span.check_millirank);
            for (mass, weight) in terrain_training_mass.iter_mut().zip([
                span.terrain.plains,
                span.terrain.forest,
                span.terrain.hills,
                span.terrain.wetlands,
                span.terrain.urban,
            ]) {
                *mass += overlap * training * u128::from(weight);
            }
        }
        let surface = [
            Surface::Road,
            Surface::Open,
            Surface::SparseWoods,
            Surface::DeepWoods,
            Surface::Water,
            Surface::Wetland,
        ]
        .into_iter()
        .enumerate()
        .max_by_key(|(index, _)| (durations[*index], Reverse(*index)))
        .map(|(_, surface)| surface)
        .unwrap_or(Surface::Open);
        let duration = u128::from(end - start);
        let training_multiplier_permille = ((training_mass + duration / 2) / duration) as u16;
        let check_millirank = ((check_mass + duration / 2) / duration) as u16;
        let weights = normalized_weighted_parts(terrain_training_mass, training_mass);
        let terrain = TerrainWeights {
            plains: weights[0],
            forest: weights[1],
            hills: weights[2],
            wetlands: weights[3],
            urban: weights[4],
        };
        if let Some(previous) = compacted.last_mut().filter(|previous| {
            previous.surface == surface
                && previous.terrain == terrain
                && previous.training_multiplier_permille == training_multiplier_permille
                && previous.check_millirank == check_millirank
        }) {
            previous.duration_minutes += end - start;
        } else {
            compacted.push(TerrainSpan {
                surface,
                terrain,
                training_multiplier_permille,
                check_millirank,
                start_minute: start,
                duration_minutes: end - start,
            });
        }
    }
    *spans = compacted;
}

fn route_nodes(
    parent: &HashMap<GridPosition, GridPosition>,
    start: GridPosition,
    goal: GridPosition,
) -> Result<Vec<GridPosition>> {
    let mut nodes = vec![goal];
    let mut current = goal;
    while current != start {
        current = *parent.get(&current).ok_or(Error::NoRoute)?;
        nodes.push(current);
        if nodes.len() > MAX_ROUTE_NODES {
            return Err(Error::NoRoute);
        }
    }
    nodes.reverse();
    Ok(nodes)
}

fn measure_route<G: RoutingGrid>(
    grid: &G,
    nodes: &[GridPosition],
    profile: TerrainSkillProfile,
    weather: RoutingWeather,
) -> Result<RouteMeasurements> {
    let mut distance_m = 0_u64;
    let mut seconds = RouteSeconds::default();
    let mut spans: Vec<SecondSpan> = Vec::new();
    for pair in nodes.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let from = grid.cell(a.column(), a.row()).ok_or(Error::NoRoute)?;
        let to = grid.cell(b.column(), b.row()).ok_or(Error::NoRoute)?;
        let diagonal = a.column() != b.column() && a.row() != b.row();
        distance_m += step_distance(grid.metres_per_cell(), diagonal);
        let step = step_seconds(from, to, grid.metres_per_cell(), diagonal, profile, weather);
        seconds += step;
        let terrain = from
            .terrain_weights()
            .with_ground_moisture(weather.ground_moisture_bps)
            .weights;
        let training_multiplier_permille = if from.surface == Surface::Road {
            (from.underlying_surface().speed_metres_per_hour() / 5) as u16
        } else {
            1_000
        };
        let underlying_check = terrain.dot(profile);
        let whole_bps = u32::from(BASIS_POINTS_PER_WHOLE);
        let cover = u32::from(weather.snow_cover_bps);
        let check_millirank = (((u32::from(underlying_check) * (whole_bps - cover))
            + u32::from(weather.snow_check_millirank) * cover)
            / whole_bps) as u16;
        if let Some(last) = spans.last_mut().filter(|span| {
            span.surface == from.surface
                && span.terrain == terrain
                && span.training_multiplier_permille == training_multiplier_permille
                && span.check_millirank == check_millirank
        }) {
            last.duration += step;
        } else {
            spans.push(SecondSpan {
                surface: from.surface,
                terrain,
                training_multiplier_permille,
                check_millirank,
                duration: step,
            });
        }
    }
    Ok(RouteMeasurements {
        distance_m,
        seconds,
        spans,
    })
}

fn minute_spans(second_spans: Vec<SecondSpan>) -> Result<Vec<TerrainSpan>> {
    let mut cursor_seconds = RouteSeconds::default();
    let mut cursor_minutes = 0_u64;
    let mut spans: Vec<TerrainSpan> = Vec::new();
    for segment in second_spans {
        cursor_seconds += segment.duration;
        let end_minutes = cursor_seconds.rounded_minutes()?;
        let duration_minutes = end_minutes.saturating_sub(cursor_minutes);
        if duration_minutes == 0 {
            continue;
        }
        if let Some(last) = spans.last_mut().filter(|span| {
            span.surface == segment.surface
                && span.terrain == segment.terrain
                && span.training_multiplier_permille == segment.training_multiplier_permille
                && span.check_millirank == segment.check_millirank
        }) {
            last.duration_minutes += duration_minutes;
        } else {
            spans.push(TerrainSpan {
                surface: segment.surface,
                terrain: segment.terrain,
                training_multiplier_permille: segment.training_multiplier_permille,
                check_millirank: segment.check_millirank,
                start_minute: cursor_minutes,
                duration_minutes,
            });
        }
        cursor_minutes = end_minutes;
    }
    Ok(spans)
}

fn normalized_weighted_parts(numerators: [u128; 5], denominator: u128) -> [u16; 5] {
    if denominator == 0 {
        return [TerrainWeights::TOTAL, 0, 0, 0, 0];
    }
    let mut result = [0_u16; 5];
    let mut remainders = [0_u128; 5];
    for index in 0..5 {
        result[index] = (numerators[index] / denominator) as u16;
        remainders[index] = numerators[index] % denominator;
    }
    let missing = TerrainWeights::TOTAL - result.iter().sum::<u16>();
    let mut order = [0_usize, 1, 2, 3, 4];
    order.sort_unstable_by_key(|&index| (Reverse(remainders[index]), index));
    for index in order.into_iter().take(usize::from(missing)) {
        result[index] += 1;
    }
    result
}

fn surface_index(surface: Surface) -> usize {
    match surface {
        Surface::Road => 0,
        Surface::Open => 1,
        Surface::SparseWoods => 2,
        Surface::DeepWoods => 3,
        Surface::Water => 4,
        Surface::Wetland => 5,
    }
}

fn retain_grid_turns(points: &mut Vec<RoutePoint>) {
    if points.len() <= 2 {
        return;
    }
    let mut retained = Vec::with_capacity(points.len());
    retained.push(points[0].clone());
    for window in points.windows(3) {
        let before = (
            (window[1].longitude - window[0].longitude).signum() as i8,
            (window[1].latitude - window[0].latitude).signum() as i8,
        );
        let after = (
            (window[2].longitude - window[1].longitude).signum() as i8,
            (window[2].latitude - window[1].latitude).signum() as i8,
        );
        if before != after {
            retained.push(window[1].clone());
        }
    }
    retained.push(points[points.len() - 1].clone());
    *points = retained;
}
