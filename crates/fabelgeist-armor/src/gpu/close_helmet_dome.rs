//! The helmet bowl as the close helmet and the mail coif use it: a plain dome
//! of latitude rings, or a crown fanned from the temples when it is fluted or
//! crested.
//!
//! Which vertices exist and how they join is decided here from the design;
//! [`DOME`] evaluates each vertex on the device from the wearer's radii.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use super::coord_topology::CoordTopology;
use crate::HelmetCrown;

/// Vertices around a helmet rim.
pub(crate) const AROUND: usize = 48;
const DOME_RINGS: usize = 12;
/// Evenly spaced columns across an unfluted fan crown.
const FAN_COLUMNS: usize = 40;

/// Kinds of dome vertex, in a coordinate's fourth float.
pub(crate) const DOME_POLE: f32 = 0.0;
pub(crate) const DOME_RING: f32 = 1.0;
pub(crate) const FAN_RIM: f32 = 2.0;
pub(crate) const FAN_CHART: f32 = 3.0;

/// A fan crown's latitudes: the regular rows, and for a crest the four
/// latitudes at the crest's edges, whose angles depend on the wearer's width.
#[derive(Clone, Copy)]
enum Latitude {
    Row(usize),
    /// Below (-1) or above (+1) the midline, at the crest's top (`false`) or
    /// base (`true`) width.
    CrestEdge {
        side: i8,
        base: bool,
    },
}

impl Latitude {
    /// The coordinate's latitude slot: an authored angle, or which crest edge.
    fn coord(self) -> [f32; 2] {
        match self {
            Self::Row(row) => [row as f32 / (AROUND / 2) as f32 * PI, 0.0],
            Self::CrestEdge { side, base } => [0.0, f32::from(side) * if base { 2.0 } else { 1.0 }],
        }
    }

    fn rim_row(self) -> usize {
        match self {
            Self::Row(row) => row,
            Self::CrestEdge { .. } => AROUND / 4,
        }
    }
}

/// A plain dome of latitude rings closed by a pole; returns its rim.
pub(crate) fn full_dome(surface: &mut CoordTopology) -> Vec<u32> {
    let angles = (0..AROUND)
        .map(|i| i as f32 / AROUND as f32 * TAU)
        .collect::<Vec<_>>();
    let pole = surface.vertex([0.0, 0.0, 0.0, DOME_POLE]);
    let mut previous = Vec::new();
    for row in 1..=DOME_RINGS {
        let latitude = row as f32 / DOME_RINGS as f32 * FRAC_PI_2;
        let ring = angles
            .iter()
            .map(|&angle| surface.vertex([latitude, angle, 0.0, DOME_RING]))
            .collect::<Vec<_>>();
        if previous.is_empty() {
            for i in 0..angles.len() {
                surface
                    .indices
                    .extend([pole, ring[i], ring[(i + 1) % angles.len()]]);
            }
        } else {
            surface.connect(&previous, &ring, true);
        }
        previous = ring;
    }
    previous
}

/// The styled bowl: fanned when fluted or crested, a plain dome otherwise.
pub(crate) fn styled_dome(
    surface: &mut CoordTopology,
    style: &HelmetCrown,
    crest: f32,
) -> Vec<u32> {
    let first_index = surface.indices.len();
    let fan = style.fluting.is_some() || crest > 0.0;
    if !fan {
        return full_dome(surface);
    }
    let rim = fan_dome(surface, style, crest);
    for face in surface.indices[first_index..].as_chunks_mut::<3>().0 {
        face.swap(1, 2);
    }
    rim
}

