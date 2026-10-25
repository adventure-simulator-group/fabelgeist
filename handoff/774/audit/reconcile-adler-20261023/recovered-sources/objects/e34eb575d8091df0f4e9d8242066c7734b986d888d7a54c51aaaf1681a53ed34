//! Parse and compare typed demographic selectors from authored quest data.

use adventuresim_world_schema::Sex;
use serde_json::{Map, Value};
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum SexSelectorError {
    Missing,
    ExpectedArray,
    ExpectedString(usize),
    Unknown { index: usize, value: String },
}

impl SexSelectorError {
    pub(super) fn at(self, rule_path: &str) -> String {
        format!("{rule_path}.sexes: {self}")
    }
}

impl fmt::Display for SexSelectorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => formatter.write_str("missing"),
            Self::ExpectedArray => formatter.write_str("expected array"),
            Self::ExpectedString(index) => write!(formatter, "entry {index}: expected string"),
            Self::Unknown { index, value } => {
                write!(formatter, "entry {index}: unknown sex selector {value}")
            }
        }
    }
}

pub(super) fn parse_sexes(rule: &Map<String, Value>) -> Result<Vec<Sex>, SexSelectorError> {
    let values = rule.get("sexes").ok_or(SexSelectorError::Missing)?;
    let values = values.as_array().ok_or(SexSelectorError::ExpectedArray)?;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let value = value
                .as_str()
                .ok_or(SexSelectorError::ExpectedString(index))?;
            value.parse().map_err(|_| SexSelectorError::Unknown {
                index,
                value: value.to_owned(),
            })
        })
        .collect()
}

pub(super) fn sex_selectors_overlap(left: &[Sex], right: &[Sex]) -> bool {
    left.is_empty() || right.is_empty() || left.iter().any(|sex| right.contains(sex))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_parser_reports_typed_shape_and_value_errors() {
        let mut rule = Map::new();
        assert_eq!(parse_sexes(&rule), Err(SexSelectorError::Missing));
        rule.insert("sexes".into(), Value::Null);
        assert_eq!(parse_sexes(&rule), Err(SexSelectorError::ExpectedArray));
        rule.insert("sexes".into(), serde_json::json!(["female", 1]));
        assert_eq!(parse_sexes(&rule), Err(SexSelectorError::ExpectedString(1)));
        rule.insert("sexes".into(), serde_json::json!(["female", "unknown"]));
        assert_eq!(
            parse_sexes(&rule),
            Err(SexSelectorError::Unknown {
                index: 1,
                value: "unknown".into(),
            })
        );
        rule.insert("sexes".into(), serde_json::json!(["female", "male"]));
        assert_eq!(parse_sexes(&rule), Ok(vec![Sex::Female, Sex::Male]));
    }
}
