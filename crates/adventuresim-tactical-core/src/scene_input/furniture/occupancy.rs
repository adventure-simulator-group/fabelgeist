//! Spatial broad phase for the same exact rotated reservation tests.
use super::*;
use std::collections::BTreeMap;
const RESERVATION_CELL_METRES: f32 = 16.0;
#[derive(Default)]
pub(super) struct Occupancy(BTreeMap<(i32, i32), Vec<FurnitureFootprint>>);
fn cells(footprint: FurnitureFootprint) -> impl Iterator<Item = (i32, i32)> {
    let corners = footprint.corners();
    let min = (corners
        .into_iter()
        .fold(Vec2::splat(f32::INFINITY), Vec2::min)
        / RESERVATION_CELL_METRES)
        .floor()
        .as_ivec2();
    let max = (corners
        .into_iter()
        .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max)
        / RESERVATION_CELL_METRES)
        .floor()
        .as_ivec2();
    (min.x..=max.x).flat_map(move |x| (min.y..=max.y).map(move |y| (x, y)))
}
impl Occupancy {
    pub(super) fn insert(&mut self, footprint: FurnitureFootprint) {
        for cell in cells(footprint) {
            self.0.entry(cell).or_default().push(footprint);
        }
    }
    pub(super) fn intersects(&self, footprint: FurnitureFootprint) -> bool {
        cells(footprint).any(|cell| {
            self.0
                .get(&cell)
                .is_some_and(|items| items.iter().any(|other| footprint.intersects(*other)))
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn broad_phase_matches_exact_intersection_for_rotated_multicell_footprints() {
        let mut index = Occupancy::default();
        let footprints = (0..20)
            .map(|i| FurnitureFootprint {
                centre_metres: Vec2::new(i as f32 * 7.0 - 60.0, i as f32 - 10.0),
                half_extents_metres: Vec2::new(20.0, 2.0),
                orientation: BuildingOrientation::from_radians(i as f32 * 0.3).unwrap(),
            })
            .collect::<Vec<_>>();
        for f in &footprints {
            index.insert(*f);
        }
        for x in -30..30 {
            for z in -30..30 {
                let q = FurnitureFootprint {
                    centre_metres: Vec2::new(x as f32 * 3.0, z as f32 * 3.0),
                    half_extents_metres: Vec2::new(2.0, 4.0),
                    orientation: BuildingOrientation::from_radians(0.6).unwrap(),
                };
                assert_eq!(
                    index.intersects(q),
                    footprints.iter().any(|f| f.intersects(q))
                );
            }
        }
    }
}
