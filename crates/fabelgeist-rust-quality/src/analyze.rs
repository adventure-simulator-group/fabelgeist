use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use walkdir::WalkDir;

use crate::{
    calendar_flow,
    config::{Baseline, Config, Debt, Exception, FindingBaseline},
    float_epsilon, manifests,
    scan::{Finding, FunctionSize, LiteralOccurrence, path_matches, scan},
};

#[derive(Clone, Debug)]
pub struct CensusEntry {
    value: String,
    occurrences: Vec<(String, usize)>,
}

pub struct Report {
    pub diagnostics: Vec<String>,
    pub census: Vec<CensusEntry>,
    pub snapshot: Baseline,
}

pub fn check_repository(
    root: &Path,
    config: &Config,
    baseline: &Baseline,
) -> Result<Report, String> {
    let mut files = BTreeMap::new();
    let mut functions = Vec::new();
    let mut findings = Vec::new();
    let mut literals = Vec::new();
    let mut calendar_diagnostics = Vec::new();
    let mut epsilon_files = Vec::new();
    let mut seen_items: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for entry in WalkDir::new(root.join("crates")) {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("rs")
        {
            continue;
        }
        let relative = relative_path(root, entry.path())?;
        if excluded(config, &relative) {
            continue;
        }
        let source = fs::read_to_string(entry.path())
            .map_err(|error| format!("cannot read {relative}: {error}"))?;
        let syntax = syn::parse_file(&source)
            .map_err(|error| format!("cannot parse {relative}: {error}"))?;
        if test_file(&relative) {
            epsilon_files.push(float_epsilon::ParsedFile {
                path: relative,
                syntax,
            });
            continue;
        }
        calendar_diagnostics.extend(calendar_flow::check_file(&relative, &syntax));
        let result = scan(&relative, &source, &syntax, config);
        files.insert(relative.clone(), result.production_lines);
        functions.extend(result.functions);
        findings.extend(result.findings);
        literals.extend(result.literals);
        seen_items.insert(relative.clone(), result.seen_items);
        epsilon_files.push(float_epsilon::ParsedFile {
            path: relative,
            syntax,
        });
    }

    findings.extend(float_epsilon::check_files(root, &epsilon_files)?);
    let grouped_findings = group_findings(&findings);
    let mut diagnostics = manifests::check(root)?;
    diagnostics.extend(calendar_diagnostics);
    validate_scopes(config, &files, &seen_items, &mut diagnostics);
    validate_findings(config, baseline, &grouped_findings, &mut diagnostics);
    validate_debt(config, baseline, &files, &functions, &mut diagnostics);
    let census = build_census(literals, config.census_minimum);
    let snapshot = snapshot(config, files, functions, grouped_findings);
    diagnostics.sort();
    diagnostics.dedup();
    Ok(Report {
        diagnostics,
        census,
        snapshot,
    })
}

