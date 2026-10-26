//! Measure a composed surface and verify unowned source vertices independently.
use super::*;
use std::path::Path;

pub(super) struct Composition {
    pub accepted: bool,
    pub report: Value,
}

impl Composition {
    pub fn compile(
        selected: SelectedCityGrounding,
        geographic: &GeographicSurface,
        policy: CompoundGradingPolicy,
        output: Option<&Path>,
        placements_output: Option<&Path>,
        projection_output: Option<&Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let started = Instant::now();
        let grounded = match selected.compile() {
            Ok(grounded) => grounded,
            Err(error) => {
                return Ok(Self {
                    accepted: false,
                    report: json!({"composition_rejection": error}),
                });
            }
        };
        let compilation_seconds = started.elapsed().as_secs_f64();
        let projection_check = projection::check(&grounded, geographic, projection_output)?;
        let surface = grounded.terrain();
        let plans = grounded.support_surfaces();
        let plan_bytes = serde_json::to_vec(plans)?.len();
        if let Some(path) = placements_output {
            placements::write(path, grounded.layout())?;
        }
        let encoded = serde_json::to_vec(&surface)?;
        let started = Instant::now();
        let mut checksum = 0.0_f64;
        let mut query_count = 0;
        let mut unchanged_samples = 0;
        for triangle in geographic.triangles() {
            for point in triangle {
                let query = Vec2::new(point.x, point.z);
                let Some(hit) = surface.highest_surface_at(
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(
                        query,
                    )
                    .ok_or("nonfinite support query")?,
                ) else {
                    return Ok(Self {
                        accepted: false,
                        report: json!({
                            "source_query_missing": {"location_m":point,"constraint":"source_coverage"}
                        }),
                    });
                };
                let height = hit.elevation.metres();
                checksum += f64::from(height);
                query_count += 1;
                let scene_query =
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(query)?;
                if plans.iter().all(|plan| !plan.contains(scene_query)) {
                    let error = (height - point.y).abs();
                    let permitted = policy.limits.contact_tolerance_metres();
                    if error > permitted {
                        return Ok(Self {
                            accepted: false,
                            report: json!({"unowned_source_change":{
                                "location_m":query,"expected_m":point.y,"observed_m":height,
                                "measured_error_m":error,"permitted_m":permitted,"shortfall_m":error-permitted
                            }}),
                        });
                    }
                    unchanged_samples += 1;
                }
            }
        }
        let query_seconds = started.elapsed().as_secs_f64();
        if let Some(path) = output {
            std::fs::write(path, &encoded)?;
        }
        Ok(Self {
            accepted: true,
            report: json!({
                "plans": plans.iter().map(|plan| json!({
                    "property_id":plan.property_id(),"members":plan.member_building_ids(),
                    "regions":plan.support_regions(),"maximum_grade":plan.mesh().maximum_grade()
                })).collect::<Vec<_>>(),
                "composition":{
                    "seconds":compilation_seconds,"surface_json_bytes":encoded.len(),
                    "support_plan_json_bytes":plan_bytes,
                    "compact_round_trip":projection_check,
                    "seated_playable_buildings":grounded.layout().playable.len(),
                    "seated_distant_buildings":grounded.layout().distant.len(),
                    "foundation_owners":surface.foundations().len(),
                    "foundation_solid_triangles":surface.foundations().iter().map(|f| f.solid_triangles().len()).sum::<usize>(),
                    "source_triangles_after_clipping":surface.natural_triangles().len(),
                    "query_seconds":query_seconds,"queries":query_count,
                    "unchanged_unowned_vertex_queries":unchanged_samples,"query_height_checksum":checksum
                }
            }),
        })
    }
}
