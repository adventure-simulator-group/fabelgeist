//! Canonical street connections and overlapping carriage passes.
use super::*;

const NODE_SNAP_PER_METRE: f32 = 1000.0;
const GEOMETRY_EPSILON: f32 = 0.001;
const PATH_STEP_METRES: f32 = 0.6;
const VEHICLE_PASSES: usize = 7;
const MIN_GAUGE_METRES: f32 = 1.1;
const GAUGE_VARIATION_METRES: f32 = 0.12;
const WHEEL_HALF_WIDTH_METRES: f32 = 0.13;
pub(super) const WHEEL_FEATHER_METRES: f32 = 0.16;
const WHEELBASE_METRES: f32 = 2.0;
const FRONTAGE_CLEARANCE_METRES: f32 = 0.35;
const TURN_REACH_WIDTHS: f32 = 2.5;
const TURN_CONTROL_REACH_FRACTION: f32 = 0.45;
const PASS_STRENGTHS: [f32; VEHICLE_PASSES] = [0.35, 0.5, 0.85, 0.45, 0.65, 1.0, 0.4];

pub(in super::super) struct TrafficNetwork {
    pub(super) roads: Vec<Road>,
    pub(super) markets: Vec<[Vec2; 4]>,
    pub(super) strokes: Vec<WheelStroke>,
    pub(super) tiles: BTreeMap<TrafficTile, Vec<usize>>,
    road_tiles: BTreeMap<TrafficTile, Vec<usize>>,
}

#[derive(Clone, Copy)]
struct Arm {
    direction: Vec2,
    length: f32,
    half_width: f32,
}

enum AxleTracks {
    Front,
    Both,
}

struct Junction {
    point: Vec2,
    arms: Vec<Arm>,
}

impl TrafficNetwork {
    pub(in super::super) fn new(streets: &[CityStreetPatch]) -> Self {
        let mut roads: BTreeMap<_, Road> = BTreeMap::new();
        let mut markets = Vec::new();
        for street in streets {
            match *street {
                CityStreetPatch::Corridor {
                    mut start_metres,
                    mut end_metres,
                    half_width_metres,
                    ..
                } => {
                    if node_key(end_metres) < node_key(start_metres) {
                        std::mem::swap(&mut start_metres, &mut end_metres);
                    }
                    roads
                        .entry((node_key(start_metres), node_key(end_metres)))
                        .and_modify(|road| road.half_width = road.half_width.max(half_width_metres))
                        .or_insert(Road {
                            start: start_metres,
                            end: end_metres,
                            half_width: half_width_metres,
                        });
                }
                CityStreetPatch::Market { corners_metres, .. } => markets.push(corners_metres),
            }
        }
        let mut network = Self {
            roads: roads.into_values().collect(),
            markets,
            strokes: Vec::new(),
            tiles: BTreeMap::new(),
            road_tiles: BTreeMap::new(),
        };
        for (index, road) in network.roads.iter().enumerate() {
            let padding = Vec2::splat(road.half_width);
            for tile in TrafficTile::covering(
                road.start.min(road.end) - padding,
                road.start.max(road.end) + padding,
            ) {
                network.road_tiles.entry(tile).or_default().push(index);
            }
        }
        for index in 0..network.roads.len() {
            network.straight(network.roads[index]);
        }
        for junction in network.junctions() {
            for a in 0..junction.arms.len() {
                for b in a + 1..junction.arms.len() {
                    network.turn(junction.point, junction.arms[a], junction.arms[b]);
                }
            }
        }
        for (index, stroke) in network.strokes.iter().enumerate() {
            let padding =
                Vec2::splat(stroke.half_width + WHEEL_FEATHER_METRES + 1.0 / TEXELS_PER_METRE);
            for tile in TrafficTile::covering(
                stroke.start.min(stroke.end) - padding,
                stroke.start.max(stroke.end) + padding,
            ) {
                network.tiles.entry(tile).or_default().push(index);
            }
        }
        network
    }

    pub(in super::super) fn stroke_count(&self) -> usize {
        self.strokes.len()
    }

