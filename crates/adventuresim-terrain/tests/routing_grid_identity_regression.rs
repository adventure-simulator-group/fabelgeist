use adventuresim_terrain::{
    Cell, Error, RoutePlan, RoutePoint, RoutingGrid, RoutingWeather, Surface, TerrainSkillProfile,
    astar, astar_with_profile, astar_with_profile_and_weather,
};

struct FixtureGrid<'a> {
    width: u32,
    height: u32,
    metres_per_cell: u32,
    roads: Roads<'a>,
}

enum Roads<'a> {
    Listed(&'a [[u32; 2]]),
    All,
}

impl RoutingGrid for FixtureGrid<'_> {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn cell(&self, x: u32, y: u32) -> Option<Cell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let present = match &self.roads {
            Roads::Listed(cells) => cells.contains(&[x, y]),
            Roads::All => true,
        };
        present.then_some(Cell {
            surface: Surface::Road,
            ..Cell::default()
        })
    }

    fn metres_per_cell(&self) -> u32 {
        self.metres_per_cell
    }
}

fn assert_feasible_path(
    grid: &FixtureGrid<'_>,
    path: &[[u32; 2]],
    expected_distance: u64,
    expected_seconds: u64,
) {
    assert!(!path.is_empty());
    for &[x, y] in path {
        let cell = grid.cell(x, y).expect("path cell must be present");
        assert_eq!(cell.surface, Surface::Road);
        assert_eq!(cell.elevation_m, 0);
    }
    let mut distance = 0_u64;
    let mut seconds = 0_u64;
    for pair in path.windows(2) {
        let [x, y] = pair[0];
        let [next_x, next_y] = pair[1];
        let dx = x.abs_diff(next_x);
        let dy = y.abs_diff(next_y);
        assert!(dx <= 1 && dy <= 1 && dx + dy > 0);
        let diagonal = dx == 1 && dy == 1;
        if diagonal {
            assert!(
                grid.cell(next_x, y)
                    .is_some_and(|cell| cell.surface == Surface::Road)
            );
            assert!(
                grid.cell(x, next_y)
                    .is_some_and(|cell| cell.surface == Surface::Road)
            );
        }
        // These fixture costs are also checked through the public routing API
        // on repeated flat-road paths in `repeated_flat_road_cost_controls`.
        let (edge_distance, edge_seconds) = match (grid.metres_per_cell, diagonal) {
            (30, false) => (30, 22),
            (30, true) => (42, 31),
            (100, false) => (100, 72),
            (100, true) => (141, 102),
            _ => panic!("unexpected fixture spacing"),
        };
        distance += edge_distance;
        seconds += edge_seconds;
    }
    assert_eq!(distance, expected_distance);
    assert_eq!(seconds, expected_seconds);
}

fn assert_route(
    grid: &FixtureGrid<'_>,
    path: &[[u32; 2]],
    distance: u64,
    seconds: u64,
    minutes: u64,
) -> RoutePlan {
    assert_feasible_path(grid, path, distance, seconds);
    assert_eq!(seconds.div_ceil(60), minutes);
    let [start_x, start_y] = *path.first().expect("a path has a start");
    let [goal_x, goal_y] = *path.last().expect("a path has a goal");
    let plan =
        astar(grid, (start_x, start_y), (goal_x, goal_y)).expect("a listed path is feasible");
    assert_eq!(
        plan.points.first(),
        Some(&RoutePoint {
            latitude: f64::from(start_y),
            longitude: f64::from(start_x),
        })
    );
    assert_eq!(
        plan.points.last(),
        Some(&RoutePoint {
            latitude: f64::from(goal_y),
            longitude: f64::from(goal_x),
        })
    );
    assert_eq!(plan.distance_m, distance);
    assert_eq!(plan.minutes, minutes);
    assert_eq!(plan.spans.len(), 1);
    assert_eq!(plan.spans[0].surface, Surface::Road);
    assert_eq!(plan.spans[0].start_minute, 0);
    assert_eq!(plan.spans[0].duration_minutes, minutes);
    plan
}

