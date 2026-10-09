//! The wardrobe: garments draped and settled once in the studio, saved by
//! name. Inventory articles copy a saved garment, so a recipe never depends
//! on the wardrobe.

use crate::garment::SettledGarment;
pub use crate::library_name::LibraryName;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

/// Named settled garments.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Wardrobe {
    entries: BTreeMap<LibraryName, SettledGarment>,
}

impl Wardrobe {
    /// The wardrobe saved at `path`; a wardrobe not yet saved there is empty.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let wardrobe: Self = serde_json::from_slice(&std::fs::read(path)?)
            .with_context(|| format!("parsing wardrobe {}", path.display()))?;
        wardrobe
            .validate()
            .with_context(|| format!("checking wardrobe {}", path.display()))?;
        Ok(wardrobe)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(self)?)
            .with_context(|| format!("saving {}", path.display()))
    }

    fn validate(&self) -> Result<()> {
        for (name, garment) in &self.entries {
            anyhow::ensure!(
                garment.selection.name == name.as_ref(),
                "{name} is saved as {}",
                garment.selection.name
            );
            garment
                .validate()
                .with_context(|| format!("invalid garment {name}"))?;
        }
        Ok(())
    }

    /// Save `garment` under `name`, which also becomes its name, replacing any
    /// garment named so.
    pub fn insert(&mut self, name: LibraryName, mut garment: SettledGarment) -> Result<()> {
        garment.selection.name = name.to_string();
        garment.validate()?;
        self.entries.insert(name, garment);
        Ok(())
    }

    pub fn remove(&mut self, name: &LibraryName) -> Option<SettledGarment> {
        self.entries.remove(name)
    }

    pub fn get(&self, name: &LibraryName) -> Option<&SettledGarment> {
        self.entries.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&LibraryName, &SettledGarment)> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garment::{
        DrapeInput, DrapeStage, DrapedGarment, FabricPreset, GarmentForm, GarmentSelection,
    };

    fn name(name: &str) -> LibraryName {
        LibraryName::try_from(name.to_owned()).unwrap()
    }

    /// One triangle of cloth settled above a tetrahedron.
    fn garment() -> SettledGarment {
        let positions = vec![[0., 0., 0.], [1., 0., 0.], [0., 0., 1.], [0., -1., 0.]];
        let input = DrapeInput {
            under_plate: None,
            selection: GarmentSelection::default(),
            settled: None,
            obstacles: vec![],
            indices: vec![[0; 8]; positions.len()],
            weights: vec![[1., 0., 0., 0., 0., 0., 0., 0.]; positions.len()],
            positions,
            faces: vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [2, 0, 3]],
            names: vec![],
            joints: vec![],
        };
        let cloth = DrapedGarment {
            form: GarmentForm::Upper,
            name: "Cloth".into(),
            fabric: FabricPreset::Cotton,
            texcoords: vec![[0., 0.], [1., 0.], [0., 1.]],
            positions: vec![[0.1, 0.1, 0.1], [0.5, 0.1, 0.1], [0.1, 0.1, 0.5]],
            normals: vec![],
            faces: vec![[0, 2, 1]],
            indices: vec![],
            weights: vec![],
            stage: DrapeStage::Settling { step: 1, of: 1 },
        };
        SettledGarment::capture(&input, &cloth).unwrap()
    }

    #[test]
    fn a_saved_wardrobe_reloads_with_garments_named_as_saved() {
        let dir = std::env::temp_dir().join(format!("wardrobe-{}", std::process::id()));
        let path = dir.join("nested").join("wardrobe.json");
        assert!(Wardrobe::load(&path).unwrap().is_empty());

        let mut wardrobe = Wardrobe::default();
        wardrobe.insert(name(" Linen shirt "), garment()).unwrap();
        wardrobe.save(&path).unwrap();
        let loaded = Wardrobe::load(&path).unwrap();
        assert_eq!(loaded, wardrobe);
        let saved = loaded.get(&name("Linen shirt")).unwrap();
        assert_eq!(saved.selection.name, "Linen shirt");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn invalid_garments_are_not_saved() {
        let mut broken = garment();
        broken.selection.resolution_cm = f32::NAN;
        let mut wardrobe = Wardrobe::default();
        assert!(wardrobe.insert(name("Broken"), broken).is_err());
        assert!(wardrobe.is_empty());
    }
}
