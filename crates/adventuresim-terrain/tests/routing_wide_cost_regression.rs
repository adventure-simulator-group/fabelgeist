use adventuresim_terrain::{
    Cell, MAX_TERRAIN_SPANS, RoutePlan, RoutePoint, RoutingGrid, RoutingWeather, Surface,
    TerrainSkillProfile, TerrainSpan, TerrainWeights, astar_with_profile_and_weather,
};

const ISOLATED_OPEN_COUNT: u32 = 300;
const ISOLATED_OPEN_SPACING: u32 = 760;
const FORK_END: u32 = 228_000;
const SQUARE_START: u32 = 341_042;
const SQUARE_SIDE: u32 = 650;

struct ForkGrid;

struct PriorityGrid;

impl RoutingGrid for ForkGrid {
    fn width(&self) -> u32 {
        FORK_END + 3
    }

    fn height(&self) -> u32 {
        3
    }

    fn cell(&self, x: u32, y: u32) -> Option<Cell> {
        match (x, y) {
            (x, 1) if x <= FORK_END => Some(stem_cell(x)),
            (x, 1) if x == FORK_END + 2 => Some(wetland_cell(Surface::Water, i16::MIN)),
            (x, 0) if (FORK_END..=FORK_END + 2).contains(&x) => {
                Some(wetland_cell(Surface::Water, i16::MIN))
            }
            (x, 2) if (FORK_END..=FORK_END + 2).contains(&x) => {
                Some(wetland_cell(Surface::Road, i16::MIN))
            }
            _ => None,
        }
    }

    fn metres_per_cell(&self) -> u32 {
        u32::MAX
    }
}

impl RoutingGrid for PriorityGrid {
    fn width(&self) -> u32 {
        SQUARE_START + SQUARE_SIDE
    }

    fn height(&self) -> u32 {
        SQUARE_SIDE
    }

    fn cell(&self, x: u32, y: u32) -> Option<Cell> {
        if y == 0 && x < SQUARE_START {
            Some(stem_cell(x))
        } else if (SQUARE_START..self.width()).contains(&x) && y < self.height() {
            Some(Cell {
                surface: Surface::Road,
                elevation_m: i16::MIN,
                ..Cell::default()
            })
        } else {
            None
        }
    }

    fn metres_per_cell(&self) -> u32 {
        u32::MAX
    }
}

fn wetland_cell(surface: Surface, elevation_m: i16) -> Cell {
    Cell {
        elevation_m,
        surface,
        crossing: surface == Surface::Water,
        wetland_fraction_percent: 100,
        ..Cell::default()
    }
}

fn stem_cell(x: u32) -> Cell {
    let last_open = 1 + ISOLATED_OPEN_SPACING * (ISOLATED_OPEN_COUNT - 1);
    let isolated_open = x % ISOLATED_OPEN_SPACING == 1 && x <= last_open;
    let surface = if isolated_open {
        Surface::Open
    } else {
        Surface::Water
    };
    let elevation = if x.is_multiple_of(2) {
        i16::MIN
    } else {
        i16::MAX
    };
    wetland_cell(surface, elevation)
}

fn check_stem_identity(grid: &impl RoutingGrid, y: u32, end: u32) {
    let mut open_cells = Vec::new();
    for x in 0..end {
        let cell = grid.cell(x, y).expect("every stem cell is present");
        assert_eq!(cell.wetland_fraction_percent, 100);
        assert_eq!(cell.canopy_percent, 0);
        assert_eq!(cell.hilly_fraction_percent, 0);
        assert!(!cell.cultivated);
        assert_eq!(
            cell.elevation_m,
            if x.is_multiple_of(2) {
                i16::MIN
            } else {
                i16::MAX
            }
        );
        match cell.surface {
            Surface::Open => {
                assert!(!cell.crossing);
                open_cells.push(x);
            }
            Surface::Water => assert!(cell.crossing),
            other => panic!("unexpected stem surface {other:?} at {x}"),
        }
    }
    let expected: Vec<_> = (0..ISOLATED_OPEN_COUNT)
        .map(|index| 1 + ISOLATED_OPEN_SPACING * index)
        .collect();
    assert_eq!(open_cells, expected);
}

