use adventuresim_terrain::{
    Cell, Error, RoutePoint, RoutingGrid, Surface, TerrainSkillProfile, astar_with_profile,
};
use sha2::{Digest, Sha256};

const ROUTE_ROWS: &str = include_str!("fixtures/routing_heuristic_rows.txt");
const ROUTE_ROWS_SHA256: &str = "54b5d16382c9c44deff1046d470cfa859818dcb0e60a35eb2ee65d71b4643e25";

struct FixtureGrid {
    width: u32,
    height: u32,
    metres: u32,
    cells: Vec<Cell>,
}

impl FixtureGrid {
    fn from_rows(rows: &[&str], metres: u32) -> Self {
        let height = u32::try_from(rows.len()).unwrap();
        let width = u32::try_from(rows.first().unwrap().len()).unwrap();
        let mut cells = Vec::new();
        for row in rows {
            assert_eq!(row.len(), usize::try_from(width).unwrap());
            for symbol in row.bytes() {
                let (surface, elevation_m) = match symbol {
                    b'R' => (Surface::Road, 0),
                    b'H' => (Surface::Road, 1),
                    b'S' => (Surface::Road, -2),
                    b'G' => (Surface::Road, 2),
                    b'.' => (Surface::Open, 0),
                    b'#' => (Surface::Water, 0),
                    _ => panic!("unknown fixture symbol: {symbol}"),
                };
                cells.push(Cell {
                    surface,
                    elevation_m,
                    ..Cell::default()
                });
            }
        }
        Self {
            width,
            height,
            metres,
            cells,
        }
    }

    fn roads(width: u32, height: u32, metres: u32) -> Self {
        let count = usize::try_from(u64::from(width) * u64::from(height)).unwrap();
        Self {
            width,
            height,
            metres,
            cells: vec![
                Cell {
                    surface: Surface::Road,
                    ..Cell::default()
                };
                count
            ],
        }
    }

    fn block(&mut self, x: u32, y: u32) {
        assert!(x < self.width && y < self.height);
        let index = usize::try_from(u64::from(y) * u64::from(self.width) + u64::from(x)).unwrap();
        self.cells[index].surface = Surface::Water;
    }
}

impl RoutingGrid for FixtureGrid {
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
        let index = usize::try_from(u64::from(y) * u64::from(self.width) + u64::from(x)).ok()?;
        self.cells.get(index).copied()
    }

    fn metres_per_cell(&self) -> u32 {
        self.metres
    }
}

fn plains_expert() -> TerrainSkillProfile {
    TerrainSkillProfile {
        plains: 5_000,
        ..TerrainSkillProfile::default()
    }
}

fn passable(cell: Cell) -> bool {
    cell.surface != Surface::Water || cell.crossing
}

fn verify_alternative(grid: &FixtureGrid, path: &[[u32; 2]], start: [u32; 2], goal: [u32; 2]) {
    assert_eq!(path.len(), 105);
    assert_eq!(path.first(), Some(&start));
    assert_eq!(path.last(), Some(&goal));

    let mut cardinal_flat = 0_u64;
    let mut cardinal_rise_two = 0_u64;
    let mut diagonal_flat = 0_u64;
    let mut diagonal_rise_two = 0_u64;
    for pair in path.windows(2) {
        let [x, y] = pair[0];
        let [next_x, next_y] = pair[1];
        let dx = x.abs_diff(next_x);
        let dy = y.abs_diff(next_y);
        assert!(dx <= 1 && dy <= 1 && dx + dy > 0);
        let from = grid.cell(x, y).expect("alternative source in bounds");
        let to = grid
            .cell(next_x, next_y)
            .expect("alternative destination in bounds");
        assert_eq!(from.surface, Surface::Road);
        assert!(passable(from) && passable(to));

        let diagonal = dx == 1 && dy == 1;
        if diagonal {
            assert!(grid.cell(next_x, y).is_some_and(passable));
            assert!(grid.cell(x, next_y).is_some_and(passable));
        }
        let rise = i32::from(to.elevation_m) - i32::from(from.elevation_m);
        match (diagonal, rise) {
            (false, 0) => cardinal_flat += 1,
            (false, 2) => cardinal_rise_two += 1,
            (true, 0) => diagonal_flat += 1,
            (true, 2) => diagonal_rise_two += 1,
            _ => panic!("unexpected alternative step: diagonal={diagonal}, rise={rise}"),
        }
    }
    assert_eq!(
        [
            cardinal_flat,
            cardinal_rise_two,
            diagonal_flat,
            diagonal_rise_two,
        ],
        [1, 1, 101, 1]
    );
    // These per-step seconds were independently established for this fixture;
    // this verifies the feasible path without calling a private cost helper.
    let seconds =
        cardinal_flat * 29 + cardinal_rise_two * 30 + diagonal_flat * 40 + diagonal_rise_two * 41;
    assert_eq!(seconds, 4_140);
}

