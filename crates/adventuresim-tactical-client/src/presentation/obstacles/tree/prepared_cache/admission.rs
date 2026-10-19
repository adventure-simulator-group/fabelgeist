//! Bounded runtime admission of externally pinned, offline-verified content.
use super::*;
use std::{fs, io::Read, path::Path};

impl PreparedTreeCache {
    pub(crate) fn load(path: &Path, expected: &ExpectedTreeCache<'_>) -> Result<Self> {
        let started = Instant::now();
        let bytes = read_encoded(path)?;
        let digest = bytes.get(FORMAT.len()..FORMAT.len() + blake3::OUT_LEN);
        ensure!(
            digest.is_some_and(|digest| expected.content.matches_digest(digest)),
            "tree cache expected producer content mismatch"
        );
        let acquired = Instant::now();
        let mut checked = Self::decode(
            &bytes,
            expected.input,
            expected.environment,
            expected.source,
        )?;
        checked.measurements.encoded_file_read = acquired.duration_since(started);
        Ok(checked)
    }
}

/// CPU-only admission probe for a validated input identity. Physical scene
/// generation and tree-source regeneration have separate acceptance contracts.
pub(crate) fn admit_tree_cache(
    input_path: &Path,
    cache_path: &Path,
    source: PlantId,
    content: TreeCacheContentId,
) -> Result<()> {
    let started = Instant::now();
    let input = TacticalSceneInput::load(input_path)?;
    let input_loaded = Instant::now();
    let environment = input.environment_snapshot(input.digest()?);
    let expected = ExpectedTreeCache::new(&input, &environment, source, content);
    let environment_created = Instant::now();
    let checked = PreparedTreeCache::load(cache_path, &expected)?;
    let admitted = Instant::now();
    println!(
        "{}",
        serde_json::json!({"content_id":checked.content_id(),
        "source_id":source.word(), "complete_tiers_admitted":true,
        "probe_phases_microseconds": {
            "input_load":input_loaded.duration_since(started).as_micros(),
            "environment_request":environment_created.duration_since(input_loaded).as_micros(),
            "cache_load":admitted.duration_since(environment_created).as_micros(),
            "probe_total":admitted.duration_since(started).as_micros()},
        "cache_load_subphases":checked.measurements.report(),
        "scope":"CPU admission for validated input identity; no physical scene generation, renderer or arrival acceptance"})
    );
    Ok(())
}

pub(super) fn read_encoded(path: &Path) -> Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let length = file.metadata()?.len();
    ensure!(
        length <= MAX_COMPRESSED_BYTES,
        "compressed tree cache limit"
    );
    let mut bytes = vec![0; usize::try_from(length)?];
    file.read_exact(&mut bytes)?;
    ensure!(
        file.read(&mut [0])? == 0,
        "tree cache changed while reading"
    );
    Ok(bytes)
}