fn check_plan(
    plan: &RoutePlan,
    start: (u32, u32),
    goal: (u32, u32),
    expected_minutes: u64,
    expected_distance_m: u64,
) {
    assert_eq!(plan.minutes, expected_minutes);
    assert_eq!(plan.distance_m, expected_distance_m);
    assert_eq!(
        plan.points.first(),
        Some(&RoutePoint {
            latitude: f64::from(start.1),
            longitude: f64::from(start.0),
        })
    );
    assert_eq!(
        plan.points.last(),
        Some(&RoutePoint {
            latitude: f64::from(goal.1),
            longitude: f64::from(goal.0),
        })
    );
    assert!(!plan.spans.is_empty());
    assert!(plan.spans.len() <= MAX_TERRAIN_SPANS);
    let mut cursor = 0_u64;
    for span in &plan.spans {
        assert_eq!(span.start_minute, cursor);
        assert!(span.duration_minutes > 0);
        cursor = cursor
            .checked_add(span.duration_minutes)
            .expect("span end fits u64");
    }
    assert_eq!(cursor, expected_minutes);
    assert_eq!(
        plan.spans.as_slice(),
        &[TerrainSpan {
            surface: Surface::Water,
            terrain: TerrainWeights {
                wetlands: 1_000,
                ..TerrainWeights::default()
            },
            training_multiplier_permille: 1_000,
            check_millirank: 0,
            start_minute: 0,
            duration_minutes: expected_minutes,
        }]
    );
}

#[test]
fn wide_costs_choose_the_faster_fork_and_compact_spans() {
    let grid = ForkGrid;
    assert_eq!(
        (grid.width(), grid.height(), grid.metres_per_cell()),
        (228_003, 3, u32::MAX)
    );
    check_stem_identity(&grid, 1, FORK_END);
    assert_eq!(
        grid.cell(FORK_END, 1),
        Some(wetland_cell(Surface::Water, i16::MIN))
    );
    for x in FORK_END..=FORK_END + 2 {
        assert_eq!(
            grid.cell(x, 0),
            Some(wetland_cell(Surface::Water, i16::MIN))
        );
        assert_eq!(grid.cell(x, 2), Some(wetland_cell(Surface::Road, i16::MIN)));
    }
    assert_eq!(
        grid.cell(FORK_END + 2, 1),
        Some(wetland_cell(Surface::Water, i16::MIN))
    );
    assert!(grid.cell(FORK_END - 1, 0).is_none());
    assert!(grid.cell(FORK_END - 1, 2).is_none());
    assert!(grid.cell(FORK_END + 1, 1).is_none());
    assert!(grid.cell(FORK_END + 3, 0).is_none());
    assert!(grid.cell(FORK_END + 3, 2).is_none());

    let start = (0, 1);
    let goal = (FORK_END + 2, 1);
    let weather = RoutingWeather {
        ground_moisture_bps: 10_000,
        snow_cover_bps: 10_000,
        snow_check_millirank: 0,
    };
    let plan =
        astar_with_profile_and_weather(&grid, start, goal, TerrainSkillProfile::default(), weather)
            .expect("the lower Road fork is a feasible route");
    check_plan(
        &plan,
        start,
        goal,
        308_233_581_993_526_718,
        979_269_723_129_180,
    );
}

#[test]
fn wide_priorities_reach_the_road_square_within_the_visit_cap() {
    let grid = PriorityGrid;
    assert_eq!(
        (grid.width(), grid.height(), grid.metres_per_cell()),
        (341_692, 650, u32::MAX)
    );
    check_stem_identity(&grid, 0, SQUARE_START);
    let square_cell = Cell {
        surface: Surface::Road,
        elevation_m: i16::MIN,
        ..Cell::default()
    };
    for y in 0..SQUARE_SIDE {
        for x in SQUARE_START..SQUARE_START + SQUARE_SIDE {
            assert_eq!(grid.cell(x, y), Some(square_cell));
        }
    }
    assert!(grid.cell(SQUARE_START - 1, 1).is_none());
    assert!(grid.cell(SQUARE_START + SQUARE_SIDE, 0).is_none());
    assert!(grid.cell(SQUARE_START, SQUARE_SIDE).is_none());

    let start = (0, 0);
    let goal = (SQUARE_START + SQUARE_SIDE - 1, SQUARE_SIDE - 1);
    let profile = TerrainSkillProfile {
        plains: 5_000,
        ..TerrainSkillProfile::default()
    };
    let weather = RoutingWeather {
        ground_moisture_bps: 10_000,
        snow_cover_bps: 0,
        snow_check_millirank: 0,
    };
    let plan = astar_with_profile_and_weather(&grid, start, goal, profile, weather)
        .expect("the clear diagonal across the Road square is a feasible route");
    check_plan(
        &plan,
        start,
        goal,
        307_446_026_010_380_863,
        1_468_705_667_578_385,
    );
}
