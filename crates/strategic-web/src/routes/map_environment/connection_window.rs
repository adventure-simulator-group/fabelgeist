//! Bounded presentation clipping and scale-dependent source simplification.
use adventuresim_tactical_core::{
    regional_environment::{
        MAX_REGIONAL_CONNECTION_POINTS, MAX_REGIONAL_CONNECTIONS, RegionalConnection,
        RegionalEnvironmentError,
    },
    regional_terrain::{REGIONAL_TERRAIN_SIDE, RegionalTerrainRequest},
};
use adventuresim_terrain::road_pack::RoadPack;
use adventuresim_world_schema::{
    coordinates::{
        Wgs84BoundsE7, Wgs84CoordinateE7,
        terrain_projection::{NativeTerrainCoordinate, NativeTerrainOffset},
    },
    regional_connection::RegionalConnectionKind,
};
use simplification::simplify;

#[path = "connection_window/simplification.rs"]
mod simplification;

/// Maximum perpendicular presentation error relative to a captured grid cell.
const CONNECTION_SIMPLIFICATION_CELL_FRACTION: f64 = 0.1;
type Result<T> = std::result::Result<T, RegionalEnvironmentError>;

struct WindowOutput {
    lines: Vec<RegionalConnection>,
    points: usize,
}

impl WindowOutput {
    fn admit(
        &mut self,
        kind: RegionalConnectionKind,
        points: Vec<Wgs84CoordinateE7>,
    ) -> Result<()> {
        if points.len() < 2 {
            return Ok(());
        }
        let line = RegionalConnection::new(kind, points)?;
        self.points += line.points().len();
        if self.points > MAX_REGIONAL_CONNECTION_POINTS
            || self.lines.len() >= MAX_REGIONAL_CONNECTIONS
        {
            return Err(RegionalEnvironmentError::Capacity);
        }
        self.lines.push(line);
        Ok(())
    }
}

pub(super) fn capture(
    roads: &RoadPack,
    request: RegionalTerrainRequest,
) -> Result<Vec<RegionalConnection>> {
    let origin = NativeTerrainCoordinate::from(request.origin.to_e7());
    let spacing = f64::from(request.scale.spacing_metres());
    let half = (REGIONAL_TERRAIN_SIDE - 1) as f64 * 0.5 * spacing;
    let southwest = origin.at_offset(-half, -half);
    let northeast = origin.at_offset(half, half);
    let bounds = Wgs84BoundsE7::from_longitude_latitude_degrees([
        southwest.longitude_degrees.clamp(-180.0, 180.0),
        southwest.latitude_degrees.clamp(-90.0, 90.0),
        northeast.longitude_degrees.clamp(-180.0, 180.0),
        northeast.latitude_degrees.clamp(-90.0, 90.0),
    ])
    .ok_or(RegionalEnvironmentError::Geometry)?;
    let mut output = WindowOutput {
        lines: Vec::new(),
        points: 0,
    };
    for line in roads.intersecting(bounds) {
        let offsets = line
            .points()
            .iter()
            .map(|point| origin.offset_to((*point).into()))
            .collect::<Vec<_>>();
        let offsets = simplify(&offsets, spacing * CONNECTION_SIMPLIFICATION_CELL_FRACTION);
        let mut piece = Vec::new();
        for segment in offsets.windows(2) {
            let Some(clipped) = clip([segment[0], segment[1]], half) else {
                output.admit(line.kind(), std::mem::take(&mut piece))?;
                continue;
            };
            let [start, end] = clipped.map(|point| {
                let point = origin.at_offset(point.east_metres, point.north_metres);
                Wgs84CoordinateE7::from_longitude_latitude_degrees(
                    point.longitude_degrees,
                    point.latitude_degrees,
                )
                .ok_or(RegionalEnvironmentError::Geometry)
            });
            let (start, end) = (start?, end?);
            if start == end {
                continue;
            }
            if piece.last().is_some_and(|last| *last != start) {
                output.admit(line.kind(), std::mem::take(&mut piece))?;
            }
            if piece.is_empty() {
                piece.push(start);
            }
            piece.push(end);
            if piece.len() + output.points > MAX_REGIONAL_CONNECTION_POINTS {
                return Err(RegionalEnvironmentError::Capacity);
            }
        }
        output.admit(line.kind(), piece)?;
    }
    Ok(output.lines)
}

fn clip(
    [start, end]: [NativeTerrainOffset; 2],
    half_metres: f64,
) -> Option<[NativeTerrainOffset; 2]> {
    let mut enter = 0.0_f64;
    let mut leave = 1.0_f64;
    for [first, last] in [
        [start.east_metres, end.east_metres],
        [start.north_metres, end.north_metres],
    ] {
        let delta = last - first;
        if delta == 0.0 {
            if first.abs() > half_metres {
                return None;
            }
        } else {
            let near = (-half_metres - first) / delta;
            let far = (half_metres - first) / delta;
            enter = enter.max(near.min(far));
            leave = leave.min(near.max(far));
            if enter >= leave {
                return None;
            }
        }
    }
    Some([enter, leave].map(|parameter| NativeTerrainOffset {
        east_metres: start.east_metres + (end.east_metres - start.east_metres) * parameter,
        north_metres: start.north_metres + (end.north_metres - start.north_metres) * parameter,
    }))
}
