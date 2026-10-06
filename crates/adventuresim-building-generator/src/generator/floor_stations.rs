//! Floor member stations keep masonry bays within the authored maximum pitch.
use crate::BuildingProgram;
use crate::spatial_geometry::{Architectural, GeometryError, SpatialBounds};
mod measurements;
use measurements::FloorSpan;
pub(super) use measurements::{FloorStation, FloorWidth};
const MAXIMUM_JOIST_PITCH_METRES: f32 = 1.35;
const EDGE_BEARING_INSET_METRES: f32 = 0.20;
const UPPER_HEATED_BAY_SET_OUT_METRES: f32 = 0.08;
const GROUNDED_HEATED_BAY_SET_OUT_METRES: [f32; 3] = [0.0, 0.04, 0.08];

fn joists(
    program: &BuildingProgram,
    width: FloorWidth,
) -> Result<Vec<FloorStation>, GeometryError> {
    let width = width.metres();
    let count = (width / MAXIMUM_JOIST_PITCH_METRES).ceil().max(2.0) as usize;
    let span = FloorSpan::from_metres(width - 2.0 * EDGE_BEARING_INSET_METRES)?.metres();
    let heated = program.domestic_heating.is_some() && program.storeys.len() > 1;
    if !heated {
        return (0..=count)
            .map(|index| {
                FloorStation::from_metres(
                    EDGE_BEARING_INSET_METRES + span * index as f32 / count as f32,
                )
            })
            .collect();
    }
    // Align the reserved rear masonry bay with the unchanged roof frame.
    // A seeded set-out could admit the plinth but put its bearing ledge into
    // a joist, while the next clear floor station met a roof girder.
    let pitch = MAXIMUM_JOIST_PITCH_METRES;
    let set_out = if super::heated_rooms::HeatingStorey::for_program(
        program,
        crate::StoreyIndex::FIRST_UPPER,
    )
    .is_some()
    {
        UPPER_HEATED_BAY_SET_OUT_METRES
    } else {
        GROUNDED_HEATED_BAY_SET_OUT_METRES[fabelgeist_determinism::StreamId::new(
            "building.heated-bay-set-out",
        )
        .rng(program.seed, &[])
        .index(GROUNDED_HEATED_BAY_SET_OUT_METRES.len())]
    };
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
        .map(|station| FloorStation::from_metres(station + EDGE_BEARING_INSET_METRES))
        .collect()
}

pub(super) fn with_stair(
    program: &BuildingProgram,
    width: FloorWidth,
    opening: Option<SpatialBounds<Architectural>>,
) -> Result<Vec<FloorStation>, GeometryError> {
    const STATION_MERGE_DISTANCE_METRES: f32 = 0.08;
    let mut stations = joists(program, width)?;
    if let Some(opening) = opening {
        stations.extend([
            FloorStation::from_metres(opening.min().metres().x)?,
            FloorStation::from_metres(opening.max().metres().x)?,
        ]);
        stations.sort_by(|a, b| a.metres().total_cmp(&b.metres()));
        stations.dedup_by(|a, b| (a.metres() - b.metres()).abs() < STATION_MERGE_DISTANCE_METRES);
    }
    Ok(stations)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heated_bays_keep_edge_bearings_and_maximum_pitch_across_dimensions_and_seeds() {
        let mut program = BuildingProgram::fixture(crate::BuildingArchetype::TownHouse, 0);
        program.domestic_heating = Some(crate::DomesticHeatingProgramme::HearthAndRearFedStove);
        for width in (3..24)
            .map(|cells| cells as f32 * crate::CELL_SIZE_METRES)
            .chain([11.0])
        {
            for seed in [0, 42, 47, 101, u64::MAX] {
                program.seed = seed;
                let stations = joists(&program, FloorWidth::from_metres(width).unwrap()).unwrap();
                assert_eq!(stations[0].metres(), EDGE_BEARING_INSET_METRES);
                assert!(
                    (stations.last().unwrap().metres() - (width - EDGE_BEARING_INSET_METRES)).abs()
                        < 0.0001
                );
                assert!(
                    stations
                        .windows(2)
                        .all(|bay| bay[1].metres() > bay[0].metres()
                            && bay[1].metres() - bay[0].metres()
                                <= MAXIMUM_JOIST_PITCH_METRES + 0.0001)
                );
            }
        }
    }
}
