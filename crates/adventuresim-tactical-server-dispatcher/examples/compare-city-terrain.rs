//! Bounded comparison of terrain sources at unchanged production building IDs.
use adventuresim_building_generator::{compile_ground_entrances, generate};
use adventuresim_tactical_core::{
    city_layout::grounding::{GeographicSurface, SupportElevation},
    prelude::*,
    scene_input::GeneratedBuildingRecipes,
};
use bevy::math::{Vec2, Vec3Swizzles};
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;
#[path = "compare_city_terrain/capture.rs"]
mod capture;
#[path = "compare_city_terrain/capture_export.rs"]
mod capture_export;
#[path = "compare_city_terrain/contacts.rs"]
mod contacts;
#[path = "compare_city_terrain/gardens.rs"]
mod gardens;
use capture::{PreparedTerrainCapture, TerrainPreparationPurpose};

#[derive(Parser)]
struct Request {
    #[arg(long)]
    scene_input: PathBuf,
    #[arg(long)]
    terrain_stages: PathBuf,
    #[arg(long)]
    output: PathBuf,
    /// Save fine terrain prepared with complete production scene context.
    #[arg(long)]
    prepared_terrain_output: Option<PathBuf>,
    /// Save exact physical triangles for bounded footprint/collision inspection.
    #[arg(long)]
    physical_surface_output: Option<PathBuf>,
    /// Materialize production colliders and probe exact bearing vertices.
    #[arg(long)]
    collision_probes: bool,
    /// Restrict expensive triangle comparisons to these exact identities.
    #[arg(long, value_delimiter = ',')]
    building_id: Vec<u64>,
}

#[derive(Deserialize)]
struct TerrainStages {
    input_digest: String,
    source_digest: String,
    ungraded_vista: adventuresim_tactical_core::scene_input::VistaSample,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::parse();
    let input = TacticalSceneInput::load(&request.scene_input)?;
    let stages: TerrainStages = serde_json::from_slice(&std::fs::read(&request.terrain_stages)?)?;
    if input.digest()? != stages.input_digest
        || input.source != SceneSource::ImportedPackage(stages.source_digest.clone())
    {
        return Err("stage provenance differs from the frozen production input".into());
    }
    let lod = stages
        .ungraded_vista
        .lods
        .iter()
        .find(|lod| lod.level == 0)
        .ok_or("near source vista is absent")?;
    let vista = GeographicSurface::from_vista_lod(lod).ok_or("invalid near vista")?;
    let raw = SceneTerrain::from_heightmap(
        usize::from(input.playable.width),
        usize::from(input.playable.depth),
        input.playable.spacing_metres,
        input.playable.heights_metres.clone(),
    )
    .ok_or("invalid production playable grid")?;
    let raw_surface = raw
        .sampled_geographic_surface()
        .ok_or("raw terrain is not sampled")?;
    let graded = input.generate_unfurnished(GeneratedBuildingRecipes::default())?;
    let graded_surface = graded
        .terrain
        .physical_geographic_surface()
        .ok_or("production terrain lacks physical support")?;

