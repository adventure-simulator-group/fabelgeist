//! Bounded runtime admission of externally pinned, offline-verified content.
use super::*;
use std::{fs, io::Read, path::Path};

impl PreparedTreeCache {
    pub(crate) fn load(path: &Path, expected: &ExpectedTreeCache<'_>) -> Result<Self> {
        let bytes = read_encoded(path)?;
        let digest = bytes.get(FORMAT.len()..FORMAT.len() + blake3::OUT_LEN);
        ensure!(
            digest.is_some_and(|digest| expected.content.matches_digest(digest)),
            "tree cache expected producer content mismatch"
        );
        Self::decode(
            &bytes,
            expected.input,
            expected.environment,
            expected.source,
        )
    }
}

/// CPU-only runtime admission probe. Scene generation supplies the same live
/// environment binding; source-tree regeneration belongs to offline inspection.
pub(crate) fn admit_tree_cache(
    input_path: &Path,
    cache_path: &Path,
    source: PlantId,
    content: TreeCacheContentId,
) -> Result<()> {
    let input = TacticalSceneInput::load(input_path)?;
    let generated = input.generate()?;
    let environment = input.environment_snapshot(generated.digest);
    let expected = ExpectedTreeCache::new(&input, &environment, source, content);
    let checked = PreparedTreeCache::load(cache_path, &expected)?;
    println!(
        "{}",
        serde_json::json!({"content_id":checked.content_id(),
        "source_id":source.word(), "complete_tiers_admitted":true,
        "scope":"CPU runtime admission of independently pinned offline-verified content; no renderer or arrival acceptance"})
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
