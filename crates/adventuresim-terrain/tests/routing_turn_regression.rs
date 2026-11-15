use std::collections::HashSet;

use adventuresim_terrain::{
    Cell, MAX_ROUTE_POINTS, RoutePlan, RoutePoint, RoutingGrid, Surface, TerrainSpan,
    TerrainWeights, astar,
};

const METRES_PER_CELL: u32 = 100;

struct RoadGrid {
    width: u32,
    height: u32,
    roads: HashSet<[u32; 2]>,
}

#[derive(Clone, Copy)]
enum RowProgress {
    Increasing,
    Decreasing,
}

impl RoadGrid {
    fn new(roads: &[[u32; 2]]) -> Self {
        let width = roads.iter().map(|cell| cell[0]).max().unwrap() + 1;
        let height = roads.iter().map(|cell| cell[1]).max().unwrap() + 1;
        Self {
            width,
            height,
            roads: roads.iter().copied().collect(),
        }
    }
}

impl RoutingGrid for RoadGrid {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn cell(&self, x: u32, y: u32) -> Option<Cell> {
        (x < self.width && y < self.height && self.roads.contains(&[x, y])).then_some(Cell {
            surface: Surface::Road,
            ..Cell::default()
        })
    }

    fn metres_per_cell(&self) -> u32 {
        METRES_PER_CELL
    }
}

fn point([x, y]: [u32; 2]) -> RoutePoint {
    RoutePoint {
        latitude: f64::from(y),
        longitude: f64::from(x),
    }
}

fn expected_points(path: &[[u32; 2]]) -> Vec<RoutePoint> {
    path.iter().copied().map(point).collect()
}

fn assert_feasible_path(grid: &RoadGrid, path: &[[u32; 2]], distance_m: u64, seconds: u64) {
    assert!(!path.is_empty());
    for &[x, y] in path {
        let cell = grid.cell(x, y).expect("a path cell must be present");
        assert_eq!(cell.surface, Surface::Road);
        assert_eq!(cell.elevation_m, 0);
    }
    let mut measured_distance = 0;
    let mut measured_seconds = 0;
    for pair in path.windows(2) {
        let [x, y] = pair[0];
        let [next_x, next_y] = pair[1];
        let dx = x.abs_diff(next_x);
        let dy = y.abs_diff(next_y);
        assert!(dx <= 1 && dy <= 1 && dx + dy > 0);
        if dx == 1 && dy == 1 {
            assert!(grid.cell(next_x, y).is_some());
            assert!(grid.cell(x, next_y).is_some());
            measured_distance += 141;
            measured_seconds += 102;
        } else {
            measured_distance += 100;
            measured_seconds += 72;
        }
    }
    assert_eq!(measured_distance, distance_m);
    assert_eq!(measured_seconds, seconds);
}

fn assert_route(grid: &RoadGrid, path: &[[u32; 2]], distance_m: u64, seconds: u64) -> RoutePlan {
    assert_feasible_path(grid, path, distance_m, seconds);
    let minutes = seconds.div_ceil(60);
    let [start_x, start_y] = *path.first().expect("a route has a start");
    let [goal_x, goal_y] = *path.last().expect("a route has a goal");
    let plan =
        astar(grid, (start_x, start_y), (goal_x, goal_y)).expect("the listed path is feasible");
    assert_eq!(plan.points.first(), Some(&point([start_x, start_y])));
    assert_eq!(plan.points.last(), Some(&point([goal_x, goal_y])));
    assert_eq!(plan.distance_m, distance_m);
    assert_eq!(plan.minutes, minutes);
    assert_eq!(
        plan.spans,
        vec![TerrainSpan {
            surface: Surface::Road,
            terrain: TerrainWeights {
                plains: 1_000,
                ..TerrainWeights::default()
            },
            training_multiplier_permille: 250,
            check_millirank: 0,
            start_minute: 0,
            duration_minutes: minutes,
        }]
    );
    plan
}

fn staircase(point_count: usize, row_progress: RowProgress) -> Vec<[u32; 2]> {
    assert!(point_count > 0);
    let mut path = Vec::with_capacity(point_count);
    let last_row = u32::try_from((point_count - 1) / 2).unwrap();
    let start_row = match row_progress {
        RowProgress::Increasing => 0,
        RowProgress::Decreasing => last_row,
    };
    let mut position = [0_u32, start_row];
    for index in 0..point_count {
        path.push(position);
        if index + 1 == point_count {
            break;
        }
        if index % 2 == 0 {
            position[0] += 1;
        } else {
            match row_progress {
                RowProgress::Increasing => position[1] += 1,
                RowProgress::Decreasing => position[1] -= 1,
            }
        }
    }
    path
}

