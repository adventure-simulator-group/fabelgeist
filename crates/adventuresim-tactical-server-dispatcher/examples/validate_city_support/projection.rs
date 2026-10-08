//! Compact transport must reproduce the producer's exact accepted geometry.
use super::*;

pub(super) fn check(
    grounded: &GroundedCitySceneLayout,
    source: &GeographicSurface,
    output: Option<&std::path::Path>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let projection = grounded.support_projection()?;
    let encoded = serde_json::to_vec(&projection)?;
    let decoded: CityGroundingProjection = serde_json::from_slice(&encoded)?;
    let placements: Vec<_> = grounded
        .layout()
        .playable
        .iter()
        .cloned()
        .chain(
            grounded
                .layout()
                .distant
                .iter()
                .copied()
                .map(TacticalBuildingPlacement::from),
        )
        .collect();
    let started = Instant::now();
    let reconstructed = match decoded.reconstruct(source, &placements) {
        Ok(surface) => surface,
        Err(error) => {
            if let Some(path) = output {
                let details = match &error {
                    CityGroundingProjectionError::Surface { property, issue } => {
                        let original = grounded
                            .support_surfaces()
                            .iter()
                            .find(|surface| surface.property_id() == *property);
                        json!({"accepted":false,"complete_acceptance":false,
                            "property_id":property,"issue":issue.to_string(),
                            "accepted_producer_plan":original})
                    }
                    _ => json!({"accepted":false,"complete_acceptance":false,
                        "binding_error":error.to_string()}),
                };
                std::fs::write(
                    path.with_extension("rejected.json"),
                    serde_json::to_vec_pretty(&details)?,
                )?;
            }
            // Recording evidence never changes a rejection into acceptance.
            // Returning the original error preserves the CLI's failing exit.
            return Err(error.into());
        }
    };
    let seconds = started.elapsed().as_secs_f64();
    if &reconstructed != grounded.terrain() {
        return Err("compact reconstruction differs from exact producer geometry".into());
    }
    if let Some(path) = output {
        std::fs::write(path, &encoded)?;
    }
    Ok(
        json!({"bytes":encoded.len(),"reconstruction_seconds":seconds,
        "exact_geometry_equal":true,"source_and_placement_bindings_verified":true,
        "scope":"Native serial compact transport/reconstruction check, not browser execution or production installation."}),
    )
}
