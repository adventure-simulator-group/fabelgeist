//! Checked production-cache admission and source-derived cameras.
use super::*;

pub(super) fn load(options: Request, setup: &SceneSetupData) -> Result<Review> {
    let expected = ExpectedTreeCache::new(
        &setup.input,
        &setup.environment,
        options.source,
        options.content,
    );
    let cache = PreparedTreeCache::load(&options.directory.join("variant.treecache"), &expected)?;
    let readmission =
        if options.role == Role::Candidate && options.layout == layout::TreeLayout::Shared {
            Some(PreparedTreeCache::load(
                &options.directory.join("variant.treecache"),
                &expected,
            )?)
        } else {
            None
        };
    let source = TreeSourceGeometry::find(options.source, &setup.environment)?;
    let roots = options.layout.roots(&setup.generated.terrain)?;
    let cameras = cameras::Cameras::from_source(&source, &roots, &setup.generated.terrain)?;
    let metadata = serde_json::json!({
        "role": options.role, "source_id": options.source.word(),
        "species": source.species, "seed": source.seed.to_u64(),
        "input_digest": setup.input.digest()?, "scene_digest": setup.generated.digest,
        "environment": setup.environment, "cache_content_id": cache.content_id(),
        "root_metres": roots.first(), "roots_metres": roots, "layout": options.layout, "source_mesh_minimum": source.minimum,
        "source_mesh_maximum": source.maximum,
        "source_branches": source.branches, "source_cards": source.cards,
        "suppressed_generated_obstacles": setup.generated.obstacles.len(),
        "cameras": cameras, "selection": options.representation,
        "time_phase": super::super::CAPTURE_CLOCK_PHASE_SECONDS,
        "scope": "isolated static content; no adaptive, arrival or motion claim"
    });
    Ok(Review {
        role: options.role,
        layout: options.layout,
        source: options.source,
        representation: options.representation,
        cache: Some(cache),
        readmission,
        cameras,
        roots,
        metadata,
    })
}
