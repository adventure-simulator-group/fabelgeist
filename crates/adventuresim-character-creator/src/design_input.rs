//! Reproducible design inputs shared by studio and headless exports.

use std::path::Path;

use adventuresim_armor_model::{BreastplateDesign, validate_breastplate};
use anyhow::{Context, Result};

pub fn load_breastplate_design(path: Option<&Path>) -> Result<BreastplateDesign> {
    let Some(path) = path else {
        return Ok(BreastplateDesign::default());
    };
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading breastplate design {}", path.display()))?;
    parse_breastplate_design(&bytes)
        .with_context(|| format!("loading breastplate design {}", path.display()))
}

fn parse_breastplate_design(bytes: &[u8]) -> Result<BreastplateDesign> {
    let design = serde_json::from_slice(bytes).context("parsing breastplate design JSON")?;
    validate_breastplate(&design).context("invalid breastplate design parameters")?;
    Ok(design)
}

#[cfg(test)]
mod tests {
    use adventuresim_armor_model::Millimeters;

    use super::*;

    #[test]
    fn absent_path_keeps_the_existing_default() {
        assert_eq!(
            load_breastplate_design(None).unwrap(),
            BreastplateDesign::default()
        );
    }

    #[test]
    fn json_preserves_an_individual_parameter_override() {
        let expected = BreastplateDesign {
            skirt_flare: Millimeters(60),
            ..BreastplateDesign::default()
        };
        let actual = parse_breastplate_design(&serde_json::to_vec(&expected).unwrap()).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn checked_in_example_matches_the_default() {
        let example = include_bytes!("../breastplate-design.example.json");
        assert_eq!(
            parse_breastplate_design(example).unwrap(),
            BreastplateDesign::default()
        );
    }

    #[test]
    fn malformed_or_out_of_range_designs_are_not_silently_defaulted() {
        assert!(parse_breastplate_design(b"{}").is_err());
        assert!(parse_breastplate_design(b"not JSON").is_err());
        let invalid = BreastplateDesign {
            wall_thickness: Millimeters(0),
            ..BreastplateDesign::default()
        };
        assert!(parse_breastplate_design(&serde_json::to_vec(&invalid).unwrap()).is_err());
    }

    #[test]
    fn misspelled_overrides_and_duplicate_fields_are_rejected() {
        let mut document = serde_json::to_value(BreastplateDesign::default()).unwrap();
        document["skirt_flair"] = serde_json::json!(60);
        assert!(parse_breastplate_design(&serde_json::to_vec(&document).unwrap()).is_err());
        let valid = serde_json::to_string(&BreastplateDesign::default()).unwrap();
        let duplicate = valid.replacen('{', "{\"skirt_flare\":60,", 1);
        assert!(parse_breastplate_design(duplicate.as_bytes()).is_err());
    }
}