pub fn print_census(entries: &[CensusEntry], limit: usize) {
    if entries.is_empty() {
        println!("semantic-value census: no repeated candidates");
        return;
    }
    println!(
        "semantic-value census: {} repeated candidate(s), advisory only",
        entries.len()
    );
    for entry in entries.iter().take(limit) {
        let samples = entry
            .occurrences
            .iter()
            .take(3)
            .map(|(path, line)| format!("{path}:{line}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "  {:?}: {} occurrence(s) ({samples})",
            entry.value,
            entry.occurrences.len()
        );
    }
    if entries.len() > limit {
        println!("  ... run the census subcommand for all candidates");
    }
}

pub fn print_baseline(baseline: &Baseline) -> Result<(), String> {
    let encoded = toml::to_string_pretty(baseline)
        .map_err(|error| format!("cannot serialize baseline: {error}"))?;
    print!("{encoded}");
    Ok(())
}

fn validate_scopes(
    config: &Config,
    files: &BTreeMap<String, usize>,
    items: &BTreeMap<String, BTreeSet<String>>,
    diagnostics: &mut Vec<String>,
) {
    for scope in &config.scopes {
        let matching_paths = files
            .keys()
            .filter(|path| path_matches(&scope.path, path))
            .collect::<Vec<_>>();
        if matching_paths.is_empty() {
            diagnostics.push(format!(
                "rust-quality.toml: stale {:?} scope {} matches no production Rust file",
                scope.kind, scope.path
            ));
        } else if let Some(item) = &scope.item
            && !matching_paths
                .iter()
                .any(|path| items.get(*path).is_some_and(|found| found.contains(item)))
        {
            diagnostics.push(format!(
                "rust-quality.toml: stale {:?} scope {}::{item} matches no item",
                scope.kind, scope.path
            ));
        }
    }
}

fn validate_findings(
    config: &Config,
    baseline: &Baseline,
    actual: &BTreeMap<FindingKey, FindingGroup>,
    diagnostics: &mut Vec<String>,
) {
    let baselined = baseline
        .findings
        .iter()
        .map(|finding| (FindingKey::from_baseline(finding), finding.occurrences))
        .collect::<BTreeMap<_, _>>();
    let exceptions = config
        .exceptions
        .iter()
        .map(|exception| (FindingKey::from_exception(exception), exception))
        .collect::<BTreeMap<_, _>>();

    for (key, group) in actual {
        if key.rule == float_epsilon::RULE && baselined.contains_key(key) {
            diagnostics.push(format!(
                "{}:{}: machine-epsilon requires an audited exception, not a baseline",
                key.path, group.first_line
            ));
        } else if let Some(expected) = baselined.get(key) {
            if *expected != group.count {
                diagnostics.push(format!(
                    "{}:{}: baseline for {} {} expected {} occurrence(s), found {}; tighten it",
                    key.path, group.first_line, key.rule, key.fingerprint, expected, group.count
                ));
            }
        } else if let Some(exception) = exceptions.get(key) {
            if exception.occurrences != group.count {
                diagnostics.push(format!(
                    "{}:{}: exception for {} {} expected {} occurrence(s), found {}",
                    key.path,
                    group.first_line,
                    key.rule,
                    key.fingerprint,
                    exception.occurrences,
                    group.count
                ));
            }
        } else {
            diagnostics.push(format!(
                "{}:{}: {} in {} ({})",
                key.path, group.first_line, key.rule, key.item, key.fingerprint
            ));
        }
    }
    for (key, expected) in &baselined {
        if !actual.contains_key(key) {
            diagnostics.push(format!(
                "rust-quality-baseline.toml: stale finding {} {} in {}::{} ({} occurrence(s))",
                key.rule, key.fingerprint, key.path, key.item, expected
            ));
        }
    }
    for (key, exception) in &exceptions {
        if !actual.contains_key(key) {
            diagnostics.push(format!(
                "rust-quality.toml: stale exception {} {} in {}::{} ({})",
                key.rule, key.fingerprint, key.path, key.item, exception.reason
            ));
        }
    }
}

fn validate_debt(
    config: &Config,
    baseline: &Baseline,
    files: &BTreeMap<String, usize>,
    functions: &[FunctionSize],
    diagnostics: &mut Vec<String>,
) {
    let file_baseline = baseline
        .files
        .iter()
        .map(|debt| (debt.path.as_str(), debt.ceiling))
        .collect::<BTreeMap<_, _>>();
    for (path, lines) in files {
        if *lines > config.file_limit {
            match file_baseline.get(path.as_str()) {
                Some(ceiling) if lines <= ceiling => {}
                Some(ceiling) => diagnostics.push(format!(
                    "{path}: production file grew to {lines} lines above its {ceiling}-line ceiling"
                )),
                None => diagnostics.push(format!(
                    "{path}: new production file debt is {lines} lines (limit {})",
                    config.file_limit
                )),
            }
        }
    }
    for debt in &baseline.files {
        match files.get(&debt.path) {
            None => diagnostics.push(format!(
                "rust-quality-baseline.toml: stale file debt for missing {}",
                debt.path
            )),
            Some(lines) if *lines <= config.file_limit => diagnostics.push(format!(
                "rust-quality-baseline.toml: {} is now below the file limit; remove its debt entry",
                debt.path
            )),
            _ => {}
        }
    }

    let actual_functions = functions
        .iter()
        .map(|function| {
            (
                (function.path.as_str(), function.item.as_str()),
                function.lines,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let function_baseline = baseline
        .functions
        .iter()
        .filter_map(|debt| {
            debt.item
                .as_deref()
                .map(|item| ((debt.path.as_str(), item), debt.ceiling))
        })
        .collect::<BTreeMap<_, _>>();
    for ((path, item), lines) in &actual_functions {
        if *lines > config.function_limit {
            match function_baseline.get(&(*path, *item)) {
                Some(ceiling) if lines <= ceiling => {}
                Some(ceiling) => diagnostics.push(format!(
                    "{path}::{item}: function grew to {lines} lines above its {ceiling}-line ceiling"
                )),
                None => diagnostics.push(format!(
                    "{path}::{item}: new function debt is {lines} lines (limit {})",
                    config.function_limit
                )),
            }
        }
    }
    for debt in &baseline.functions {
        let Some(item) = debt.item.as_deref() else {
            diagnostics.push(format!(
                "rust-quality-baseline.toml: function debt {} has no item",
                debt.path
            ));
            continue;
        };
        match actual_functions.get(&(debt.path.as_str(), item)) {
            None => diagnostics.push(format!(
                "rust-quality-baseline.toml: stale function debt for {}::{item}",
                debt.path
            )),
            Some(lines) if *lines <= config.function_limit => diagnostics.push(format!(
                "rust-quality-baseline.toml: {}::{item} is now below the function limit; remove its debt entry",
                debt.path
            )),
            _ => {}
        }
    }
}

fn snapshot(
    config: &Config,
    files: BTreeMap<String, usize>,
    mut functions: Vec<FunctionSize>,
    findings: BTreeMap<FindingKey, FindingGroup>,
) -> Baseline {
    let mut file_debt = files
        .into_iter()
        .filter(|(_, lines)| *lines > config.file_limit)
        .map(|(path, ceiling)| Debt {
            path,
            item: None,
            ceiling,
        })
        .collect::<Vec<_>>();
    functions.sort();
    let function_debt = functions
        .into_iter()
        .filter(|function| function.lines > config.function_limit)
        .map(|function| Debt {
            path: function.path,
            item: Some(function.item),
            ceiling: function.lines,
        })
        .collect();
    let finding_debt = findings
        .into_iter()
        .filter(|(key, _)| key.rule != float_epsilon::RULE)
        .map(|(key, group)| FindingBaseline {
            rule: key.rule,
            path: key.path,
            item: key.item,
            fingerprint: key.fingerprint,
            occurrences: group.count,
        })
        .collect();
    file_debt.sort_by(|left, right| left.path.cmp(&right.path));
    Baseline {
        files: file_debt,
        functions: function_debt,
        findings: finding_debt,
    }
}

fn build_census(literals: Vec<LiteralOccurrence>, minimum: usize) -> Vec<CensusEntry> {
    let mut grouped: BTreeMap<String, Vec<(String, usize)>> = BTreeMap::new();
    for literal in literals {
        grouped
            .entry(literal.value)
            .or_default()
            .push((literal.path, literal.line));
    }
    let mut entries = grouped
        .into_iter()
        .filter(|(_, occurrences)| occurrences.len() >= minimum)
        .map(|(value, occurrences)| CensusEntry { value, occurrences })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .occurrences
            .len()
            .cmp(&left.occurrences.len())
            .then_with(|| left.value.cmp(&right.value))
    });
    entries
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct FindingKey {
    rule: String,
    path: String,
    item: String,
    fingerprint: String,
}

impl FindingKey {
    fn from_baseline(value: &FindingBaseline) -> Self {
        Self {
            rule: value.rule.clone(),
            path: value.path.clone(),
            item: value.item.clone(),
            fingerprint: value.fingerprint.clone(),
        }
    }

    fn from_exception(value: &Exception) -> Self {
        Self {
            rule: value.rule.clone(),
            path: value.path.clone(),
            item: value.item.clone(),
            fingerprint: value.fingerprint.clone(),
        }
    }
}

struct FindingGroup {
    count: usize,
    first_line: usize,
}

fn group_findings(findings: &[Finding]) -> BTreeMap<FindingKey, FindingGroup> {
    let mut grouped = BTreeMap::new();
    for finding in findings {
        let key = FindingKey {
            rule: finding.rule.clone(),
            path: finding.path.clone(),
            item: finding.item.clone(),
            fingerprint: finding.fingerprint.clone(),
        };
        let group = grouped.entry(key).or_insert(FindingGroup {
            count: 0,
            first_line: finding.line,
        });
        group.count += 1;
        group.first_line = group.first_line.min(finding.line);
    }
    grouped
}

fn excluded(config: &Config, path: &str) -> bool {
    path.contains("/target/")
        || config
            .excluded_paths
            .iter()
            .any(|pattern| path_matches(pattern, path))
}

fn test_file(path: &str) -> bool {
    path.contains("/tests/") || path.ends_with("_test.rs") || path.ends_with("/tests.rs")
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .map_err(|_| format!("{} is outside {}", path.display(), root.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_files_are_not_production() {
        assert!(test_file("crates/example/tests/behavior.rs"));
        assert!(test_file("crates/example/src/tests.rs"));
        assert!(!test_file("crates/example/src/lib.rs"));
    }

    #[test]
    fn standalone_build_outputs_are_excluded() {
        let config: Config = toml::from_str("file_limit = 500\nfunction_limit = 100\n").unwrap();
        assert!(excluded(
            &config,
            "crates/example/target/debug/build/generated.rs"
        ));
    }

    #[test]
    fn precision_scan_includes_test_files_without_changing_production_metrics() {
        let root =
            std::env::temp_dir().join(format!("fabelgeist-epsilon-scan-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        fs::create_dir_all(root.join("crates/example/tests")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = []\n[workspace.lints.clippy]\nall = 'warn'\n",
        )
        .unwrap();
        fs::write(
            root.join("crates/example/tests/numerics.rs"),
            "#[test] fn boundary() { assert!(value <= f32::EPSILON); }",
        )
        .unwrap();
        let config: Config = toml::from_str("file_limit = 1\nfunction_limit = 1\n").unwrap();
        let report = check_repository(&root, &config, &Baseline::default()).unwrap();
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(report.diagnostics.len(), 1);
        assert!(report.diagnostics[0].contains(float_epsilon::RULE));
        assert!(report.snapshot.files.is_empty());
        assert!(report.snapshot.functions.is_empty());
    }

    #[test]
    fn epsilon_provenance_crosses_reexports_test_files_and_macro_definitions() {
        let root = std::env::temp_dir().join(format!(
            "fabelgeist-epsilon-reexports-{}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("crates/example/src")).unwrap();
        fs::create_dir_all(root.join("crates/example/tests")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = []\n[workspace.lints.clippy]\nall = 'warn'\n",
        )
        .unwrap();
        // Create consumers first and reverse the alias chain's lexical order.
        // Neither filesystem order nor the containing syntax tree determines
        // the imported module's provenance.
        for (path, source) in [
            (
                "src/lib.rs",
                r#"
                mod floats; pub mod bridge; mod macros;
                use bridge::number::*;
                pub fn near_zero(x: f32) -> bool { x <= EPSILON }
            "#,
            ),
            (
                "tests/numeric.rs",
                r#"
                use example::bridge::number::*;
                #[test] fn boundary() { assert!(value <= EPSILON); }
            "#,
            ),
            (
                "src/macros.rs",
                r#"
                macro_rules! reexported {
                    ($x:expr) => {{ use crate::bridge::number::*; $x <= EPSILON }};
                }
                macro_rules! another_alias {
                    ($x:expr) => {{ use core::f64 as double; use double::*; $x <= EPSILON }};
                }
                macro_rules! grouped {
                    ($x:expr) => {{ use std::f32::{EPSILON as error}; $x <= error }};
                }
            "#,
            ),
            ("src/bridge.rs", "pub use crate::floats::single as number;"),
            ("src/floats.rs", "pub use std::f32 as single;"),
        ] {
            fs::write(root.join("crates/example").join(path), source).unwrap();
        }
        let config: Config = toml::from_str("file_limit = 500\nfunction_limit = 100\n").unwrap();
        let report = check_repository(&root, &config, &Baseline::default()).unwrap();
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(report.diagnostics.len(), 5, "{:?}", report.diagnostics);
        assert!(
            report
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.contains(float_epsilon::RULE))
        );
        for path in ["src/lib.rs", "src/macros.rs", "tests/numeric.rs"] {
            assert!(
                report
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.contains(path)),
                "{path}"
            );
        }
        assert!(
            report.snapshot.findings.is_empty(),
            "epsilon cannot be baselined"
        );
    }

    #[test]
    fn included_production_and_test_fragments_keep_exact_exception_locations() {
        let root = std::env::temp_dir().join(format!(
            "fabelgeist-epsilon-includes-{}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("crates/example/src")).unwrap();
        fs::create_dir_all(root.join("crates/example/tests/fragments")).unwrap();
        for (path, source) in [
            (
                "Cargo.toml",
                "[workspace]\nmembers = []\n[workspace.lints.clippy]\nall = 'warn'\n",
            ),
            (
                "crates/example/src/lib.rs",
                r#"
                use std::f32 as values;
                include!("checks.rs");
                #[cfg(test)] mod tests {
                    use core::f64 as values;
                    include!("../tests/fragments/checks.rs");
                }
            "#,
            ),
            (
                "crates/example/src/checks.rs",
                "use values::*; pub fn near_zero(x: f32) -> bool { x <= EPSILON }",
            ),
            (
                "crates/example/tests/fragments/checks.rs",
                "use values::*; #[test] fn boundary() { assert!(0.0 <= EPSILON); }",
            ),
        ] {
            fs::write(root.join(path), source).unwrap();
        }
        let mut config: Config =
            toml::from_str("file_limit = 500\nfunction_limit = 100\n").unwrap();
        let report = check_repository(&root, &config, &Baseline::default()).unwrap();
        assert_eq!(report.diagnostics.len(), 2, "{:?}", report.diagnostics);
        for path in [
            "crates/example/src/checks.rs",
            "crates/example/tests/fragments/checks.rs",
        ] {
            assert!(report.diagnostics.iter().any(|message| {
                message.starts_with(&format!("{path}:1: machine-epsilon in <module>"))
            }));
            config.exceptions.push(Exception {
                rule: float_epsilon::RULE.into(),
                path: path.into(),
                item: "<module>".into(),
                fingerprint: "EPSILON".into(),
                occurrences: 1,
                reason: "Fixture checks source identity for inherited floating-point imports."
                    .into(),
            });
        }
        let audited = check_repository(&root, &config, &Baseline::default()).unwrap();
        fs::write(root.join("crates/example/src/checks.rs"), "").unwrap();
        let stale = check_repository(&root, &config, &Baseline::default()).unwrap();
        fs::remove_dir_all(&root).unwrap();
        assert!(audited.diagnostics.is_empty(), "{:?}", audited.diagnostics);
        assert_eq!(stale.diagnostics.len(), 1, "{:?}", stale.diagnostics);
        assert!(stale.diagnostics[0].contains("stale exception machine-epsilon"));
        assert!(stale.diagnostics[0].contains("crates/example/src/checks.rs::<module>"));
    }

    #[test]
    fn independent_crates_and_modules_with_matching_names_need_no_exception() {
        let root = std::env::temp_dir().join(format!(
            "fabelgeist-epsilon-isolation-{}",
            std::process::id()
        ));
        for directory in ["crates/a/src", "crates/a/tests", "crates/b/src"] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = []\n[workspace.lints.clippy]\nall = 'warn'\n",
        )
        .unwrap();
        for (path, source) in [
            (
                "crates/a/src/lib.rs",
                "use std::f32 as values; mod producer; mod consumer; pub fn minimum() -> f32 { values::MIN }",
            ),
            (
                "crates/a/src/producer.rs",
                "use core::f64 as local; pub fn minimum() -> f64 { local::MIN }",
            ),
            (
                "crates/a/src/consumer.rs",
                "mod local { pub fn count() -> usize { 0 } } use local::*; pub fn count_items() -> usize { count() }",
            ),
            (
                "crates/a/src/main.rs",
                "mod values {} use values::*; fn main() {}",
            ),
            (
                "crates/a/tests/independent.rs",
                "mod values {} use values::*;",
            ),
            (
                "crates/b/src/lib.rs",
                "mod values { pub fn count() -> usize { 0 } } use values::*; pub fn count_items() -> usize { count() }",
            ),
        ] {
            fs::write(root.join(path), source).unwrap();
        }
        let config: Config = toml::from_str("file_limit = 500\nfunction_limit = 100\n").unwrap();
        let report = check_repository(&root, &config, &Baseline::default()).unwrap();
        fs::remove_dir_all(&root).unwrap();
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        assert!(report.snapshot.findings.is_empty());
    }

    #[test]
    fn cargo_target_paths_and_dependency_renames_preserve_import_identity() {
        let root =
            std::env::temp_dir().join(format!("fabelgeist-epsilon-cargo-{}", std::process::id()));
        for directory in [
            "crates/a/src",
            "crates/b/src",
            "crates/c/src",
            "crates/d/src",
            "crates/ordinary/src",
        ] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        for (path, source) in [
            (
                "Cargo.toml",
                r#"
                [workspace]
                members = ["crates/a", "crates/b", "crates/c", "crates/d", "crates/ordinary"]
                resolver = '3'
                [workspace.lints.clippy]
                all = 'warn'
                [workspace.dependencies]
                shared_numbers = { package = 'numeric-origin', path = 'crates/a' }
            "#,
            ),
            (
                "crates/a/Cargo.toml",
                r#"
                [package]
                name = 'numeric-origin'
                version = '0.1.0'
                edition = '2024'
                [lib]
                name = 'precision_values'
                path = 'public.rs'
                [lints]
                workspace = true
            "#,
            ),
            (
                "crates/b/Cargo.toml",
                r#"
                [package]
                name = 'consumer'
                version = '0.1.0'
                edition = '2024'
                [dependencies]
                numbers = { package = 'numeric-origin', path = '../a' }
                values = { package = 'ordinary-origin', path = '../ordinary' }
                [lints]
                workspace = true
            "#,
            ),
            (
                "crates/c/Cargo.toml",
                r#"
                [package]
                name = 'workspace-consumer'
                version = '0.1.0'
                edition = '2024'
                [dependencies]
                shared_numbers.workspace = true
                [lints]
                workspace = true
            "#,
            ),
            (
                "crates/d/Cargo.toml",
                r#"
                [package]
                name = 'default-consumer'
                version = '0.1.0'
                edition = '2024'
                [dependencies]
                numeric-origin = { path = '../a' }
                [lints]
                workspace = true
            "#,
            ),
            (
                "crates/ordinary/Cargo.toml",
                r#"
                [package]
                name = 'ordinary-origin'
                version = '0.1.0'
                edition = '2024'
                [lib]
                name = 'values'
                [lints]
                workspace = true
            "#,
            ),
            (
                "crates/a/public.rs",
                "#[path = \"src/floats.rs\"] pub mod floats;",
            ),
            ("crates/a/src/floats.rs", "pub use std::f64 as values;"),
            (
                "crates/ordinary/src/lib.rs",
                "use std::f32 as values; pub fn count() -> usize { 0 } pub fn minimum() -> f32 { values::MIN } pub mod exports { pub use super::*; }",
            ),
            (
                "crates/b/src/lib.rs",
                r#"
                use values::exports::*;
                use values::*;
                pub fn count_items() -> usize { count() }
                mod nested {
                    use numbers::floats::values::*;
                }
            "#,
            ),
            (
                "crates/c/src/lib.rs",
                "mod nested { use shared_numbers::floats::values::*; }",
            ),
            (
                "crates/d/src/lib.rs",
                "mod nested { use precision_values::floats::values::*; }",
            ),
        ] {
            fs::write(root.join(path), source).unwrap();
        }
        let mut config: Config =
            toml::from_str("file_limit = 500\nfunction_limit = 100\n").unwrap();
        let report = check_repository(&root, &config, &Baseline::default()).unwrap();
        for path in [
            "crates/b/src/lib.rs",
            "crates/c/src/lib.rs",
            "crates/d/src/lib.rs",
        ] {
            config.exceptions.push(Exception {
                rule: float_epsilon::RULE.into(),
                path: path.into(),
                item: "nested".into(),
                fingerprint: "EPSILON".into(),
                occurrences: 1,
                reason:
                    "Fixture verifies an explicit import of floating-point precision containers."
                        .into(),
            });
        }
        let audited = check_repository(&root, &config, &Baseline::default()).unwrap();
        let ordinary = root.join("crates/ordinary/src/lib.rs");
        let source = fs::read_to_string(&ordinary).unwrap();
        fs::write(
            &ordinary,
            source.replacen("use std::f32", "pub(crate) use std::f32", 1),
        )
        .unwrap();
        let restricted = check_repository(&root, &config, &Baseline::default()).unwrap();
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(report.diagnostics.len(), 3, "{:?}", report.diagnostics);
        assert!(
            report
                .diagnostics
                .iter()
                .all(|message| message.contains("machine-epsilon in nested (EPSILON)"))
        );
        assert!(audited.diagnostics.is_empty(), "{:?}", audited.diagnostics);
        assert!(
            restricted.diagnostics.is_empty(),
            "{:?}",
            restricted.diagnostics
        );
    }

    #[test]
    fn stale_exceptions_fail() {
        let config = Config {
            file_limit: 500,
            function_limit: 100,
            census_minimum: 3,
            census_summary_limit: 20,
            excluded_paths: Vec::new(),
            scopes: Vec::new(),
            semantic_families: Vec::new(),
            exceptions: vec![Exception {
                rule: "raw-string-branch".into(),
                path: "crates/example/src/lib.rs".into(),
                item: "decide".into(),
                fingerprint: "string:pending".into(),
                occurrences: 1,
                reason: "External protocol adapter value.".into(),
            }],
        };
        let mut diagnostics = Vec::new();
        validate_findings(
            &config,
            &Baseline::default(),
            &BTreeMap::new(),
            &mut diagnostics,
        );
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("stale exception"));
    }

    #[test]
    fn machine_precision_requires_exact_audited_exceptions() {
        let mut config: Config =
            toml::from_str("file_limit = 500\nfunction_limit = 100\n").unwrap();
        let syntax = syn::parse_file("const ROUNDING: f32 = f32::EPSILON;").unwrap();
        let findings = float_epsilon::check_file("crates/example/tests/rounding.rs", &syntax);
        let actual = group_findings(&findings);
        let finding = &findings[0];
        let mut diagnostics = Vec::new();
        validate_findings(&config, &Baseline::default(), &actual, &mut diagnostics);
        assert_eq!(diagnostics.len(), 1);
        config.exceptions.push(Exception {
            rule: finding.rule.clone(),
            path: finding.path.clone(),
            item: finding.item.clone(),
            fingerprint: finding.fingerprint.clone(),
            occurrences: 1,
            reason: "Relative roundoff in a normalized calculation.".into(),
        });
        diagnostics.clear();
        validate_findings(&config, &Baseline::default(), &actual, &mut diagnostics);
        assert!(diagnostics.is_empty());
        config.exceptions[0].occurrences = 2;
        validate_findings(&config, &Baseline::default(), &actual, &mut diagnostics);
        assert!(diagnostics[0].contains("expected 2 occurrence(s), found 1"));
        diagnostics.clear();
        validate_findings(
            &config,
            &Baseline::default(),
            &BTreeMap::new(),
            &mut diagnostics,
        );
        assert!(diagnostics[0].contains("stale exception"));
        let baseline = Baseline {
            findings: vec![FindingBaseline {
                rule: finding.rule.clone(),
                path: finding.path.clone(),
                item: finding.item.clone(),
                fingerprint: finding.fingerprint.clone(),
                occurrences: 1,
            }],
            ..Default::default()
        };
        diagnostics.clear();
        validate_findings(&config, &baseline, &actual, &mut diagnostics);
        assert!(diagnostics[0].contains("not a baseline"));
        assert!(
            snapshot(&config, BTreeMap::new(), Vec::new(), actual)
                .findings
                .is_empty()
        );
    }

    #[test]
    fn debt_can_shrink_but_cannot_grow_past_its_ceiling() {
        let config = Config {
            file_limit: 500,
            function_limit: 100,
            census_minimum: 3,
            census_summary_limit: 20,
            excluded_paths: Vec::new(),
            scopes: Vec::new(),
            semantic_families: Vec::new(),
            exceptions: Vec::new(),
        };
        let baseline = Baseline {
            files: vec![Debt {
                path: "crates/example/src/lib.rs".into(),
                item: None,
                ceiling: 550,
            }],
            functions: Vec::new(),
            findings: Vec::new(),
        };
        let mut files = BTreeMap::from([("crates/example/src/lib.rs".into(), 525)]);
        let mut diagnostics = Vec::new();
        validate_debt(&config, &baseline, &files, &[], &mut diagnostics);
        assert!(diagnostics.is_empty());

        files.insert("crates/example/src/lib.rs".into(), 551);
        validate_debt(&config, &baseline, &files, &[], &mut diagnostics);
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("grew to 551"))
        );
    }
}