fn assert_narrow_staircase(grid: &RoadGrid, path: &[[u32; 2]], row_progress: RowProgress) {
    assert!(!path.is_empty());
    assert_eq!(grid.roads.len(), path.len());
    let last_row = (path.len() - 1) / 2;
    for (index, &[x, y]) in path.iter().enumerate() {
        assert_eq!(usize::try_from(x).unwrap(), index.div_ceil(2));
        let expected_row = match row_progress {
            RowProgress::Increasing => index / 2,
            RowProgress::Decreasing => last_row - index / 2,
        };
        assert_eq!(usize::try_from(y).unwrap(), expected_row);
    }
    for triple in path.windows(3) {
        let [x0, y0] = triple[0];
        let [x2, y2] = triple[2];
        assert_eq!(x0.abs_diff(x2), 1);
        assert_eq!(y0.abs_diff(y2), 1);
        assert_ne!(grid.cell(x0, y2).is_some(), grid.cell(x2, y0).is_some());
    }
}

#[test]
fn unique_cardinal_turn_keeps_its_required_corner() {
    let path = [[0, 0], [1, 0], [1, 1]];
    let grid = RoadGrid::new(&path);
    assert!(grid.cell(0, 1).is_none());
    let plan = assert_route(&grid, &path, 200, 144);
    assert_eq!(plan.points, expected_points(&path));
}

#[test]
fn reversing_the_unique_cardinal_turn_keeps_its_corner() {
    let path = [[1, 1], [1, 0], [0, 0]];
    let grid = RoadGrid::new(&path);
    assert!(grid.cell(0, 1).is_none());
    let plan = assert_route(&grid, &path, 200, 144);
    assert_eq!(plan.points, expected_points(&path));
}

#[test]
fn mirrored_cardinal_turn_keeps_its_required_corner() {
    let path = [[0, 0], [0, 1], [1, 1]];
    let grid = RoadGrid::new(&path);
    assert!(grid.cell(1, 0).is_none());
    let plan = assert_route(&grid, &path, 200, 144);
    assert_eq!(plan.points, expected_points(&path));
}

#[test]
fn already_distinguished_cardinal_turn_remains_visible() {
    let path = [[0, 1], [1, 1], [1, 0]];
    let grid = RoadGrid::new(&path);
    assert!(grid.cell(0, 0).is_none());
    let plan = assert_route(&grid, &path, 200, 144);
    assert_eq!(plan.points, expected_points(&path));
}

#[test]
fn unique_fastest_cardinal_to_diagonal_route_keeps_its_turn() {
    let roads = [[0, 0], [1, 0], [2, 0], [1, 1], [2, 1]];
    let path = [[0, 0], [1, 0], [2, 1]];
    let grid = RoadGrid::new(&roads);
    assert!(grid.cell(0, 1).is_none());
    assert_feasible_path(&grid, &[[0, 0], [1, 0], [2, 0], [2, 1]], 300, 216);
    assert_feasible_path(&grid, &[[0, 0], [1, 0], [1, 1], [2, 1]], 300, 216);
    let plan = assert_route(&grid, &path, 241, 174);
    assert_eq!(plan.points, expected_points(&path));
}

#[test]
fn straight_cardinal_route_keeps_only_its_endpoints() {
    let path = [[0, 0], [1, 0], [2, 0]];
    let grid = RoadGrid::new(&path);
    let plan = assert_route(&grid, &path, 200, 144);
    assert_eq!(plan.points, expected_points(&[[0, 0], [2, 0]]));
}

#[test]
fn straight_diagonal_route_keeps_only_its_endpoints() {
    let roads = [
        [0, 0],
        [1, 0],
        [2, 0],
        [0, 1],
        [1, 1],
        [2, 1],
        [0, 2],
        [1, 2],
        [2, 2],
    ];
    let path = [[0, 0], [1, 1], [2, 2]];
    let grid = RoadGrid::new(&roads);
    let plan = assert_route(&grid, &path, 282, 204);
    assert_eq!(plan.points, expected_points(&[[0, 0], [2, 2]]));
}

#[test]
fn one_cell_route_has_one_point_and_no_measurement_span() {
    let grid = RoadGrid::new(&[[0, 0]]);
    assert_feasible_path(&grid, &[[0, 0]], 0, 0);
    let plan = astar(&grid, (0, 0), (0, 0)).expect("the cell is present");
    assert_eq!(plan.points, expected_points(&[[0, 0]]));
    assert_eq!(plan.distance_m, 0);
    assert_eq!(plan.minutes, 0);
    assert!(plan.spans.is_empty());
}

#[test]
fn exactly_the_point_cap_preserves_every_required_turn() {
    assert_eq!(MAX_ROUTE_POINTS, 512);
    let path = staircase(512, RowProgress::Increasing);
    let grid = RoadGrid::new(&path);
    assert_narrow_staircase(&grid, &path, RowProgress::Increasing);
    let plan = assert_route(&grid, &path, 51_100, 36_792);
    assert_eq!(plan.points.len(), 512);
    assert_eq!(plan.points, expected_points(&path));
}

#[test]
fn over_the_point_cap_preserves_endpoints_and_measurements() {
    assert_eq!(MAX_ROUTE_POINTS, 512);
    let path = staircase(513, RowProgress::Decreasing);
    let grid = RoadGrid::new(&path);
    assert_narrow_staircase(&grid, &path, RowProgress::Decreasing);
    let plan = assert_route(&grid, &path, 51_200, 36_864);
    assert!(plan.points.len() > 2);
    assert!(plan.points.len() <= MAX_ROUTE_POINTS);
}
