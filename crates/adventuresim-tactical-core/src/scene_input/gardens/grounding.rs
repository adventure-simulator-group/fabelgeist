//! Boundary properties anchor their shared playable terrace to the graded vista.
use super::*;
use bevy::math::Vec2;
use std::collections::BTreeMap;
mod boundary;
#[derive(Default)]
pub(in crate::scene_input) struct GardenGrounding {
    pub anchors: BTreeMap<u64, f32>,
    pub boundary_pads: Vec<(Vec<u64>, buildings::BuildingPad)>,
}
const LEVEL_TOLERANCE_METRES: f32 = 0.001;

pub(in crate::scene_input) fn terrain_anchors(
    input: &TacticalSceneInput,
) -> Result<GardenGrounding, SceneInputError> {
    let half = Vec2::new(
        f32::from(input.playable.width - 1),
        f32::from(input.playable.depth - 1),
    ) * input.playable.spacing_metres
        * 0.5;
    let mut grounding = GardenGrounding::default();
    for garden in &input.gardens {
        if garden
            .plot
            .corners()
            .iter()
            .all(|p| p.abs().cmple(half).all())
        {
            continue;
        }
        let elevation = vista_level(input, garden)?;
        if let Some(front) = input
            .distant_buildings
            .iter()
            .find(|b| b.id == garden.front_building_id)
            && (front.base_elevation_metres - elevation).abs() > LEVEL_TOLERANCE_METRES
        {
            return invalid("distant garden owner does not share its graded vista elevation");
        }
        grounding
            .anchors
            .insert(garden.front_building_id, elevation);
        grounding
            .boundary_pads
            .extend(boundary::supports(input, garden, elevation, half));
    }
    Ok(grounding)
}

fn vista_level(input: &TacticalSceneInput, garden: &CityGarden) -> Result<f32, SceneInputError> {
    let Some(first) = input.vista.lods.first() else {
        return invalid("boundary garden requires graded vista terrain");
    };
    let origin = Vec2::new(
        first.origin_east_metres as f32,
        first.origin_north_metres as f32,
    );
    for (index, lod) in input.vista.lods.iter().enumerate() {
        let dimensions = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1));
        let lod_origin = Vec2::new(
            lod.origin_east_metres as f32,
            lod.origin_north_metres as f32,
        );
        let grid = garden
            .plot
            .corners()
            .map(|p| (p + origin - lod_origin) / lod.spacing_metres + dimensions * 0.5);
        let minimum = grid
            .into_iter()
            .fold(Vec2::splat(f32::INFINITY), Vec2::min)
            .floor();
        let maximum = grid
            .into_iter()
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max)
            .ceil();
        if minimum.cmplt(Vec2::ZERO).any() || maximum.cmpgt(dimensions).any() {
            continue;
        }
        let reference =
            lod.heights_metres[minimum.y as usize * usize::from(lod.width) + minimum.x as usize];
        for z in minimum.y as usize..=maximum.y as usize {
            for x in minimum.x as usize..=maximum.x as usize {
                let world = (Vec2::new(x as f32, z as f32) - dimensions * 0.5) * lod.spacing_metres
                    + lod_origin;
                let height = crate::vista_surface::presented_height_at(
                    lod,
                    world,
                    input.vista.lods.get(index + 1),
                );
                if height.is_none_or(|h| (h - reference).abs() > LEVEL_TOLERANCE_METRES) {
                    return invalid(
                        "garden requires a level terrace across its complete vista footprint and LOD morph",
                    );
                }
            }
        }
        return Ok(reference);
    }
    invalid("garden leaves the supplied vista domain")
}

pub(in crate::scene_input) fn validate_surface(
    input: &TacticalSceneInput,
    terrain: &SceneTerrain,
    buildings: &[GeneratedBuilding],
) -> Result<(), SceneInputError> {
    for garden in &input.gardens {
        let elevation = buildings
            .iter()
            .find(|b| b.placement.id == garden.front_building_id)
            .map(|b| b.pad_elevation_metres)
            .or_else(|| {
                input
                    .distant_buildings
                    .iter()
                    .find(|b| b.id == garden.front_building_id)
                    .map(|b| b.base_elevation_metres)
            })
            .expect("validated garden owner");
        for point in garden
            .plot
            .corners()
            .into_iter()
            .chain(garden.plants.iter().flat_map(|p| {
                p.world_hull()
                    .into_iter()
                    .chain(std::iter::once(p.centre_metres))
            }))
        {
            let height = terrain.height_at(point).or_else(|| {
                let first = input.vista.lods.first()?;
                let world = point
                    + Vec2::new(
                        first.origin_east_metres as f32,
                        first.origin_north_metres as f32,
                    );
                input
                    .vista
                    .lods
                    .iter()
                    .enumerate()
                    .find_map(|(index, lod)| {
                        crate::vista_surface::vista_triangle_height(
                            lod,
                            input.vista.lods.get(index + 1),
                            terrain,
                            world,
                        )
                    })
            });
            if height.is_none_or(|h| (h - elevation).abs() > LEVEL_TOLERANCE_METRES) {
                return invalid("garden terrace does not match the final stitched terrain");
            }
        }
    }
    Ok(())
}
