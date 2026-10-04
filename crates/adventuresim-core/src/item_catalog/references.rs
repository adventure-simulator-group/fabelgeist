//! Ordered cross-catalog admission retains every missing item identity.

use super::{ItemDefinitionId, definition};

#[derive(Debug, Eq, PartialEq)]
pub struct MissingItemDefinitions {
    ids: Vec<ItemDefinitionId>,
}

impl MissingItemDefinitions {
    pub fn as_slice(&self) -> &[ItemDefinitionId] {
        &self.ids
    }
}

impl std::fmt::Display for MissingItemDefinitions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, id) in self.ids.iter().enumerate() {
            if index != 0 {
                formatter.write_str(", ")?;
            }
            id.fmt(formatter)?;
        }
        Ok(())
    }
}

impl std::error::Error for MissingItemDefinitions {}

pub fn validate_references(references: &[ItemDefinitionId]) -> Result<(), MissingItemDefinitions> {
    let mut missing = Vec::new();
    for id in references {
        if definition(id).is_none() {
            missing.push(id.clone());
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(MissingItemDefinitions { ids: missing })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_admission_retains_missing_order_duplicates_and_exact_spelling() {
        let references =
            ["missing_b", "torch", "Missing_a", "missing_b"].map(ItemDefinitionId::from);
        let error = validate_references(&references).unwrap_err();
        assert_eq!(
            error.as_slice(),
            &[
                ItemDefinitionId::from("missing_b"),
                ItemDefinitionId::from("Missing_a"),
                ItemDefinitionId::from("missing_b"),
            ]
        );
        assert_eq!(error.to_string(), "missing_b, Missing_a, missing_b");
        assert!(validate_references(&[]).is_ok());
    }
}
