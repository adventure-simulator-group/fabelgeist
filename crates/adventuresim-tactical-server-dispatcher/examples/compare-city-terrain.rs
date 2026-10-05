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
        .find(|lod| lod.level == adventuresim_tactical_core::scene_input::VistaLevelIndex::new(0))
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
        StageSource {
            role: TerrainStageRole::UngradedNearVista,
            surface: &vista,
        },
        StageSource {
            role: TerrainStageRole::RawPlayable,
            surface: &raw_surface,
        },
        StageSource {
            role: TerrainStageRole::PreparedFinePlayable,
            surface: &natural_surface,
        },
        StageSource {
            role: TerrainStageRole::BoundedSupport,
            surface: &graded_surface,
        },
    ];
    let enclosures = contacts::measure_enclosures(&input, &graded.terrain)?;
    let contacts = request
        .collision_probes
        .then(|| contacts::CornerSupportProbe::new(&graded.terrain));
    let buildings = request.selected_building_reports(&input, &sources, contacts.as_ref())?;
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
    sources: &[StageSource<'_>; 4],
    contacts: Option<&contacts::CornerSupportProbe>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let plan = generate(&placement.program)?;
    let collision = adventuresim_building_generator::compile_building_collision(&plan);
    let footprint = collision
        .ground_floor_footprint()?
        .ok_or("missing actual floor contact")?;
    let project = |p: Vec2| {
        placement.centre_metres
            + placement
                .orientation
                .local_to_world(p - collision.bounds.centre().xz())
    };
    let projection =
        adventuresim_tactical_core::scene_coordinates::ArchitecturalPlanProjection::from_placement(
            placement,
            collision.bounds,
        )?;
    let scene_outline =
        adventuresim_tactical_core::scene_coordinates::ScenePlanPolygon::from_architectural(
            footprint.polygon(),
            projection,
        )?;
    let outline: Vec<_> = scene_outline
        .vertices()
        .iter()
        .map(|point| point.metres())
        .collect();
    let thresholds: Vec<_> = compile_ground_entrances(&plan).into_iter().map(|entry| {
            let position = project(entry.threshold_metres);
            json!({"entrance": entry.id, "support":entry.support, "position_metres":position,
                "elevations":sources.iter().map(|source| (source.role.output_name(), source.surface.elevation_at(position)
                    .map(SupportElevation::metres))).collect::<std::collections::BTreeMap<_,_>>()})
        }).collect();
    let ranges: Vec<_> = sources
        .iter()
        .map(|source| {
            let range = source.surface.height_range_in_outline(&outline);
            json!({"source":source.role.output_name(),"minimum":range.map(|r|(r.minimum.point.metres(),r.minimum.elevation.metres())),
                "maximum":range.map(|r|(r.maximum.point.metres(),r.maximum.elevation.metres()))})
        })
        .collect();
    let comparisons: Vec<_> = [
        StageComparison {
            first: TerrainStageRole::UngradedNearVista,
            second: TerrainStageRole::RawPlayable,
        },
        StageComparison {
            first: TerrainStageRole::UngradedNearVista,
            second: TerrainStageRole::PreparedFinePlayable,
        },
        StageComparison {
            first: TerrainStageRole::RawPlayable,
            second: TerrainStageRole::PreparedFinePlayable,
        },
        StageComparison {
            first: TerrainStageRole::PreparedFinePlayable,
            second: TerrainStageRole::BoundedSupport,
        },
    ]
    .into_iter()
    .map(|comparison| {
        let first = comparison.first.source(sources);
        let second = comparison.second.source(sources);
        json!({"first":comparison.first.output_name(), "second":comparison.second.output_name(),
            "complete_triangle_overlay":first.compare_in_outline(second, &outline)})
    })
    .collect();
    let floor = SupportElevation::from_metres(placement.base_elevation_metres)
        .ok_or(adventuresim_building_generator::plan_geometry::PlanGeometryError::NonFinite)?;
    let corner_support = contacts
        .map(|probe| probe.measure(&scene_outline, floor))
        .transpose()?;
    Ok(json!({"building_id":placement.id,
            "property_id":input.properties.as_ref().and_then(|catalog|catalog.homes.iter()
                .find(|home|home.building_id==placement.id).map(|home|&home.id)),
            "placement":placement,"actual_bearing_outline_metres":outline,
            "ranges":ranges,"comparisons":comparisons,"thresholds":thresholds,
            "corner_support":corner_support}))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerrainStageRole {
    UngradedNearVista,
    RawPlayable,
    PreparedFinePlayable,
    BoundedSupport,
}
impl TerrainStageRole {
    fn output_name(self) -> &'static str {
        match self {
            Self::UngradedNearVista => "ungraded_near_vista",
            Self::RawPlayable => "raw_playable",
            Self::PreparedFinePlayable => "complete_context_fine_playable",
            Self::BoundedSupport => "production_bounded_support",
        }
    }
    fn source<'a>(self, sources: &[StageSource<'a>; 4]) -> &'a GeographicSurface {
        sources
            .iter()
            .find(|source| source.role == self)
            .expect("all four processing stages are supplied")
            .surface
    }
}
struct StageSource<'a> {
    role: TerrainStageRole,
    surface: &'a GeographicSurface,
}
struct StageComparison {
    first: TerrainStageRole,
    second: TerrainStageRole,
}

#[cfg(test)]
mod stage_tests {
    use super::*;
    use bevy::math::Vec3;
    #[test]
    fn comparison_stages_are_selected_by_role_when_sources_are_reordered() {
        let surface = |height| {
            GeographicSurface::from_triangles([[
                Vec3::new(0.0, height, 0.0),
                Vec3::new(1.0, height, 0.0),
                Vec3::new(0.0, height, 1.0),
            ]])
            .unwrap()
        };
        let vista = surface(1.0);
        let raw = surface(2.0);
        let prepared = surface(3.0);
        let bounded = surface(4.0);
        let sources = [
            StageSource {
                role: TerrainStageRole::BoundedSupport,
                surface: &bounded,
            },
            StageSource {
                role: TerrainStageRole::PreparedFinePlayable,
                surface: &prepared,
            },
            StageSource {
                role: TerrainStageRole::RawPlayable,
                surface: &raw,
            },
            StageSource {
                role: TerrainStageRole::UngradedNearVista,
                surface: &vista,
            },
        ];
        assert_eq!(
            TerrainStageRole::UngradedNearVista
                .source(&sources)
                .elevation_at(Vec2::splat(0.25))
                .unwrap()
                .metres(),
            1.0
        );
        assert_eq!(
            TerrainStageRole::BoundedSupport
                .source(&sources)
                .elevation_at(Vec2::splat(0.25))
                .unwrap()
                .metres(),
            4.0
        );
    }
}

impl Request {
    /// Require every explicitly requested building before producing a capture.
    fn selected_building_reports(
        &self,
        input: &TacticalSceneInput,
        sources: &[StageSource<'_>; 4],
        contacts: Option<&contacts::CornerSupportProbe>,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let mut buildings = Vec::new();
        for placement in &input.buildings {
            if !self.building_id.is_empty() && !self.building_id.contains(&placement.id) {
                continue;
            }
            buildings.push(inspect_building(input, placement, sources, contacts)?);
        }
        if buildings.is_empty()
            || self
                .building_id
                .iter()
                .any(|id| !input.buildings.iter().any(|building| building.id == *id))
        {
            return Err("requested exact playable building is absent".into());
        }
        Ok(buildings)
    }
}