#[test]
fn repeated_flat_road_cost_controls() {
    let thirty_metre_line = FixtureGrid {
        width: 61,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::All,
    };
    let line: Vec<_> = (0_u32..=60).map(|x| [x, 0]).collect();
    assert_route(&thirty_metre_line, &line, 1_800, 1_320, 22);
    let thirty_metre_square = FixtureGrid {
        width: 61,
        height: 61,
        metres_per_cell: 30,
        roads: Roads::All,
    };
    let diagonal: Vec<_> = (0_u32..=60)
        .map(|coordinate| [coordinate, coordinate])
        .collect();
    assert_route(&thirty_metre_square, &diagonal, 2_520, 1_860, 31);
    let hundred_metre_line = FixtureGrid {
        width: 61,
        height: 1,
        metres_per_cell: 100,
        roads: Roads::All,
    };
    assert_route(&hundred_metre_line, &line, 6_000, 4_320, 72);
    let hundred_metre_square = FixtureGrid {
        width: 61,
        height: 61,
        metres_per_cell: 100,
        roads: Roads::All,
    };
    assert_route(&hundred_metre_square, &diagonal, 8_460, 6_120, 102);
}

#[test]
fn row_product_does_not_alias_the_start() {
    let cells = [[0, 0], [0, 1], [0, 2]];
    let grid = FixtureGrid {
        width: 1_u32 << 31,
        height: 3,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &cells, 60, 44, 1);
}

#[test]
fn row_offset_addition_does_not_alias_the_start() {
    let cells = [[0, 0], [1, 0], [0, 1], [1, 1]];
    let grid = FixtureGrid {
        width: u32::MAX,
        height: 2,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &[[0, 0], [1, 1]], 42, 31, 1);
}

#[test]
fn crosses_signed_x_boundary_forward() {
    let cells = [[i32::MAX as u32, 0], [i32::MAX as u32 + 1, 0]];
    let grid = FixtureGrid {
        width: i32::MAX as u32 + 2,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &cells, 30, 22, 1);
}

#[test]
fn crosses_signed_x_boundary_backward() {
    let cells = [[i32::MAX as u32, 0], [i32::MAX as u32 + 1, 0]];
    let grid = FixtureGrid {
        width: i32::MAX as u32 + 2,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &[cells[1], cells[0]], 30, 22, 1);
}

#[test]
fn crosses_signed_y_boundary_forward() {
    let cells = [[0, i32::MAX as u32], [0, i32::MAX as u32 + 1]];
    let grid = FixtureGrid {
        width: 1,
        height: i32::MAX as u32 + 2,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &cells, 30, 22, 1);
}

#[test]
fn crosses_signed_y_boundary_backward() {
    let cells = [[0, i32::MAX as u32], [0, i32::MAX as u32 + 1]];
    let grid = FixtureGrid {
        width: 1,
        height: i32::MAX as u32 + 2,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &[cells[1], cells[0]], 30, 22, 1);
}

#[test]
fn routes_near_unsigned_x_limit() {
    let cells = [[u32::MAX - 2, 0], [u32::MAX - 1, 0]];
    let grid = FixtureGrid {
        width: u32::MAX,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &cells, 30, 22, 1);
}

#[test]
fn routes_near_unsigned_y_limit() {
    let cells = [[0, u32::MAX - 2], [0, u32::MAX - 1]];
    let grid = FixtureGrid {
        width: 1,
        height: u32::MAX,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &cells, 30, 22, 1);
}

#[test]
fn routes_backward_near_unsigned_x_limit() {
    let cells = [[u32::MAX - 2, 0], [u32::MAX - 1, 0]];
    let grid = FixtureGrid {
        width: u32::MAX,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &[cells[1], cells[0]], 30, 22, 1);
}

#[test]
fn routes_backward_near_unsigned_y_limit() {
    let cells = [[0, u32::MAX - 2], [0, u32::MAX - 1]];
    let grid = FixtureGrid {
        width: 1,
        height: u32::MAX,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &[cells[1], cells[0]], 30, 22, 1);
}

#[test]
fn routes_at_exact_current_ordinal_area_limit() {
    let cells = [[65_534, 65_535], [65_535, 65_535]];
    let grid = FixtureGrid {
        width: 65_536,
        height: 65_536,
        metres_per_cell: 30,
        roads: Roads::Listed(&cells),
    };
    assert_route(&grid, &cells, 30, 22, 1);
}

