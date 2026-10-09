//! Source geometry admission to the cultivation distance kernel.
use super::Point;
use adventuresim_world_import::{
    cultivation::{MetricSegment, SegmentDistanceIndex},
    spatial::SpatialProjection,
};

pub(super) fn build(
    projection: &SpatialProjection,
    lines: &[&[Point]],
) -> Result<SegmentDistanceIndex, Box<dyn std::error::Error>> {
    // This numerical index uses whole east/north metres in EPSG:3035. Source
    // points remain full longitude/latitude degrees until projection here.
    let segments = lines
        .iter()
        .flat_map(|line| line.windows(2))
        .map(|pair| {
            let from = projection.project(pair[0].0[1], pair[0].0[0])?;
            let to = projection.project(pair[1].0[1], pair[1].0[0])?;
            Ok(MetricSegment {
                from: [
                    from.easting_millimeters().div_euclid(1_000),
                    from.northing_millimeters().div_euclid(1_000),
                ],
                to: [
                    to.easting_millimeters().div_euclid(1_000),
                    to.northing_millimeters().div_euclid(1_000),
                ],
            })
        })
        .collect::<Result<Vec<_>, adventuresim_world_import::Error>>()?;
    Ok(SegmentDistanceIndex::new(segments)?)
}