    fn junctions(&self) -> Vec<Junction> {
        let mut nodes: BTreeMap<(i32, i32), Vec2> = BTreeMap::new();
        for road in &self.roads {
            for point in [road.start, road.end] {
                nodes.insert(node_key(point), point);
            }
        }
        for (index, a) in self.roads.iter().enumerate() {
            for b in &self.roads[index + 1..] {
                let da = a.end - a.start;
                let db = b.end - b.start;
                let cross = da.perp_dot(db);
                if cross.abs() < GEOMETRY_EPSILON {
                    continue;
                }
                let delta = b.start - a.start;
                let t = delta.perp_dot(db) / cross;
                let u = delta.perp_dot(da) / cross;
                if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                    let point = a.start + da * t;
                    nodes.entry(node_key(point)).or_insert(point);
                }
            }
        }
        nodes
            .into_values()
            .filter_map(|point| {
                let mut arms: Vec<Arm> = Vec::new();
                for road in &self.roads {
                    let local = road.coordinates(point);
                    if local.x.abs() > GEOMETRY_EPSILON
                        || local.y < -GEOMETRY_EPSILON
                        || local.y > road.start.distance(road.end) + GEOMETRY_EPSILON
                    {
                        continue;
                    }
                    for end in [road.start, road.end] {
                        let delta = end - point;
                        let length = delta.length();
                        if length <= GEOMETRY_EPSILON {
                            continue;
                        }
                        let direction = delta / length;
                        if !arms
                            .iter()
                            .any(|arm| arm.direction.dot(direction) > 1.0 - GEOMETRY_EPSILON)
                        {
                            arms.push(Arm {
                                direction,
                                length,
                                half_width: road.half_width,
                            });
                        }
                    }
                }
                (arms.len() >= 2).then_some(Junction { point, arms })
            })
            .collect()
    }

    fn straight(&mut self, road: Road) {
        let length = road.start.distance(road.end);
        let direction = (road.end - road.start) / length;
        let normal = Vec2::new(-direction.y, direction.x);
        let steps = (length / PATH_STEP_METRES).ceil() as usize;
        for pass in 0..VEHICLE_PASSES {
            let (gauge, offset) = pass_dimensions(pass, road.half_width);
            let points = (0..=steps)
                .map(|step| {
                    let t = step as f32 / steps as f32;
                    let wander = (t * std::f32::consts::TAU).sin() * (pass as f32 + 1.0) * 0.025;
                    road.start.lerp(road.end, t) + normal * (offset + wander)
                })
                .collect::<Vec<_>>();
            self.wheels(&points, gauge, pass, AxleTracks::Front);
        }
    }

    fn turn(&mut self, node: Vec2, a: Arm, b: Arm) {
        let alignment = a.direction.dot(b.direction);
        if alignment > 0.98 || alignment < -1.0 + GEOMETRY_EPSILON {
            return;
        }
        let width = a.half_width.min(b.half_width);
        let reach = (width * TURN_REACH_WIDTHS)
            .min(a.length * 0.45)
            .min(b.length * 0.45);
        let controls = [
            node + a.direction * reach,
            node + a.direction * reach * TURN_CONTROL_REACH_FRACTION,
            node + b.direction * reach * TURN_CONTROL_REACH_FRACTION,
            node + b.direction * reach,
        ];
        let steps = ((reach * 2.0 / PATH_STEP_METRES).ceil() as usize).max(4);
        for pass in 0..VEHICLE_PASSES {
            let (gauge, offset) = pass_dimensions(pass, width);
            let points = (0..=steps)
                .map(|step| {
                    let t = step as f32 / steps as f32;
                    let position = bezier(controls, t);
                    let tangent = ((controls[1] - controls[0]) * (1.0 - t).powi(2)
                        + (controls[2] - controls[1]) * 2.0 * (1.0 - t) * t
                        + (controls[3] - controls[2]) * t * t)
                        .normalize();
                    position + Vec2::new(-tangent.y, tangent.x) * offset
                })
                .collect::<Vec<_>>();
            let lead_steps = (WHEELBASE_METRES * 2.0 / PATH_STEP_METRES).ceil() as usize;
            let mut extended = Vec::new();
            let incoming = (points[1] - points[0]).normalize();
            let outgoing = (points[points.len() - 1] - points[points.len() - 2]).normalize();
            for step in (1..=lead_steps).rev() {
                extended.push(points[0] - incoming * step as f32 * PATH_STEP_METRES);
            }
            extended.extend_from_slice(&points);
            for step in 1..=lead_steps {
                extended.push(points[points.len() - 1] + outgoing * step as f32 * PATH_STEP_METRES);
            }
            self.wheels(&extended, gauge, pass, AxleTracks::Both);
            extended.reverse();
            self.wheels(&extended, gauge, pass, AxleTracks::Both);
        }
    }

    fn wheels(&mut self, points: &[Vec2], gauge: f32, pass: usize, axles: AxleTracks) {
        let mut candidate = Vec::new();
        let mut rear = points[0] - (points[1] - points[0]).normalize() * WHEELBASE_METRES;
        let mut rear_points = vec![rear];
        for pair in points.windows(2) {
            rear = pair[1] - (pair[1] - rear).normalize() * WHEELBASE_METRES;
            rear_points.push(rear);
        }
        for axle in std::iter::once(points)
            .chain(matches!(axles, AxleTracks::Both).then_some(rear_points.as_slice()))
        {
            for pair in axle.windows(2) {
                let delta = pair[1] - pair[0];
                if delta.length_squared() < GEOMETRY_EPSILON {
                    continue;
                }
                let normal = Vec2::new(-delta.y, delta.x).normalize();
                for side in [-1.0, 1.0] {
                    let shift = normal * gauge * 0.5 * side;
                    let stroke = WheelStroke {
                        start: pair[0] + shift,
                        end: pair[1] + shift,
                        half_width: WHEEL_HALF_WIDTH_METRES + pass as f32 * 0.013,
                        strength: PASS_STRENGTHS[pass],
                    };
                    // Check the full feathered wheel envelope against the visible
                    // rectangle union, including halfway through each segment.
                    if [stroke.start, stroke.end, stroke.start.lerp(stroke.end, 0.5)]
                        .into_iter()
                        .any(|p| {
                            !union_support(p, stroke.half_width + WHEEL_FEATHER_METRES, |point| {
                                self.clearance(point)
                            })
                        })
                    {
                        if matches!(axles, AxleTracks::Both) {
                            return;
                        }
                        continue;
                    }
                    candidate.push(stroke);
                }
            }
        }
        self.strokes.extend(candidate);
    }

    pub(super) fn clearance(&self, point: Vec2) -> f32 {
        self.road_tiles
            .get(&TrafficTile::at(point))
            .into_iter()
            .flatten()
            .map(|&index| self.roads[index].clearance(point))
            .chain(
                self.markets
                    .iter()
                    .map(|&corners| quad_clearance(corners, point)),
            )
            .fold(f32::NEG_INFINITY, f32::max)
    }
}

fn node_key(point: Vec2) -> (i32, i32) {
    let key = (point * NODE_SNAP_PER_METRE).round().as_ivec2();
    (key.x, key.y)
}

fn pass_dimensions(pass: usize, half_width: f32) -> (f32, f32) {
    let gauge = (MIN_GAUGE_METRES + pass as f32 * GAUGE_VARIATION_METRES).min(half_width * 0.85);
    let usable = (half_width * 0.68 - gauge * 0.5 - FRONTAGE_CLEARANCE_METRES).max(0.0);
    let offset = (pass as f32 / (VEHICLE_PASSES - 1) as f32 * 2.0 - 1.0) * usable;
    (gauge, offset)
}

fn bezier(p: [Vec2; 4], t: f32) -> Vec2 {
    p[0] * (1.0 - t).powi(3)
        + p[1] * 3.0 * (1.0 - t).powi(2) * t
        + p[2] * 3.0 * (1.0 - t) * t * t
        + p[3] * t.powi(3)
}