    let natural = input.prepare_geographic_terrain(&mut GeneratedBuildingRecipes::default())?;
    let natural_surface = natural
        .terrain
        .sampled_geographic_surface()
        .ok_or("complete-context terrain is not sampled")?;
    if let Some(path) = &request.physical_surface_output {
        std::fs::write(
            path,
            serde_json::to_vec(&graded_surface.triangles().collect::<Vec<_>>())?,
        )?;
    }
    let sources = [
        ("ungraded_near_vista", &vista),
        ("raw_playable", &raw_surface),
        ("complete_context_fine_playable", &natural_surface),
        ("production_bounded_support", &graded_surface),
    ];
    let enclosures = contacts::measure_enclosures(&input, &graded.terrain)?;
    let contacts = request
        .collision_probes
        .then(|| contacts::CornerSupportProbe::new(&graded.terrain));
    let mut buildings = Vec::new();
    for placement in &input.buildings {
        if !request.building_id.is_empty() && !request.building_id.contains(&placement.id) {
            continue;
        }
        buildings.push(inspect_building(
            &input,
            placement,
            &sources,
            contacts.as_ref(),
        )?);
    }
    if buildings.is_empty()
        || request
            .building_id
            .iter()
            .any(|id| !input.buildings.iter().any(|building| building.id == *id))
    {
        return Err("requested exact playable building is absent".into());
    }
    let presented =
        GeographicSurface::from_presented_scene(&natural.terrain, &stages.ungraded_vista)
            .ok_or("complete source presentation is invalid")?;
    let garden_measurements = gardens::measure(&input, &presented, &graded.terrain);
    let report: Value = json!({"input_digest":stages.input_digest,"source_digest":stages.source_digest,
        "seed":input.seed,"absolute_minute":input.absolute_minute,"schema":input.schema_version,
        "generation":input.generation_version,"vista_spacing_metres":lod.spacing_metres,
        "raw_playable_spacing_metres":raw.grid_scale(),
        "prepared_playable_spacing_metres":natural.terrain.grid_scale(),
        "current_production_repairs":graded.repairs,"geographic_preparation_repairs":natural.repairs,
        "scope":"Complete production context prepares geographic terrain before grading. Input, source identity, buildings, property reservations, gardens, streets and yards are retained. The physical surface is reconstructed by production generate_unfurnished. These measurements do not certify complete access, collision or renderer acceptance. Comparison coverage is explicit; partial footprints are not accepted.",
        "buildings":buildings,"gardens":garden_measurements,"enclosures":enclosures});
    std::fs::write(request.output, serde_json::to_vec_pretty(&report)?)?;
    if let Some(path) = request.prepared_terrain_output {
        let capture = PreparedTerrainCapture {
            purpose: TerrainPreparationPurpose::CompleteSceneContext,
            input_digest: input.digest()?,
            source_digest: stages.source_digest,
            terrain: natural.terrain,
        };
        capture.write_with_metadata(&path, &input, &stages.ungraded_vista)?;
    }
    Ok(())
}

fn inspect_building(
    input: &TacticalSceneInput,
    placement: &adventuresim_tactical_core::scene_input::TacticalBuildingPlacement,
    sources: &[(&str, &GeographicSurface); 4],
    contacts: Option<&contacts::CornerSupportProbe>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let plan = generate(&placement.program)?;
    let collision = adventuresim_building_generator::compile_building_collision(&plan);
    let footprint = collision
        .ground_floor_footprint()
        .ok_or("missing actual floor contact")?;
    let project = |p: Vec2| {
        placement.centre_metres
            + placement
                .orientation
                .local_to_world(p - collision.bounds.centre().xz())
    };
    let outline: Vec<_> = footprint.vertices().iter().copied().map(project).collect();
    let thresholds: Vec<_> = compile_ground_entrances(&plan).into_iter().map(|entry| {
            let position = project(entry.threshold_metres);
            json!({"entrance": entry.id, "support":entry.support, "position_metres":position,
                "elevations":sources.iter().map(|(name, source)| (*name, source.elevation_at(position)
                    .map(SupportElevation::metres))).collect::<std::collections::BTreeMap<_,_>>()})
        }).collect();
    let ranges: Vec<_> = sources
        .iter()
        .map(|(name, source)| {
            let range = source.height_range_in_outline(&outline);
            json!({"source":name,"minimum":range.map(|r|(r.minimum.0,r.minimum.1.metres())),
                "maximum":range.map(|r|(r.maximum.0,r.maximum.1.metres()))})
        })
        .collect();
    let comparisons: Vec<_> = [(0, 1), (0, 2), (1, 2), (2, 3)]
        .into_iter()
        .map(|(a, b)| {
            json!({"first":sources[a].0,"second":sources[b].0,
                "complete_triangle_overlay":sources[a].1.compare_in_outline(sources[b].1,&outline)})
        })
        .collect();
    Ok(json!({"building_id":placement.id,
            "property_id":input.properties.as_ref().and_then(|catalog|catalog.homes.iter()
                .find(|home|home.building_id==placement.id).map(|home|&home.id)),
            "placement":placement,"actual_bearing_outline_metres":outline,
            "ranges":ranges,"comparisons":comparisons,"thresholds":thresholds,
            "corner_support":contacts.map(|probe| probe.measure(&outline, placement.base_elevation_metres))}))
}
