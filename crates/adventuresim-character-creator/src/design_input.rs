//! Reproducible design inputs shared by studio and headless exports.

use std::path::Path;

use adventuresim_armor_model::{BreastplateDesign, validate_breastplate};
use anyhow::{Context, Result};

pub fn load_breastplate_design(path: Option<&Path>) -> Result<BreastplateDesign> {
    let Some(path) = path else {
        return parse_breastplate_design(include_bytes!(
            "../../../assets_src/equipment/breastplate-design.json"
        ))
        .context("invalid authored breastplate recipe");
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

pub fn load_bracer_design(path: Option<&Path>) -> Result<adventuresim_armor_model::BracerDesign> {
    let Some(path) = path else {
        return parse_bracer_design(include_bytes!(
            "../../../assets_src/equipment/vambrace-design.json"
        ))
        .context("invalid authored vambrace recipe");
    };
    parse_bracer_design(&std::fs::read(path)?)
        .with_context(|| format!("loading vambrace design {}", path.display()))
}

fn parse_bracer_design(bytes: &[u8]) -> Result<adventuresim_armor_model::BracerDesign> {
    let design = serde_json::from_slice(bytes).context("parsing vambrace design")?;
    adventuresim_armor_model::validate(&design).context("invalid vambrace design")?;
    Ok(design)
}

#[cfg(test)]
mod tests {
    use adventuresim_armor_model::Millimeters;

    use super::*;

    #[test]
    fn misspelled_optional_vambrace_fluting_is_rejected() {
        let mut document =
            serde_json::to_value(adventuresim_armor_model::BracerDesign::default()).unwrap();
        document.as_object_mut().unwrap().remove("fluting");
        document["flutng"] = serde_json::json!({"count":9});
        assert!(
            serde_json::from_value::<adventuresim_armor_model::BracerDesign>(document).is_err()
        );
    }

    #[test]
    fn absent_paths_load_valid_authored_equipment_recipes() {
        let breastplate = load_breastplate_design(None).unwrap();
        let vambrace = load_bracer_design(None).unwrap();
        assert_eq!(
            breastplate,
            parse_breastplate_design(include_bytes!(
                "../../../assets_src/equipment/breastplate-design.json"
            ))
            .unwrap()
        );
        assert_eq!(
            vambrace,
            parse_bracer_design(include_bytes!(
                "../../../assets_src/equipment/vambrace-design.json"
            ))
            .unwrap()
        );
        assert_ne!(breastplate, BreastplateDesign::default());
        assert_ne!(vambrace, adventuresim_armor_model::BracerDesign::default());
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