fn fan_dome(surface: &mut CoordTopology, style: &HelmetCrown, crest: f32) -> Vec<u32> {
    let columns = style.fluting.as_ref().map_or_else(
        || {
            (0..=FAN_COLUMNS)
                .map(|i| i as f32 / FAN_COLUMNS as f32)
                .collect()
        },
        |p| p.columns(FAN_COLUMNS),
    );
    let mut latitudes = (1..AROUND / 2).map(Latitude::Row).collect::<Vec<_>>();
    if crest > 0.0 {
        // A crest's edge angles stay within the two regular intervals beside
        // the midline for every head, so their order is fixed.
        let middle = AROUND / 4;
        latitudes.splice(
            middle - 1..middle - 1,
            [
                Latitude::CrestEdge {
                    side: -1,
                    base: true,
                },
                Latitude::CrestEdge {
                    side: -1,
                    base: false,
                },
            ],
        );
        latitudes.splice(
            middle + 2..middle + 2,
            [
                Latitude::CrestEdge {
                    side: 1,
                    base: false,
                },
                Latitude::CrestEdge {
                    side: 1,
                    base: true,
                },
            ],
        );
    }
    let rim = (0..AROUND)
        .map(|i| {
            let angle = i as f32 / AROUND as f32 * 2.0 * PI;
            surface.vertex([angle, 0.0, 0.0, FAN_RIM])
        })
        .collect::<Vec<_>>();
    let first = rim[AROUND / 4];
    let mut rings: Vec<Vec<u32>> = Vec::new();
    for latitude in latitudes {
        let [value, edge] = latitude.coord();
        let front = (AROUND + AROUND / 4 - latitude.rim_row()) % AROUND;
        let ring = columns
            .iter()
            .enumerate()
            .map(|(column, u)| {
                if column == 0 {
                    rim[front]
                } else if column + 1 == columns.len() {
                    rim[(AROUND + AROUND / 2 - front) % AROUND]
                } else {
                    surface.vertex([value, *u, edge, FAN_CHART])
                }
            })
            .collect::<Vec<_>>();
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
                        surface.indices.extend(triangle);
                    }
                }
            }
        } else {
            for pair in ring.windows(2) {
                surface.indices.extend([first, pair[0], pair[1]]);
            }
        }
        rings.push(ring);
    }
    let last = rim[AROUND * 3 / 4];
    for pair in rings.last().expect("crown rings").windows(2) {
        surface.indices.extend([pair[0], last, pair[1]]);
    }
    rim
}

/// The crown's flute pattern as eight design floats: count, width, depth,
/// spread, lower spread, start, end, fade; all zero when unfluted.
pub(crate) fn flute_words(fluting: Option<&crate::PlateFluting>) -> [f32; 8] {
    fluting.map_or([0.0; 8], |f| {
        [
            f32::from(f.count.0),
            f.width.unit(),
            f.depth.metres(),
            f.spread.unit(),
            f.lower_spread.unit(),
            f.start.unit(),
            f.end.unit(),
            f.fade.unit(),
        ]
    })
}

/// Dome vertices on the device. The including shader supplies `design` and
/// `MATH`; a `Dome` carries the wearer's radii and the style.
///
/// `dome_vertex` returns the bowl point before any family reshapes it, and its
/// relief: the crown's flutes plus its formed medial ridge.
pub(crate) const DOME: &str = r#"
struct Dome {
    radii: vec3<f32>,
    brow: f32,
    // The ring exponent of a plain dome; the fan's fullness otherwise.
    fullness: f32,
    ridge: f32,
    crest: f32,
    // Design offset of the flute words, or no flutes.
    fluted: bool,
    flute: u32,
};

fn dome_smooth(t: f32) -> f32 {
    return smoothstep_clamped(t);
}

// `PlateFluting::fan_coordinate`, reading the flute words from the design.
fn dome_flute_fan(d: Dome, u: f32, v: f32) -> f32 {
    let lateral = 2.0 * u - 1.0;
    let span = design[d.flute + 3u];
    let lower = design[d.flute + 4u];
    let fan = lower + (1.0 - lower) * dome_smooth(v);
    let absolute = abs(lateral);
    var mapped: f32;
    if (absolute <= span) {
        mapped = absolute * fan;
    } else {
        mapped = span * fan + (absolute - span) * (1.0 - span * fan) / (1.0 - span);
    }
    return (1.0 + sign(lateral) * mapped) * 0.5;
}

