use super::*;
use crate::scene_input::TacticalSceneInput;
mod levels;

pub(in crate::scene_input) fn level_building_pads(
    grid_width: usize,
    grid_depth: usize,
    spacing: f32,
    heights: &mut [f32],
    buildings: &mut [GeneratedBuilding],
    input: &TacticalSceneInput,
    grounding: &super::super::gardens::GardenGrounding,
) -> Result<(Vec<BuildingPad>, u32), SceneInputError> {
    let half_extent = Vec2::new(
        (grid_width - 1) as f32 * spacing,
        (grid_depth - 1) as f32 * spacing,
    ) * 0.5;
    let mut pads = Vec::with_capacity(buildings.len());
    let mut adjusted = 0u32;
    let mut proposals = proposals(buildings, &input.compounds, &input.gardens);
    proposals.extend(grounding.boundary_pads.iter().cloned());
    let elevations = levels::shared_elevations(
        &proposals.iter().map(|(_, pad)| *pad).collect::<Vec<_>>(),
        &proposals
            .iter()
            .map(|(ids, _)| ids.iter().find_map(|id| grounding.anchors.get(id).copied()))
            .collect::<Vec<_>>(),
        grid_width,
        grid_depth,
        spacing,
        half_extent,
        heights,
    )?;
    for ((ids, mut pad), elevation) in proposals.into_iter().zip(elevations) {
        pad.elevation_metres = elevation;
        for (index, point) in sample_indices(grid_width, grid_depth, spacing, half_extent) {
            let weight = pad.blend_weight(point);
            if weight <= 0.0 {
                continue;
            }
            let previous = heights[index];
            heights[index] = previous + (pad.elevation_metres - previous) * weight;
            adjusted += u32::from((heights[index] - previous).abs() > f32::EPSILON);
        }
        for building in buildings
            .iter_mut()
            .filter(|b| ids.contains(&b.placement.id))
        {
            building.pad_elevation_metres = pad.elevation_metres;
        }
        pads.push(pad);
    }
    for (index, point) in sample_indices(grid_width, grid_depth, spacing, half_extent) {
        if let Some(pad) = pads.iter().find(|pad| pad.contains_level_ground(point)) {
            heights[index] = pad.elevation_metres;
        }
    }
    Ok((pads, adjusted))
}

fn sample_indices(
    width: usize,
    depth: usize,
    spacing: f32,
    half_extent: Vec2,
) -> impl Iterator<Item = (usize, Vec2)> {
    (0..width * depth).map(move |index| {
        let point =
            Vec2::new((index % width) as f32, (index / width) as f32) * spacing - half_extent;
        (index, point)
    })
}

fn nearest_height(width: usize, depth: usize, spacing: f32, heights: &[f32], point: Vec2) -> f32 {
    let half_extent = Vec2::new((width - 1) as f32 * spacing, (depth - 1) as f32 * spacing) * 0.5;
    let grid = ((point + half_extent) / spacing).round();
    let x = (grid.x as isize).clamp(0, width as isize - 1) as usize;
    let z = (grid.y as isize).clamp(0, depth as isize - 1) as usize;
    heights[z * width + x]
}

fn proposals(
    buildings: &[GeneratedBuilding],
    compounds: &[crate::city_layout::CityCompound],
    gardens: &[crate::city_layout::CityGarden],
) -> Vec<(Vec<u64>, BuildingPad)> {
    let members = compounds
        .iter()
        .flat_map(|c| [c.front_building_id, c.rear_building_id])
        .chain(gardens.iter().map(|g| g.front_building_id))
        .collect::<std::collections::BTreeSet<_>>();
    let mut proposals = buildings
        .iter()
        .filter(|b| !members.contains(&b.placement.id))
        .map(|building| {
            (
                vec![building.placement.id],
                BuildingPad {
                    centre: building.placement.centre_metres,
                    half_extents: building.collision.bounds.plan_half_extents(),
                    orientation: building.placement.orientation,
                    elevation_metres: 0.0,
                },
            )
        })
        .collect::<Vec<_>>();
    for compound in compounds {
        if buildings
            .iter()
            .any(|b| b.placement.id == compound.front_building_id)
        {
            proposals.push((
                vec![compound.front_building_id, compound.rear_building_id],
                BuildingPad {
                    centre: compound.plot.centre_metres,
                    half_extents: compound.plot.dimensions_metres * 0.5,
                    orientation: compound.plot.orientation,
                    elevation_metres: 0.0,
                },
            ));
        }
    }
    for garden in gardens {
        if buildings
            .iter()
            .any(|b| b.placement.id == garden.front_building_id)
        {
            proposals.push((
                vec![garden.front_building_id],
                BuildingPad {
                    centre: garden.plot.centre_metres,
                    half_extents: garden.plot.dimensions_metres * 0.5,
                    orientation: garden.plot.orientation,
                    elevation_metres: 0.0,
                },
            ));
        }
    }
    proposals
}
