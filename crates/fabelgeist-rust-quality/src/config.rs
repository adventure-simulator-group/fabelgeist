use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub file_limit: usize,
    pub function_limit: usize,
    #[serde(default = "default_census_minimum")]
    pub census_minimum: usize,
    #[serde(default = "default_census_summary_limit")]
    pub census_summary_limit: usize,
    #[serde(default)]
    pub excluded_paths: Vec<String>,
    #[serde(default)]
    pub scopes: Vec<Scope>,
    #[serde(default)]
    pub semantic_families: Vec<SemanticFamily>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub kind: ScopeKind,
    pub path: String,
    pub item: Option<String>,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    Fixture,
    AuthoredCatalog,
    ShaderSource,
    BoundaryAdapter,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticFamily {
    pub name: String,
    pub literals: Vec<String>,
    pub identifier_pattern: String,
    pub owners: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exception {
    pub rule: String,
    pub path: String,
    pub item: String,
    pub fingerprint: String,
    pub occurrences: usize,
    pub reason: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    #[serde(default)]
    pub files: Vec<Debt>,
    #[serde(default)]
    pub functions: Vec<Debt>,
    #[serde(default)]
    pub findings: Vec<FindingBaseline>,
    /// Exact signature debt, grouped by source and item to avoid repeating
    /// source paths for every primitive leaf and unresolved closure.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub interfaces: BTreeMap<String, BTreeMap<String, BTreeMap<String, usize>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Debt {
    pub path: String,
    pub item: Option<String>,
    pub ceiling: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FindingBaseline {
    pub rule: String,
    pub path: String,
    pub item: String,
    pub fingerprint: String,
    pub occurrences: usize,
}

fn default_census_minimum() -> usize {
    3
}

fn default_census_summary_limit() -> usize {
    20
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let config: Self = toml::from_str(&text)
            .map_err(|error| format!("invalid {}: {error}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.file_limit == 0 || self.function_limit == 0 || self.census_minimum < 2 {
            return Err("limits must be positive and census_minimum must be at least two".into());
        }
        for scope in &self.scopes {
            require_reason(&scope.reason, "scope")?;
        }
        let mut exception_keys = BTreeSet::new();
        for exception in &self.exceptions {
            require_reason(&exception.reason, "exception")?;
            if exception.occurrences == 0 {
                return Err("exception occurrence counts must be positive".into());
            }
            let key = (
                exception.rule.as_str(),
                exception.path.as_str(),
                exception.item.as_str(),
                exception.fingerprint.as_str(),
            );
            if !exception_keys.insert(key) {
                return Err(format!(
                    "duplicate exception for {} in {}::{}",
                    exception.rule, exception.path, exception.item
                ));
            }
        }
        for family in &self.semantic_families {
            regex::Regex::new(&family.identifier_pattern).map_err(|error| {
                format!(
                    "semantic family `{}` has an invalid pattern: {error}",
                    family.name
                )
            })?;
        }
        Ok(())
    }
}

impl Baseline {
    pub fn interface_findings(&self) -> Vec<FindingBaseline> {
        self.interfaces
            .iter()
            .flat_map(|(path, items)| {
                items.iter().flat_map(move |(item, signatures)| {
                    signatures.iter().map(move |(key, occurrences)| {
                        // load_optional validates this serialized key boundary.
                        let (rule, fingerprint) = key.split_once(':').unwrap();
                        FindingBaseline {
                            rule: rule.into(),
                            path: path.clone(),
                            item: item.clone(),
                            fingerprint: fingerprint.into(),
                            occurrences: *occurrences,
                        }
                    })
                })
            })
            .collect()
    }

    pub fn load_optional(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let baseline: Self = toml::from_str(&text)
            .map_err(|error| format!("invalid {}: {error}", path.display()))?;
        for (source, items) in &baseline.interfaces {
            for (item, signatures) in items {
                for (key, occurrences) in signatures {
                    if *occurrences == 0
                        || !key.split_once(':').is_some_and(|(rule, _)| {
                            matches!(
                                rule,
                                "raw-interface"
                                    | "unresolved-interface"
                                    | "unexpanded-interface-macro"
                            )
                        })
                    {
                        return Err(format!("invalid interface debt in {source}::{item}: {key}"));
                    }
                }
            }
        }
        Ok(baseline)
    }
}

fn require_reason(reason: &str, kind: &str) -> Result<(), String> {
    if reason.trim().len() < 12 {
        Err(format!(
            "every {kind} needs a specific reason of at least 12 characters"
        ))
    } else {
        Ok(())
    }
}
