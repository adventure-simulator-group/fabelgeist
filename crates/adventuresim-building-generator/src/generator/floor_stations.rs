//! Floor member stations keep masonry bays within the authored maximum pitch.
use crate::BuildingProgram;
const MAXIMUM_JOIST_PITCH_METRES: f32 = 1.35;
const EDGE_BEARING_INSET_METRES: f32 = 0.20;
const HEATED_BAY_SET_OUT_METRES: [f32; 3] = [0.0, 0.04, 0.08];

fn joists(program: &BuildingProgram, width: f32) -> Vec<f32> {
    let count = (width / MAXIMUM_JOIST_PITCH_METRES).ceil().max(2.0) as usize;
    let span = width - 2.0 * EDGE_BEARING_INSET_METRES;
    let heated = program.domestic_heating.is_some() && program.storeys.len() > 1;
    if !heated {
        return (0..=count)
            .map(|index| EDGE_BEARING_INSET_METRES + span * index as f32 / count as f32)
            .collect();
    }
    // Full bays retain room for the masonry; the final bay takes the remainder.
    let pitch = MAXIMUM_JOIST_PITCH_METRES;
    let set_out =
        HEATED_BAY_SET_OUT_METRES[(program.seed % HEATED_BAY_SET_OUT_METRES.len() as u64) as usize];
    let mut stations = vec![0.0];
    stations.extend(
        (1..=count)
            .map(|index| index as f32 * pitch - set_out)
            .take_while(|station| *station < span),
    );
    stations.push(span);
    let last = stations.len() - 1;
    if last > 1 && stations[last] - stations[last - 1] < pitch * 0.5 {
        // Share a short remainder between two bays, retaining the exact edge
        // bearing and never increasing a span beyond the maximum pitch.
        stations[last - 1] = (stations[last - 2] + span) * 0.5;
    }
    stations
        .into_iter()
        .map(|station| station + EDGE_BEARING_INSET_METRES)
        .collect()
}

pub(super) fn with_stair(
    program: &BuildingProgram,
    width: f32,
    opening: Option<(bevy::math::Vec2, bevy::math::Vec2)>,
) -> Vec<f32> {
    const STATION_MERGE_DISTANCE_METRES: f32 = 0.08;
    let mut stations = joists(program, width);
    if let Some((min, max)) = opening {
        stations.extend([min.x, max.x]);
        stations.sort_by(f32::total_cmp);
        stations.dedup_by(|a, b| (*a - *b).abs() < STATION_MERGE_DISTANCE_METRES);
    }
    stations
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heated_bays_keep_edge_bearings_inside_the_envelope_at_every_set_out() {
        let mut program = BuildingProgram::fixture(crate::BuildingArchetype::TownHouse, 0);
        program.domestic_heating = Some(crate::DomesticHeatingProgramme::HearthAndRearFedStove);
        for width in (3..24)
            .map(|cells| cells as f32 * crate::CELL_SIZE_METRES)
            .chain([11.0])
        {
            for seed in 0..HEATED_BAY_SET_OUT_METRES.len() {
                program.seed = seed as u64;
                let stations = joists(&program, width);
                assert_eq!(stations[0], EDGE_BEARING_INSET_METRES);
                assert!(
                    (stations.last().unwrap() - (width - EDGE_BEARING_INSET_METRES)).abs() < 0.0001
                );
                assert!(
                    stations.windows(2).all(|bay| bay[1] > bay[0]
                        && bay[1] - bay[0] <= MAXIMUM_JOIST_PITCH_METRES + 0.0001)
                );
            }
        }
    }
}
