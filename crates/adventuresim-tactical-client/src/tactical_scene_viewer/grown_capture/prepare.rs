//! Closed native CLI admission before any presenting world is created.
use super::*;
use crate::presentation::TreeContentRequest;

impl GrownCaptureOptions {
    fn admit_specimens(self, setup: &SceneSetupData) -> Result<Review> {
        let role = self.role.context("grown role required")?;
        let LevelSelection::Fixed(level) = self.level.context("grown level required")? else {
            anyhow::bail!("private specimen requires an explicit fixed level");
        };
        let control = self.control.context("grown batch control required")?;
        ensure!(
            control != controls::BatchControl::MissingInactiveTag && !control.requires_vista(),
            "selected control requires generated tree roots"
        );
        let source = PlantId::from_word(self.source.context("grown source required")?);
        let path = self.library.context("grown library required")?;
        let bytes = fs::read(&path)?;
        let bank = Arc::new(TreeBank::from_bytes(&bytes)?);
        let library = bank.library();
        let mut matches = library.models.iter().filter(|model| model.source == source);
        let model = matches.next().context("grown source missing")?;
        ensure!(matches.next().is_none(), "grown source ambiguous");
        ensure!(
            model.branches.count > 0 && model.leaves.count > 0,
            "leafy grown witness requires wood and foliage"
        );
        let bounds = model.bounds;
        let source_branches = model.branches;
        let source_leaves = model.leaves;
        let species = library
            .species
            .get(model.species.index())
            .context("grown species missing")?;
        let placements = layout::RootPlacements::on_terrain(&setup.generated.terrain)?;
        let constructed = role.construct(
            &bank,
            source,
            level,
            GrownMeshBudget {
                part_vertices: GrownPartVertexLimit::from_vertices(PART_VERTEX_LIMIT)?,
                total_mesh_bytes: GrownMeshBytes::from_bytes(FINAL_MESH_BYTE_LIMIT),
            },
        )?;
        let geometry = constructed.geometry;
        let metadata = serde_json::json!({
            "role":role,"level":geometry.level(),"source_id":source.word(),"variant":geometry.variant().index(),"bank_id":bank.id(),
            "library_bytes":bytes.len(),"species":species.name,"foliage":format!("{:?}",species.foliage),
            "bark_rgb":species.bark,"leaf_rgb":species.leaf,"bounds":bounds,
            "source_branches":source_branches,"source_leaves":source_leaves,
            "detail":library.detail,"mesh_payload_bytes":geometry.mesh_bytes().bytes(),
            "part_count":geometry.parts().len(),"part_vertex_limit":PART_VERTEX_LIMIT,"final_mesh_byte_limit":FINAL_MESH_BYTE_LIMIT,
            "vertex_encoding":format!("{:?}",geometry.encoding()),"authoring_adapter":constructed.authoring_adapter,
            "input_digest":setup.input.digest()?,"scene_digest":setup.generated.digest,"environment":setup.environment,
            "root_placements":placements,"suppressed_generated_obstacles":setup.generated.obstacles.len(),
            "time_phase":super::super::CAPTURE_CLOCK_PHASE_SECONDS,
            "draw_policy":"ordinary_gpu_membership","batch_control":control,"shadow_cutout":"analytic source alpha < 0.02 discarded; binary depth/shadow, fractional forward coverage",
            "source_terrain":"saved flat source growth; destination placement by grounded root translations only",
            "scope":"three grounded placements of one grown source asset set; actual main-pass instance membership diagnostic, no complete cache, species calibration, adaptive LOD, motion, memory, performance or arrival acceptance"
        });
        Ok(Review {
            role,
            level,
            control,
            source,
            geometry: Some(geometry),
            requested: Vec::new(),
            placements,
            metadata,
        })
    }
}

pub(in super::super) fn prepare(
    mut setup: SceneSetupData,
    options: Option<GrownCaptureOptions>,
    requested: &[String],
) -> Result<PreparedCapture> {
    let Some(options) = options else {
        return Ok(PreparedCapture {
            setup,
            review: None,
        });
    };
    let mode = options.mode.context("grown mode required")?;
    let review = match mode {
        CaptureMode::Specimens => {
            ensure!(
                options.second_source.is_none(),
                "specimens do not accept a second source"
            );
            view_selection::validate(&cameras::Cameras::slugs(), requested)
                .map_err(anyhow::Error::msg)?;
            let mut review = options.admit_specimens(&setup)?;
            review.requested = requested.to_vec();
            PreparedReview::Specimens(review)
        }
        CaptureMode::Generated => {
            let review = generated::Review::admit(options, &setup, requested)?;
            setup.tree_content = TreeContentRequest::Grown(review.request.clone());
            PreparedReview::Generated(review)
        }
    };
    setup.views.clear();
    setup.requested_views.clear();
    Ok(PreparedCapture {
        setup,
        review: Some(review),
    })
}
