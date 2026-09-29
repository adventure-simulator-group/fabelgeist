//! Helmet carriers as lists of coordinates: the host builds every vertex and
//! triangle of the dome and the crown, and names each vertex by what it is
//! instead of where it is.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use super::codes::VertexKind;
use crate::gpu::coord::{CoordExtrusion, CoordShell};
use crate::{BoundaryNormals, HelmetCrown, PlateFluting};

/// Samples around a helmet's rim.
pub(super) const AROUND: usize = 48;
/// Latitude rings of a smooth dome, the pole excluded.
const DOME_RINGS: usize = 12;
/// Across-crown samples of a fanned dome, before any flute columns.
const FAN_COLUMNS: usize = 40;

/// A crown row: a fixed latitude, or one edge of a crest whose latitude
/// depends on the head's width.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Latitude {
    Regular(f32),
    /// `side` is -1 before the crown's meridian and +1 after it.
    Crest {
        side: f32,
        base: bool,
    },
}

/// The crown's rows in order of latitude.
///
/// A crest adds rows at `FRAC_PI_2 ∓ asin(width / radius)`, whose latitudes
/// depend on the head. The widths are at most 0.12 of the radius and the
/// base width is twice the half width, so every crest row falls between the
/// regular rows at `11π/24` and `13π/24` (asin(0.12) < π/24), base outside
/// half: the order is the same for every head, and is decided here.
fn latitudes(crest: bool) -> Vec<(Latitude, usize)> {
    let mut values = (1..AROUND / 2)
        .map(|row| {
            (
                Latitude::Regular(row as f32 / (AROUND / 2) as f32 * PI),
                row,
            )
        })
        .collect::<Vec<_>>();
    if crest {
        let middle = AROUND / 4;
        let edge = |side, base| (Latitude::Crest { side, base }, middle);
        // The regular row at the meridian sits at index `middle - 1`.
        values.splice(
            middle - 1..middle - 1,
            [edge(-1.0, true), edge(-1.0, false)],
        );
        values.splice(middle + 2..middle + 2, [edge(1.0, false), edge(1.0, true)]);
    }
    values
}

