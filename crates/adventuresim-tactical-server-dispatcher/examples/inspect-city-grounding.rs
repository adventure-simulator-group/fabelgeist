//! Bounded support/origin report for the production city scene, without a UI.
use adventuresim_building_generator::{
    BuildingLodLevel, BuildingProgram, compile_building_lod, compile_program_shell,
};
use adventuresim_tactical_core::{
    prelude::*, scene_input::GeneratedBuildingRecipes, vista_surface::vista_triangle_height,
};
use bevy::math::{Vec2, Vec3Swizzles};
use clap::Parser;
use serde_json::{Value, json};
use std::path::PathBuf;
#[path = "inspect_city_grounding/footprint.rs"]
mod footprint;
#[path = "inspect_city_grounding/solutions.rs"]
mod solutions;
#[path = "inspect_city_grounding/support_regions.rs"]
mod support_regions;
#[path = "inspect_city_grounding/terraces.rs"]
mod terraces;

/// Required provenance and terrain fields from the production stage export.
#[derive(serde::Deserialize)]
struct TerrainStages {
    input_digest: String,
    source_digest: String,
    ungraded_vista: adventuresim_tactical_core::scene_input::VistaSample,
}

#[derive(Parser)]
struct Inspection {
    #[arg(long)]
    scene_input: PathBuf,
    #[arg(long)]
    output: PathBuf,
    /// Limit expensive detail compilation to this radius around the active area.
    #[arg(long, default_value_t = 150.0)]
    radius_metres: f32,
    /// Inspect only these exact identities; repeat or separate with commas.
    #[arg(long, value_delimiter = ',')]
    building_id: Vec<u64>,
    /// Compare only this exact compound using ungraded production terrain.
    #[arg(long, requires = "terrain_stages")]
    property_id: Option<u64>,
    /// Source-stage export produced by export-city-scene --terrain-stages.
    #[arg(long, requires = "property_id")]
    terrain_stages: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = Inspection::parse();
    let input = TacticalSceneInput::load(&request.scene_input)?;
    let generated = input.generate()?;
    let mut recipes = GeneratedBuildingRecipes::default();
    let mut records = Vec::new();
    for building in &generated.buildings {
        if !request.building_id.is_empty() && !request.building_id.contains(&building.placement.id)
        {
            continue;
        }
        records.push(inspect(
            &input,
            &generated.terrain,
            &mut recipes,
            building.placement.id,
            &building.placement.program,
            building.placement.centre_metres,
            building.placement.orientation,
            building.placement.base_elevation_metres,
            "playable",
        )?);
    }
    for building in &input.distant_buildings {
        if (!request.building_id.is_empty() && request.building_id.contains(&building.id))
            || (request.building_id.is_empty()
                && building.centre_metres.length() <= request.radius_metres)
        {
            records.push(inspect(
                &input,
                &generated.terrain,
                &mut recipes,
                building.id,
                &building.occupied_program(),
                building.centre_metres,
                building.orientation,
                building.base_elevation_metres,
                "distant",
            )?);
        }
    }
    let report = json!({
        "input": request.scene_input, "scene_digest": generated.digest,
        "source": input.source, "seed": input.seed,
        "schema_version": input.schema_version,
        "generation_version": input.generation_version,
        "absolute_minute": input.absolute_minute,
        "absolute_elevation_metres": input.absolute_elevation_metres,
        "buildings": records,
        "property_catalog": input.properties,
        "terrace_comparison": request.terrace_comparison(&input, &mut recipes)?,
    });
    std::fs::write(request.output, serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

impl Inspection {
    fn terrace_comparison(
        &self,
        input: &TacticalSceneInput,
        recipes: &mut GeneratedBuildingRecipes,
    ) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let Some(id) = self.property_id else {
            return Ok(None);
        };
        let compound = input
            .compounds
            .iter()
            .find(|c| c.id.0 == id)
            .ok_or("requested property is absent from the scene")?;
        let stages: TerrainStages = serde_json::from_slice(&std::fs::read(
            self.terrain_stages
                .as_ref()
                .ok_or("missing terrain stages")?,
        )?)?;
        if Some(stages.source_digest.as_str())
            != match &input.source {
                adventuresim_tactical_core::scene_input::SceneSource::ImportedPackage(digest) => {
                    Some(digest.as_str())
                }
                _ => None,
            }
        {
            return Err("terrain stages do not identify the scene source".into());
        }
        if stages.input_digest != input.digest()? {
            return Err("terrain stages do not identify this exact scene input".into());
        }
        let vista = stages.ungraded_vista;
        let raw = SceneTerrain::from_heightmap(
            usize::from(input.playable.width),
            usize::from(input.playable.depth),
            input.playable.spacing_metres,
            input.playable.heights_metres.clone(),
        )
        .ok_or("invalid ungraded playable grid")?;
        let height = |point| {
            raw.height_at(point).or_else(|| {
                vista.lods.iter().enumerate().find_map(|(i, lod)| {
                    vista_triangle_height(lod, vista.lods.get(i + 1), &raw, point)
                })
            })
        };
        let main = threshold(input, recipes, compound.front_building_id, -Vec2::Y)?;
        let front = threshold(input, recipes, compound.front_building_id, Vec2::Y)?;
        let rear = threshold(input, recipes, compound.rear_building_id, -Vec2::Y)?;
        let mut comparison =
            terraces::compare(
                compound,
                front,
                rear,
                height(main).ok_or("main threshold leaves terrain")?,
                height,
                adventuresim_tactical_core::scene_input::MAX_PLAYABLE_GRADE,
            )
            .ok_or_else(|| {
                format!(
            "scene {} property {} members [{}, {}] has no endpoint terrace candidate; \
             required thresholds {:?} and {:?}, join tolerance {} m; \
             check exact route membership, positive ramp run and source coverage",
            input.scene_key, compound.id.0, compound.front_building_id,
            compound.rear_building_id, front, rear,
            adventuresim_tactical_core::city_layout::CityAccessSegment::JOIN_TOLERANCE_METRES,
        )
            })?;
        let placement = member_placement(input, compound.front_building_id)?;
        let recipe = recipes.get_or_generate(&placement.program)?;
        comparison["support_boundaries"] = support_regions::describe(compound, &placement, recipe)
            .ok_or("compound front member has no ground-contact geometry")?;
        let lod = vista.lods.first().ok_or("source stages lack a vista LOD")?;
        let contact = support_regions::contact_region(&placement, recipe)
            .ok_or("compound front member has no contact region")?;
        comparison["source_plot_triangle_extrema"] =
            footprint::vista_extrema(lod, vista.lods.get(1), &raw, compound.plot.corners());
        comparison["source_front_contact_triangle_extrema"] =
            footprint::vista_extrema(lod, vista.lods.get(1), &raw, contact.corners());
        let rear_placement = member_placement(input, compound.rear_building_id)?;
        let rear_recipe = recipes.get_or_generate(&rear_placement.program)?;
        let rear_contact = support_regions::contact_region(&rear_placement, rear_recipe)
            .ok_or("compound rear member has no contact region")?;
        let proposed = comparison["candidates"][1]["front_court_rear_elevations_m"]
            .as_array()
            .ok_or("missing proposed terrace levels")?;
        let proposed = [0, 1, 2].map(|i| proposed[i].as_f64().unwrap() as f32);
        comparison["support_solution_comparison"] = solutions::compare(
            compound,
            [(contact, front), (rear_contact, rear)],
            proposed,
            &footprint::vista_triangles(lod, vista.lods.get(1), &raw, compound.plot.corners()),
            height,
            adventuresim_tactical_core::scene_input::MAX_PLAYABLE_GRADE,
        )
        .ok_or("missing exact gate route or source coverage for support comparison")?;
        Ok(Some(comparison))
    }
}

fn threshold(
    input: &TacticalSceneInput,
    recipes: &mut GeneratedBuildingRecipes,
    id: u64,
    outward: Vec2,
) -> Result<Vec2, Box<dyn std::error::Error>> {
    let placement = member_placement(input, id)?;
    let recipe = recipes.get_or_generate(&placement.program)?;
    let doors = adventuresim_building_generator::compile_operable_doors(&recipe.plan);
    let mut matches = doors.iter().filter(|door| door.outward == outward);
    let door = matches
        .next()
        .filter(|_| matches.next().is_none())
        .ok_or("property member has no required external door")?;
    let local = door.hinge_centre.xz() + door.tangent * door.size_metres.x * 0.5
        - recipe.collision.bounds.centre().xz();
    Ok(placement.centre_metres + placement.orientation.local_to_world(local))
}

fn member_placement(
    input: &TacticalSceneInput,
    id: u64,
) -> Result<TacticalBuildingPlacement, Box<dyn std::error::Error>> {
    input
        .buildings
        .iter()
        .find(|b| b.id == id)
        .cloned()
        .or_else(|| {
            input
                .distant_buildings
                .iter()
                .find(|b| b.id == id)
                .map(|b| (*b).into())
        })
        .ok_or_else(|| "property member is absent".into())
}

#[expect(
    clippy::too_many_arguments,
    reason = "offline report records explicit scene placement and representation"
)]
fn inspect(
    input: &TacticalSceneInput,
    terrain: &SceneTerrain,
    recipes: &mut GeneratedBuildingRecipes,
    id: u64,
    program: &BuildingProgram,
    centre: Vec2,
    orientation: BuildingOrientation,
    elevation: f32,
    representation: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let recipe = recipes.get_or_generate(program)?;
    let origin = recipe.collision.bounds.centre();
    let offset = elevation;
    let world = |local: Vec2| centre + orientation.local_to_world(local - origin.xz());
    let height = |point| {
        terrain.height_at(point).or_else(|| {
            input.vista.lods.iter().enumerate().find_map(|(i, lod)| {
                vista_triangle_height(lod, input.vista.lods.get(i + 1), terrain, point)
            })
        })
    };
    let supports = recipe
        .plan
        .resolved_geometry
        .structural_nodes
        .iter()
        .filter(|node| node.grounded)
        .map(|node| {
            let point = world(node.position.xz());
            json!({
                "node": node.id, "kind": node.kind, "point": point,
                "world_elevation": node.position.y + offset,
                "terrain_elevation": height(point),
            })
        })
        .collect::<Vec<_>>();
    let thresholds = adventuresim_building_generator::compile_operable_doors(&recipe.plan)
        .iter()
        .map(|door| {
            let opening = recipe
                .plan
                .opening_assemblies
                .iter()
                .find(|opening| opening.id == door.opening)
                .expect("compiled door identifies its source opening");
            let point = world(door.closed_centre.xz());
            json!({"id":opening.id, "point":point,
                "sill_elevation":opening.sill_elevation_metres + offset,
                "terrain_elevation":height(point)})
        })
        .collect::<Vec<_>>();
    let levels = lod_minima(&recipe.plan, program, offset);
    let corners = [
        recipe.collision.bounds.min.xz(),
        Vec2::new(recipe.collision.bounds.max.x, recipe.collision.bounds.min.z),
        recipe.collision.bounds.max.xz(),
        Vec2::new(recipe.collision.bounds.min.x, recipe.collision.bounds.max.z),
    ]
    .map(world);
    Ok(json!({
        "id": id, "representation": representation, "program": program,
        "centre": centre, "base_elevation": elevation,
        "collision_minimum": recipe.collision.bounds.min.y,
        "ground_floor_contact_bounds":recipe.collision.ground_floor_contact_bounds(),
        "collision_origin": origin, "plan_to_world_vertical_offset": offset,
        "incorrect_collider_bottom_offset": elevation - recipe.collision.bounds.min.y,
        "centre_terrain": height(centre),
        "footprint_corners": corners.map(|p| json!({"point":p,"terrain":height(p)})),
        "playable_footprint_terrain": footprint::terrain_extrema(terrain, corners),
        "structural_audit_issue_count": adventuresim_building_generator::audit_plan(&recipe.plan).len(),
        "grounded_nodes": supports, "exterior_thresholds":thresholds, "lod_minima": levels,
        "compound": input.compounds.iter().find(|p| p.front_building_id == id || p.rear_building_id == id),
        "garden": input.gardens.iter().find(|p| p.front_building_id == id),
    }))
}

fn lod_minima(
    plan: &adventuresim_building_generator::BuildingPlan,
    program: &BuildingProgram,
    offset: f32,
) -> Vec<Value> {
    let mut levels = Vec::new();
    for level in [BuildingLodLevel::Facade, BuildingLodLevel::Shell] {
        let lod = if level == BuildingLodLevel::Shell {
            compile_program_shell(program).unwrap_or_else(|| compile_building_lod(plan, level))
        } else {
            compile_building_lod(plan, level)
        };
        let minimum = lod
            .meshes
            .iter()
            .flat_map(|mesh| &mesh.vertices)
            .map(|vertex| vertex.position.y)
            .fold(f32::INFINITY, f32::min);
        levels.push(json!({"level": level, "local_minimum": minimum,
            "world_minimum": minimum + offset}));
    }
    levels
}
