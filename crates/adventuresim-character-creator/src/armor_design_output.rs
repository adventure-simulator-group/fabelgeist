//! Validate a complete studio design set before writing separate recipe files.
use crate::armor_design_input::{self, ArmorDesigns};
use adventuresim_armor_model::{BracerDesign, BreastplateDesign, validate, validate_breastplate};
use anyhow::{Context, Result, ensure};
use std::path::{Path, PathBuf};

pub struct DesignPaths<'a> {
    pub catalog: &'a Path,
    pub bracer: &'a Path,
    pub breastplate: &'a Path,
}

impl DesignPaths<'_> {
    pub fn save(
        &self,
        catalog: &ArmorDesigns,
        bracer: &BracerDesign,
        breastplate: &BreastplateDesign,
    ) -> Result<()> {
        let catalog_bytes = armor_design_input::encode(catalog)?;
        validate(bracer)?;
        validate_breastplate(breastplate)?;
        let paths = [self.catalog, self.bracer, self.breastplate];
        let resolved = paths.map(resolve);
        let resolved = resolved.into_iter().collect::<Result<Vec<_>>>()?;
        ensure!(
            resolved
                .iter()
                .enumerate()
                .all(|(i, path)| !resolved[i + 1..].contains(path)),
            "design files must have distinct paths"
        );
        let bytes = [
            catalog_bytes,
            serde_json::to_vec_pretty(bracer)?,
            serde_json::to_vec_pretty(breastplate)?,
        ];
        for (path, bytes) in paths.into_iter().zip(bytes) {
            std::fs::write(path, bytes).with_context(|| format!("saving {}", path.display()))?;
        }
        Ok(())
    }
}

fn resolve(path: &Path) -> Result<PathBuf> {
    let resolved = if path.exists() {
        path.canonicalize()?
    } else {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        parent
            .canonicalize()?
            .join(path.file_name().context("design path needs a file name")?)
    };
    // Windows canonicalization resolves directory aliases but retains new-file casing.
    #[cfg(windows)]
    let resolved = PathBuf::from(resolved.as_os_str().to_string_lossy().to_lowercase());
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_are_rejected_before_any_write_and_distinct_designs_reload() {
        let dir = std::env::temp_dir().join(format!("armor-save-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("catalog.json");
        let alias = dir.join(".").join("catalog.json");
        let b = dir.join("bracer.json");
        let c = dir.join("breastplate.json");
        std::fs::write(&a, b"original").unwrap();
        let catalog = ArmorDesigns::new();
        let bracer = BracerDesign {
            center_ridge: adventuresim_armor_model::Millimeters(4),
            ..Default::default()
        };
        let breastplate = BreastplateDesign::default();
        assert!(
            DesignPaths {
                catalog: &a,
                bracer: &alias,
                breastplate: &c
            }
            .save(&catalog, &bracer, &breastplate)
            .is_err()
        );
        assert_eq!(std::fs::read(&a).unwrap(), b"original");
        DesignPaths {
            catalog: &a,
            bracer: &b,
            breastplate: &c,
        }
        .save(&catalog, &bracer, &breastplate)
        .unwrap();
        assert_eq!(
            crate::design_input::load_bracer_design(Some(&b)).unwrap(),
            bracer
        );
        assert_eq!(
            crate::design_input::load_breastplate_design(Some(&c)).unwrap(),
            breastplate
        );
        assert!(
            crate::armor_design_input::load(Some(&a))
                .unwrap()
                .is_empty()
        );
        for path in [a, b, c] {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_dir(dir).unwrap();
    }
}