/// A carrier under construction: coordinates and triangles.
#[derive(Default)]
pub(super) struct CoordSurface {
    pub coords: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl CoordSurface {
    pub fn vertex(&mut self, kind: VertexKind, values: [f32; 3]) -> u32 {
        self.coord([kind as u32 as f32, values[0], values[1], values[2]])
    }

    /// A vertex whose four coordinates are the shape's own.
    pub fn coord(&mut self, coord: [f32; 4]) -> u32 {
        self.coords.push(coord);
        self.coords.len() as u32 - 1
    }

    pub fn connect(&mut self, upper: &[u32], lower: &[u32], periodic: bool) {
        let count = if periodic {
            upper.len()
        } else {
            upper.len() - 1
        };
        for i in 0..count {
            let next = (i + 1) % upper.len();
            self.indices.extend([
                upper[i],
                lower[i],
                lower[next],
                upper[i],
                lower[next],
                upper[next],
            ]);
        }
    }

    /// A smooth dome from its pole down to the brow; returns the rim.
    pub fn full_dome(&mut self) -> Vec<u32> {
        let angles = (0..AROUND)
            .map(|i| i as f32 / AROUND as f32 * TAU)
            .collect::<Vec<_>>();
        let pole = self.vertex(VertexKind::DomePole, [0.0; 3]);
        let mut previous = Vec::new();
        for row in 1..=DOME_RINGS {
            let latitude = row as f32 / DOME_RINGS as f32 * FRAC_PI_2;
            let ring = angles
                .iter()
                .map(|&angle| self.vertex(VertexKind::DomeRing, [latitude, angle, 0.0]))
                .collect::<Vec<_>>();
            if previous.is_empty() {
                for i in 0..angles.len() {
                    self.indices
                        .extend([pole, ring[i], ring[(i + 1) % angles.len()]]);
                }
            } else {
                self.connect(&previous, &ring, true);
            }
            previous = ring;
        }
        previous
    }

    /// The crown of a styled helmet; a fluted or crested crown is fanned
    /// from the temples. Returns the rim.
    pub fn styled_dome(&mut self, style: &HelmetCrown, crest: f32) -> Vec<u32> {
        let first_index = self.indices.len();
        if style.fluting.is_none() && crest <= 0.0 {
            return self.full_dome();
        }
        let rim = self.fan_dome(style.fluting.as_ref(), crest > 0.0);
        for face in self.indices[first_index..].as_chunks_mut::<3>().0 {
            face.swap(1, 2);
        }
        rim
    }

    fn fan_dome(&mut self, pattern: Option<&PlateFluting>, crest: bool) -> Vec<u32> {
        let columns = pattern.map_or_else(
            || {
                (0..=FAN_COLUMNS)
                    .map(|i| i as f32 / FAN_COLUMNS as f32)
                    .collect()
            },
            |p| p.columns(FAN_COLUMNS),
        );
        let rim = (0..AROUND)
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * 2.0 * PI;
                self.vertex(VertexKind::FanRim, [angle, 0.0, 0.0])
            })
            .collect::<Vec<_>>();
        let first = rim[AROUND / 4];
        let mut rings: Vec<Vec<u32>> = Vec::new();
        for (latitude, rim_row) in latitudes(crest) {
            let front = (AROUND + AROUND / 4 - rim_row) % AROUND;
            let mut ring = Vec::with_capacity(columns.len());
            for (column, u) in columns.iter().enumerate() {
                // The first and last columns are the rim itself, whose flute
                // relief is zero: flutes never reach a chart's edges.
                let id = if column == 0 {
                    rim[front]
                } else if column + 1 == columns.len() {
                    rim[(AROUND + AROUND / 2 - front) % AROUND]
                } else {
                    match latitude {
                        Latitude::Regular(value) => {
                            self.vertex(VertexKind::FanPoint, [value, *u, 0.0])
                        }
                        Latitude::Crest { side, base } => {
                            self.vertex(VertexKind::FanCrest, [side, f32::from(u8::from(base)), *u])
                        }
                    }
                };
                ring.push(id);
            }
            if let Some(previous) = rings.last() {
                for i in 0..ring.len() - 1 {
                    for triangle in [
                        [previous[i], ring[i], ring[i + 1]],
                        [previous[i], ring[i + 1], previous[i + 1]],
                    ] {
                        if triangle[0] != triangle[1]
                            && triangle[1] != triangle[2]
                            && triangle[2] != triangle[0]
                        {
                            self.indices.extend(triangle);
                        }
                    }
                }
            } else {
                for pair in ring.windows(2) {
                    self.indices.extend([first, pair[0], pair[1]]);
                }
            }
            rings.push(ring);
        }
        let last = rim[AROUND * 3 / 4];
        for pair in rings.last().expect("crown rings").windows(2) {
            self.indices.extend([pair[0], last, pair[1]]);
        }
        rim
    }

    /// The carrier as one shell with smooth return walls.
    pub fn shell(self, thickness: f32, extrusion: CoordExtrusion) -> CoordShell {
        CoordShell {
            coords: self.coords,
            indices: self.indices,
            boundary_normals: BoundaryNormals::Smooth,
            thickness,
            extrusion,
            values: [0.0; 4],
            mirrored: false,
            frame: 0,
            passes: 1,
            hinge: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crest_rows_never_pass_their_regular_neighbours() {
        let interval = PI / (AROUND / 2) as f32;
        assert!(0.12f32.asin() < interval);
        let rows = latitudes(true);
        assert_eq!(rows.len(), AROUND / 2 - 1 + 4);
        assert_eq!(
            rows[AROUND / 4 + 1],
            (Latitude::Regular(FRAC_PI_2), AROUND / 4)
        );
    }
}
