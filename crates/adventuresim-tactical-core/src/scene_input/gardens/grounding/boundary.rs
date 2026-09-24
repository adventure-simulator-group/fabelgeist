//! Grade the edge intervals actually sampled by the garden's vista triangles.
use super::*;

pub(super) fn supports(
    input: &TacticalSceneInput,
    garden: &CityGarden,
    elevation: f32,
    half: Vec2,
) -> Vec<(Vec<u64>, buildings::BuildingPad)> {
    let lod = &input.vista.lods[0];
    let centre = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1)) * 0.5;
    let grid = garden
        .plot
        .corners()
        .map(|p| p / lod.spacing_metres + centre);
    let min = (grid
        .into_iter()
        .fold(Vec2::splat(f32::INFINITY), Vec2::min)
        .floor()
        - centre)
        * lod.spacing_metres;
    let max = (grid
        .into_iter()
        .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max)
        .ceil()
        - centre)
        * lod.spacing_metres;
    let mut result = Vec::new();
    for axis in 0..2 {
        let other = 1 - axis;
        for sign in [-1.0, 1.0] {
            let edge = half[axis] * sign;
            let outside = if sign > 0.0 {
                (min[axis], max[axis])
            } else {
                (-max[axis], -min[axis])
            };
            if outside.1 <= half[axis] || outside.0 >= half[axis] + lod.spacing_metres {
                continue;
            }
            let start = min[other].clamp(-half[other], half[other]);
            let end = max[other].clamp(-half[other], half[other]);
            let mut centre = Vec2::ZERO;
            centre[axis] = edge;
            centre[other] = (start + end) * 0.5;
            let mut half_extents = Vec2::ZERO;
            half_extents[other] = (end - start) * 0.5;
            result.push((
                vec![garden.front_building_id],
                buildings::BuildingPad {
                    centre,
                    half_extents,
                    orientation: BuildingOrientation::IDENTITY,
                    elevation_metres: elevation,
                },
            ));
        }
    }
    result
}
