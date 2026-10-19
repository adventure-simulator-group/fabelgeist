//! Offline mature production sources admitted into the prepared renderer.
//!
//! This local content pilot approximates source cards as analytic clusters and
//! retains plain tube junctions and generic materials. It does not replace the
//! production presenters or establish equivalent appearance.

use std::{fs, path::Path};

use adventuresim_environment::prepared::{
    VariantBudget, encode_library, encode_population, encode_unique,
};
use adventuresim_tactical_core::scene_input::TacticalSceneInput;
use anyhow::{Context, Result};
use serde_json::json;

mod evidence;
mod records;
mod review;

pub(crate) use review::{TreeSourceGeometry, TreeSourcePart};
pub(super) mod specimens;

pub(crate) const SOURCE_FORMAT: &str = "production-source-bank-pilot-v1";

pub(crate) fn write_tree_bank(input_path: &Path, output: &Path) -> Result<()> {
    let input = TacticalSceneInput::load(input_path)?;
    let generated = input.generate()?;
    let environment = input.environment_snapshot(generated.digest.clone());
    let mut prepared = specimens::prepare(&environment)?;
    let library = prepared.unique.bake(VariantBudget::AllPlants)?;
    prepared.bind_model_bounds(&library)?;
    let unique_bytes = encode_unique(&prepared.unique)?;
    let (library_bytes, library_id) = encode_library(&library)?;
    let population = prepared.unique.population(&library, library_id)?;
    let population_bytes = encode_population(&population, &library, library_id)?;
    let report = json!({
        "format": SOURCE_FORMAT,
        "input_digest": input.digest()?,
        "scene_digest": generated.digest,
        "environment": environment,
        "library": library_id,
        "models": library.models.len(),
        "mesh_bytes": library.mesh_bytes(),
        "library_bytes": library_bytes.len(),
        "population_bytes": population_bytes.len(),
        "source_normalization_metres": specimens::source_normalization().to_array(),
        "source_specimens": prepared.evidence,
        "source_geometry_frame": "collider-centred metres before one half-trunk normalization",
        "leaf_envelopes": "logical rectangles/petioles and actual unanimated production flat/cambered mesh vertices; no opacity clipping",
        "converted_models": library.models.iter().map(|model| json!({
            "source_id": model.source.word(),
            "species": model.species.word(),
            "bounds": model.bounds,
            "branches": model.branches,
            "clusters": model.leaves,
        })).collect::<Vec<_>>(),
        "limits": [
            "source foliage uses an approximate source-box-area cluster heuristic, not preserved individual transforms",
            "root aspect/lift, transported frames, collars, taper and rounded caps are not fully encoded",
            "generic botany materials do not preserve production bark, opacity, transmission or contact",
            "no runtime replacement, biological growth, strategic season or quality/performance acceptance"
        ]
    });
    // A fresh directory is part of the authoring contract: never replace assets.
    fs::create_dir(output)
        .with_context(|| format!("create fresh tree bank {}", output.display()))?;
    for (name, bytes) in [
        ("unique.forest", unique_bytes),
        ("library.forest", library_bytes),
        ("population.forest", population_bytes),
        ("source.json", serde_json::to_vec_pretty(&report)?),
    ] {
        fs::write(output.join(name), bytes)?;
    }
    println!("TREE_BANK_OUTPUT={}", output.display());
    Ok(())
}
