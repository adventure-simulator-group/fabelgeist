//! Fixed-input acceptance probe before installation of geographic city support.
use adventuresim_tactical_core::{city_layout::grounding::*, city_layout::*, scene_input::*};
use bevy::math::Vec2;
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{path::PathBuf, time::Instant};
#[path = "compare_city_terrain/capture.rs"]
mod capture;
#[path = "validate_city_support/composition.rs"]
mod composition;
#[path = "validate_city_support/diagnostic.rs"]
mod diagnostic;
#[path = "validate_city_support/layout.rs"]
mod layout;
#[path = "validate_city_support/placements.rs"]
mod placements;
#[path = "validate_city_support/policy.rs"]
mod policy;
#[path = "validate_city_support/projection.rs"]
mod projection;

#[derive(Parser)]
struct Request {
    #[arg(long)]
    scene_input: PathBuf,
    #[arg(long)]
    world: PathBuf,
    #[arg(long)]
    terrain_stages: PathBuf,
    #[arg(long)]
    policy_fixture: PathBuf,
    #[arg(long)]
    output: PathBuf,
    /// Restrict reproduction to this exact property; omission checks all.
    #[arg(long)]
    property_id: Option<u64>,
    /// Explicit prepared-terrain capture with verified scene and source identity.
    #[arg(long)]
    prepared_terrain: Option<PathBuf>,
    /// Save accepted immutable geometry for bounded collision/render inspection.
    #[arg(long)]
    surface_output: Option<PathBuf>,
    /// Save exact seated placements for handoff inspection; not a scene input.
    #[arg(long)]
    grounded_placements_output: Option<PathBuf>,
    /// Save compact plans only after their exact reconstruction passes.
    #[arg(long)]
    support_projection_output: Option<PathBuf>,
}

#[derive(Deserialize)]
struct TerrainStages {
    input_digest: String,
    source_digest: adventuresim_tactical_core::scene_input::SourcePackageDigest,
    ungraded_vista: VistaSample,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::parse();
    let input = TacticalSceneInput::load(&request.scene_input)?;
    let digest = input.digest()?;
    let stages: TerrainStages = serde_json::from_slice(&std::fs::read(&request.terrain_stages)?)?;
    if stages.input_digest != digest
        || input.source != SceneSource::ImportedPackage(stages.source_digest.clone())
    {
        return Err("terrain stages must identify this exact input and source".into());
    }
    let lod = stages
        .ungraded_vista
        .lods
        .iter()
        .find(|lod| lod.level == adventuresim_tactical_core::scene_input::VistaLevelIndex::new(0))
        .ok_or("source near vista is absent")?;
    let geographic = if let Some(path) = &request.prepared_terrain {
        let capture: capture::PreparedTerrainCapture =
            serde_json::from_slice(&std::fs::read(path)?)?;
        capture.geographic_surface(&input, &stages.ungraded_vista)?
    } else {
        GeographicSurface::from_vista_lod(lod).ok_or("invalid source triangles")?
    };
    let policy = policy::read(&request.policy_fixture)?;
    let mut layout = layout::reproduce(&input, &request.world)?;
    if let Some(id) = request.property_id {
        layout.compounds.retain(|p| p.id.0 == id);
        layout.single_properties.retain(|p| p.id.0 == id);
        if layout.compounds.len() + layout.single_properties.len() != 1 {
            return Err("requested exact property is absent or duplicated".into());
        }
        let members: std::collections::BTreeSet<_> = layout
            .compounds
            .iter()
            .flat_map(|p| [p.front_building_id, p.rear_building_id])
            .chain(layout.single_properties.iter().map(|p| p.building_id))
            .collect();
        layout.playable.retain(|p| members.contains(&p.id));
        layout.distant.retain(|p| members.contains(&p.id));
        layout
            .gardens
            .retain(|p| members.contains(&p.front_building_id));
        layout
            .businesses
            .retain(|p| members.contains(&p.building_id));
    }
    let start = Instant::now();
    let selected = SelectedCityGrounding::select(&layout, &geographic, policy);
    let seconds = start.elapsed().as_secs_f64();
    let mut accepted = selected.is_ok();
    let outcome = match selected {
        Ok(selected) => {
            let composed = composition::Composition::compile(
                selected,
                &geographic,
                policy,
                request.surface_output.as_deref(),
                request.grounded_placements_output.as_deref(),
                request.support_projection_output.as_deref(),
            )?;
            accepted = composed.accepted;
            composed.report
        }
        Err(CityGroundingError::Binding(CitySupportError::Support(error))) => {
            diagnostic::failed_property(&input, &layout, error)?
        }
        Err(error) => json!({"binding_failure":error.to_string()}),
    };
    let report = json!({"fixture":input.properties.as_ref().map(|p| &p.settlement_id),"scene_key":input.scene_key,"input_digest":digest,"source_digest":stages.source_digest,
        "schema":input.schema_version,"generation":input.generation_version,"seed":input.seed,"absolute_minute":input.absolute_minute,
        "source_preparation":if request.prepared_terrain.is_some() { "explicit prepared fine playable and presented vista rings; preparation purpose retained in the capture" } else { "ungraded near vista only" },"source_level":lod.level,"source_spacing_m":lod.spacing_metres,"source_triangles":geographic.triangles().count(),
        "properties":layout.compounds.len()+layout.single_properties.len(),"execution":"native serial support planning; no browser worker execution",
        "seconds":seconds,"support_pass":accepted,"outcome":outcome,
        "scope":"Original source/layout unchanged. Accepted projection seats exact member floors after complete support composition. Not production scene installation, collision/enclosure, full access, capacity, visual or performance acceptance.","property_scope":request.property_id});
    std::fs::write(&request.output, serde_json::to_vec_pretty(&report)?)?;
    if !accepted {
        return Err(
            "required positive support remains rejected; see exact output constraint".into(),
        );
    }
    Ok(())
}
