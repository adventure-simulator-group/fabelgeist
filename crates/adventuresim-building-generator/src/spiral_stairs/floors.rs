use super::flight::{DECK_THICKNESS_METRES, SpiralFlight, WELL_MARGIN_METRES};
use super::*;

#[derive(Clone, Copy)]
struct Rectangle {
    min: Vec2,
    max: Vec2,
}

impl Rectangle {
    fn subtract(self, hole: Self) -> Vec<Self> {
        let min = self.min.max(hole.min);
        let max = self.max.min(hole.max);
        if min.cmpge(max).any() {
            return vec![self];
        }
        [
            Self {
                min: self.min,
                max: Vec2::new(min.x, self.max.y),
            },
            Self {
                min: Vec2::new(max.x, self.min.y),
                max: self.max,
            },
            Self {
                min: Vec2::new(min.x, self.min.y),
                max: Vec2::new(max.x, min.y),
            },
            Self {
                min: Vec2::new(min.x, max.y),
                max: Vec2::new(max.x, self.max.y),
            },
        ]
        .into_iter()
        .filter(|rect| (rect.max - rect.min).min_element() > 0.001)
        .collect()
    }
}

pub(super) fn resolve(plan: &mut BuildingPlan, flights: &[(usize, SpiralFlight)]) {
    let centre = plan.dimensions_metres() * 0.5;
    let bearing = foundation(&mut plan.resolved_geometry, FLOOR_OWNER, centre);
    let mut index = 0;
    for storey in &plan.storeys {
        let cells = storey
            .rooms
            .iter()
            .flat_map(|r| &r.cells)
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        for cell in cells {
            let mut panels = vec![Rectangle {
                min: cell.centre() - Vec2::splat(crate::CELL_SIZE_METRES * 0.5),
                max: cell.centre() + Vec2::splat(crate::CELL_SIZE_METRES * 0.5),
            }];
            if storey.level > 0 {
                for (_, flight) in flights.iter().filter(|(_, flight)| {
                    let height = f32::from(storey.level) * plan.storey_height_metres;
                    height > flight.base_height_metres && height <= flight.top_height_metres + 0.001
                }) {
                    let half = Vec2::splat(flight.outer_radius_metres + WELL_MARGIN_METRES);
                    let hole = Rectangle {
                        min: flight.centre - half,
                        max: flight.centre + half,
                    };
                    panels = panels
                        .into_iter()
                        .flat_map(|panel| panel.subtract(hole))
                        .collect();
                }
            }
            for panel in panels {
                let centre = (panel.min + panel.max) * 0.5;
                let size = panel.max - panel.min;
                append(
                    &mut plan.resolved_geometry,
                    FLOOR_OWNER,
                    index,
                    bearing,
                    SpiralMember {
                        centre: Vec3::new(
                            centre.x,
                            f32::from(storey.level) * plan.storey_height_metres
                                - DECK_THICKNESS_METRES * 0.5,
                            centre.y,
                        ),
                        size: Vec3::new(size.x, DECK_THICKNESS_METRES, size.y),
                        yaw_radians: 0.0,
                        role: SolidRole::InteriorFloor,
                    },
                );
                index += 1;
            }
        }
    }
}