#[test]
fn equal_cost_paths_keep_row_first_tie_order() {
    // RRRR / RS.R / R.GR / RRRR; dots are absent, and S/G are roads.
    let cells = [
        [0, 0],
        [1, 0],
        [2, 0],
        [3, 0],
        [0, 1],
        [1, 1],
        [3, 1],
        [0, 2],
        [2, 2],
        [3, 2],
        [0, 3],
        [1, 3],
        [2, 3],
        [3, 3],
    ];
    let grid = FixtureGrid {
        width: 4,
        height: 4,
        metres_per_cell: 100,
        roads: Roads::Listed(&cells),
    };
    let top = [[1, 1], [1, 0], [2, 0], [3, 0], [3, 1], [3, 2], [2, 2]];
    let bottom = [[1, 1], [0, 1], [0, 2], [0, 3], [1, 3], [2, 3], [2, 2]];
    assert_feasible_path(&grid, &bottom, 600, 432);
    let plan = assert_route(&grid, &top, 600, 432, 8);
    // Retained points need not enumerate every traversed turn. This fixture's
    // source-derived tie model predicts this point only for row-first order.
    assert!(plan.points.contains(&RoutePoint {
        latitude: 0.0,
        longitude: 1.0
    }));
}

#[test]
fn invalid_profile_precedes_out_of_bounds() {
    let grid = FixtureGrid {
        width: 1,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::All,
    };
    let profile = TerrainSkillProfile {
        plains: TerrainSkillProfile::MAX + 1,
        ..TerrainSkillProfile::default()
    };
    assert!(matches!(
        astar_with_profile(&grid, (0, 0), (1, 0), profile),
        Err(Error::Validation(_))
    ));
}

#[test]
fn invalid_weather_precedes_out_of_bounds() {
    let grid = FixtureGrid {
        width: 1,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::All,
    };
    let weather = RoutingWeather {
        snow_check_millirank: TerrainSkillProfile::MAX + 1,
        ..RoutingWeather::default()
    };
    assert!(matches!(
        astar_with_profile_and_weather(
            &grid,
            (0, 0),
            (1, 0),
            TerrainSkillProfile::default(),
            weather
        ),
        Err(Error::Validation(_))
    ));
}

#[test]
fn valid_inputs_report_out_of_bounds() {
    let grid = FixtureGrid {
        width: 1,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::All,
    };
    assert!(matches!(
        astar_with_profile(&grid, (0, 0), (1, 0), TerrainSkillProfile::default()),
        Err(Error::NoRoute)
    ));
    assert!(matches!(
        astar_with_profile_and_weather(
            &grid,
            (0, 0),
            (1, 0),
            TerrainSkillProfile::default(),
            RoutingWeather::default()
        ),
        Err(Error::NoRoute)
    ));
}

#[test]
fn valid_inputs_reject_out_of_bounds_start() {
    let grid = FixtureGrid {
        width: 1,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::All,
    };
    assert!(matches!(
        astar_with_profile_and_weather(
            &grid,
            (1, 0),
            (0, 0),
            TerrainSkillProfile::default(),
            RoutingWeather::default()
        ),
        Err(Error::NoRoute)
    ));
}

#[test]
fn valid_inputs_reject_out_of_bounds_goal_y() {
    let grid = FixtureGrid {
        width: 1,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::All,
    };
    assert!(matches!(
        astar_with_profile_and_weather(
            &grid,
            (0, 0),
            (0, 1),
            TerrainSkillProfile::default(),
            RoutingWeather::default()
        ),
        Err(Error::NoRoute)
    ));
}

#[test]
fn valid_inputs_reject_zero_width() {
    let grid = FixtureGrid {
        width: 0,
        height: 1,
        metres_per_cell: 30,
        roads: Roads::All,
    };
    assert!(matches!(
        astar_with_profile_and_weather(
            &grid,
            (0, 0),
            (0, 0),
            TerrainSkillProfile::default(),
            RoutingWeather::default()
        ),
        Err(Error::NoRoute)
    ));
}

#[test]
fn valid_inputs_reject_zero_height() {
    let grid = FixtureGrid {
        width: 1,
        height: 0,
        metres_per_cell: 30,
        roads: Roads::All,
    };
    assert!(matches!(
        astar_with_profile_and_weather(
            &grid,
            (0, 0),
            (0, 0),
            TerrainSkillProfile::default(),
            RoutingWeather::default()
        ),
        Err(Error::NoRoute)
    ));
}
