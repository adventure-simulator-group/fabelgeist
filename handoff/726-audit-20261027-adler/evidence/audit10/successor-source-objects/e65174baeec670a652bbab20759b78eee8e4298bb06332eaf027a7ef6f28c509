//! Write verified preparation evidence; consumers share only the read contract.
use super::capture::PreparedTerrainCapture;
use adventuresim_tactical_core::prelude::*;
impl PreparedTerrainCapture {
    pub fn write_with_metadata(
        self,
        path: &std::path::Path,
        input: &TacticalSceneInput,
        ungraded_vista: &VistaSample,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = self.geographic_surface(input, ungraded_vista)?;
        let metadata = serde_json::json!({"triangles":source.triangles().count(),
            "source":"Exact complete-context fine terrain plus existing presented/stitched vista rings. Diagnostic only."});
        std::fs::write(
            path.with_extension("metadata.json"),
            serde_json::to_vec_pretty(&metadata)?,
        )?;
        std::fs::write(path, serde_json::to_vec(&self)?)?;
        Ok(())
    }
}