#[test]
fn route_is_no_slower_than_the_saved_feasible_alternative() {
    assert_eq!(
        format!("{:x}", Sha256::digest(ROUTE_ROWS.as_bytes())),
        ROUTE_ROWS_SHA256
    );
    let rows: Vec<_> = ROUTE_ROWS.lines().collect();
    assert_eq!(rows.len(), 107);
    assert!(rows.iter().all(|row| row.len() == 103));
    let mut counts = [0_usize; 6];
    for symbol in rows.iter().flat_map(|row| row.bytes()) {
        let index = match symbol {
            b'#' => 0,
            b'.' => 1,
            b'R' => 2,
            b'H' => 3,
            b'S' => 4,
            b'G' => 5,
            _ => panic!("unknown fixture symbol: {symbol}"),
        };
        counts[index] += 1;
    }
    assert_eq!(counts, [10_403, 406, 209, 1, 1, 1]);

    let grid = FixtureGrid::from_rows(&rows, 59);
    assert_eq!(grid.cell(0, 2).unwrap().elevation_m, -2);
    assert_eq!(grid.cell(51, 55).unwrap().elevation_m, 1);
    assert_eq!(grid.cell(102, 104).unwrap().elevation_m, 2);
    assert!(grid.cell(103, 0).is_none());
    assert!(grid.cell(0, 107).is_none());
    let start = [0, 2];
    let goal = [102, 104];
    let alternative: Vec<_> = std::iter::once(start)
        .chain((1..=102).map(|coordinate| [coordinate, coordinate]))
        .chain([[102, 103], goal])
        .collect();
    verify_alternative(&grid, &alternative, start, goal);

    let [start_x, start_y] = start;
    let [goal_x, goal_y] = goal;
    let plan = astar_with_profile(&grid, (start_x, start_y), (goal_x, goal_y), plains_expert())
        .expect("the saved alternative is a feasible route");
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
    assert!(plan.distance_m > 0 && plan.minutes > 0);
    assert_eq!(
        plan.spans
            .iter()
            .map(|span| span.duration_minutes)
            .sum::<u64>(),
        plan.minutes
    );
    assert!(
        plan.minutes <= 69,
        "returned {} minutes although the saved path is feasible in 69 minutes",
        plan.minutes
    );
}

#[test]
fn flat_road_controls_preserve_distance_and_rounded_duration() {
    let grid = FixtureGrid::roads(3, 1, 59);
    let short = astar_with_profile(&grid, (0, 0), (2, 0), plains_expert()).unwrap();
    assert_eq!(short.distance_m, 118);
    assert_eq!(short.minutes, 1);
    let same_cell = astar_with_profile(&grid, (1, 0), (1, 0), plains_expert()).unwrap();
    assert_eq!(same_cell.distance_m, 0);
    assert_eq!(same_cell.minutes, 0);

    let diagonal_grid = FixtureGrid::roads(24, 24, 59);
    let diagonal = astar_with_profile(&diagonal_grid, (0, 0), (23, 23), plains_expert()).unwrap();
    assert_eq!(diagonal.distance_m, 1_909);
    assert_eq!(diagonal.minutes, 16);
}

#[test]
fn repeated_flat_road_steps_preserve_rounded_cost() {
    let line = FixtureGrid::roads(61, 1, 59);
    let cardinal = astar_with_profile(&line, (0, 0), (60, 0), plains_expert()).unwrap();
    assert_eq!(cardinal.distance_m, 3_540);
    assert_eq!(cardinal.minutes, 29);

    let square = FixtureGrid::roads(61, 61, 59);
    let diagonal = astar_with_profile(&square, (0, 0), (60, 60), plains_expert()).unwrap();
    assert_eq!(diagonal.distance_m, 4_980);
    assert_eq!(diagonal.minutes, 40);
}

#[test]
fn repeated_road_rises_preserve_rounded_cost() {
    let mut line = FixtureGrid::roads(61, 1, 59);
    for (x, cell) in line.cells.iter_mut().enumerate() {
        cell.elevation_m = i16::try_from(2 * x).unwrap();
    }
    let cardinal = astar_with_profile(&line, (0, 0), (60, 0), plains_expert()).unwrap();
    assert_eq!(cardinal.distance_m, 3_540);
    assert_eq!(cardinal.minutes, 30);

    let mut square = FixtureGrid::roads(61, 61, 59);
    for (y, row) in square.cells.as_chunks_mut::<61>().0.iter_mut().enumerate() {
        for cell in row {
            cell.elevation_m = i16::try_from(2 * y).unwrap();
        }
    }
    let diagonal = astar_with_profile(&square, (0, 0), (60, 60), plains_expert()).unwrap();
    assert_eq!(diagonal.distance_m, 4_980);
    assert_eq!(diagonal.minutes, 41);
}

#[test]
fn diagonals_cannot_cut_a_blocked_corner() {
    let mut grid = FixtureGrid::roads(2, 2, 59);
    let open = astar_with_profile(&grid, (0, 0), (1, 1), plains_expert()).unwrap();
    assert_eq!(open.distance_m, 83);
    assert_eq!(open.minutes, 1);
    grid.block(1, 0);
    let around_first = astar_with_profile(&grid, (0, 0), (1, 1), plains_expert()).unwrap();
    assert_eq!(around_first.distance_m, 118);
    assert_eq!(around_first.minutes, 1);

    let mut opposite = FixtureGrid::roads(2, 2, 59);
    opposite.block(0, 1);
    let around_second = astar_with_profile(&opposite, (0, 0), (1, 1), plains_expert()).unwrap();
    assert_eq!(around_second.distance_m, 118);
    assert_eq!(around_second.minutes, 1);

    grid.block(0, 1);
    assert!(matches!(
        astar_with_profile(&grid, (0, 0), (1, 1), plains_expert()),
        Err(Error::NoRoute)
    ));
}
