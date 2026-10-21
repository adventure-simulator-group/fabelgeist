//! Validate external fixed-view selections and temporal-prefix ownership.

use super::view_specs::CaptureViewSpec;
use std::collections::BTreeSet;

pub(super) enum ViewSequence {
    Independent,
    Prefix,
}

pub(super) fn select(
    profile_views: &[CaptureViewSpec],
    requested: &[String],
    sequence: ViewSequence,
) -> Result<Vec<CaptureViewSpec>, String> {
    validate(
        &profile_views
            .iter()
            .map(|view| view.slug)
            .collect::<Vec<_>>(),
        requested,
    )?;
    if requested.is_empty() {
        return Ok(profile_views.to_vec());
    }
    if matches!(sequence, ViewSequence::Prefix) {
        let mut last_requested_index = 0;
        for slug in requested {
            let index = profile_views
                .iter()
                .position(|view| view.slug == slug)
                .ok_or_else(|| format!("unknown requested view {slug}"))?;
            last_requested_index = last_requested_index.max(index);
        }
        // A cold-traversal plate is temporal evidence, not an independent
        // camera pose. Preserve every preceding approach/retreat frame so a
        // filtered capture cannot teleport from the distant-card warmup into
        // an unstreamed near tree and manufacture a floating crown.
        return Ok(profile_views[..=last_requested_index].to_vec());
    }
    let mut selected = vec![profile_views[0]];
    for slug in requested {
        let view = profile_views
            .iter()
            .find(|view| view.slug == slug)
            .copied()
            .ok_or_else(|| format!("unknown requested view {slug}"))?;
        selected.push(view);
    }
    Ok(selected)
}

pub(super) fn validate(profile_slugs: &[&str], requested: &[String]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for slug in requested {
        if slug == "warmup" {
            return Err("warmup is implicit and cannot be requested".into());
        }
        if !seen.insert(slug.as_str()) {
            return Err(format!("duplicate requested view {slug}"));
        }
        if !profile_slugs.contains(&slug.as_str()) {
            return Err(format!("unknown requested view {slug}"));
        }
    }
    Ok(())
}