// `PlateFluting::relief`.
fn dome_flute_relief(d: Dome, u: f32, v: f32) -> f32 {
    let count = design[d.flute];
    let pitch = design[d.flute + 3u] / count;
    let start = (1.0 - design[d.flute + 3u]) * 0.5;
    let slot = floor((u - start) / pitch);
    if (slot < 0.0 || slot >= count) {
        return 0.0;
    }
    let center = start + (slot + 0.5) * pitch;
    let distance = abs(u - center) / (pitch * design[d.flute + 1u] * 0.5);
    if (distance >= 1.0) {
        return 0.0;
    }
    let fade_length = design[d.flute + 7u];
    let fade = dome_smooth((v - design[d.flute + 5u]) / fade_length)
        * dome_smooth((design[d.flute + 6u] - v) / fade_length);
    return design[d.flute + 2u] * fade * (1.0 + cos(PI * distance)) * 0.5;
}

fn dome_crest_half_width(d: Dome) -> f32 {
    return min(d.radii.x * 0.06, 0.005);
}

fn dome_crest_base_width(d: Dome) -> f32 {
    return min(d.radii.x * 0.12, 0.010);
}

// A fan latitude: authored, or at a crest edge whose angle the width sets.
fn dome_latitude(d: Dome, value: f32, edge: f32) -> f32 {
    if (edge == 0.0) {
        return value;
    }
    var width = dome_crest_half_width(d);
    if (abs(edge) > 1.5) {
        width = dome_crest_base_width(d);
    }
    let angle = asin(width / d.radii.x);
    if (edge < 0.0) {
        return FRAC_PI_2 - angle;
    }
    return FRAC_PI_2 + angle;
}

// A crown point at a latitude and an angle over the head: the bowl, its sides
// filled out by the design's fullness, with the crest raised from its middle.
fn dome_chart_point(d: Dome, latitude: f32, angle: f32) -> vec3<f32> {
    let y = sin(latitude) * sin(angle);
    let radial = sqrt(max(1.0 - y * y, 0.0));
    var fullness = 1.0;
    if (radial > 1e-6) {
        fullness = pow(radial, d.fullness - 1.0);
    }
    let x = d.radii.x * cos(latitude);
    var across = 0.0;
    if (d.crest > 0.0) {
        let base = dome_crest_base_width(d);
        across = clamp((base - abs(x)) / (base - dome_crest_half_width(d)), 0.0, 1.0);
    }
    let crest_phase = clamp((angle / PI - 0.08) / 0.84, 0.0, 1.0);
    let wave = max(sin(PI * crest_phase), 0.0);
    var rise = 0.0;
    if (wave > 0.0) {
        rise = pow(wave, 0.8);
    }
    let crest = d.crest * across * rise;
    return vec3<f32>(
        x * (fullness + (1.0 - fullness) * across * rise),
        d.brow + (d.radii.y - d.brow) * y + crest,
        d.radii.z * sin(latitude) * cos(angle) * fullness,
    );
}

fn dome_vertex(d: Dome, coord: vec4<f32>) -> ShellVertex {
    var p: vec3<f32>;
    var relief = 0.0;
    let kind = u32(coord.w);
    if (kind == 0u) {
        p = vec3<f32>(0.0, d.radii.y, 0.0);
    } else if (kind == 1u) {
        let reach = pow(sin(coord.x), d.fullness);
        p = vec3<f32>(
            d.radii.x * reach * sin(coord.y),
            d.brow + (d.radii.y - d.brow) * cos(coord.x),
            d.radii.z * reach * cos(coord.y),
        );
    } else if (kind == 2u) {
        p = vec3<f32>(d.radii.x * sin(coord.x), d.brow, d.radii.z * cos(coord.x));
    } else {
        let latitude = dome_latitude(d, coord.x, coord.z);
        let v = sin(latitude);
        var fanned = coord.y;
        if (d.fluted) {
            fanned = dome_flute_fan(d, coord.y, v);
            relief = dome_flute_relief(d, coord.y, v);
        }
        p = dome_chart_point(d, latitude, fanned * PI);
    }
    let rise = clamp((p.y - d.brow) / (d.radii.y - d.brow), 0.0, 1.0);
    let medial = max(1.0 - abs(p.x / (d.radii.x * 0.30)), 0.0);
    relief = relief + d.ridge * pow2(medial) * rise;
    return ShellVertex(p, relief);
}
"#;
