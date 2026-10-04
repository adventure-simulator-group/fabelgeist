use crate::data::gpu::shader::ShaderEntryPoint;
/// Cache identity retains the original entry-point hash within one pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PipelineEntryKey(u64);
impl From<&ShaderEntryPoint> for PipelineEntryKey {
    fn from(entry: &ShaderEntryPoint) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        entry.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::{Hash, Hasher};
    #[test]
    fn admitted_entry_preserves_the_original_string_hash() {
        for spelling in ["", "main", "entry_λ", "main "] {
            let mut original = std::collections::hash_map::DefaultHasher::new();
            spelling.hash(&mut original);
            assert_eq!(
                PipelineEntryKey::from(&ShaderEntryPoint::from(spelling)).0,
                original.finish()
            );
        }
    }
}
