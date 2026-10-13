//! Retire the atmosphere backport when the selected engine contains its fix.
//! This check uses the resolved workspace dependency, without network access.
use semver::Version;
use serde::Deserialize;
use std::{env, io};
#[cfg(not(test))]
use std::{fs, path::PathBuf, process::ExitCode};
use thiserror::Error;

const NATIVE_ATMOSPHERE_FIX_VERSION: Version = Version::new(0, 20, 0);

type Result<T> = std::result::Result<T, BevyGuardError>;

#[derive(Deserialize)]
struct ResolvedDependencies {
    package: Vec<LockfilePackage>,
}

#[derive(Deserialize)]
struct LockfilePackage {
    name: CargoPackageRole,
    version: EncodedPackageVersion,
}

/// Unparsed Cargo version text; irrelevant packages need no version parsing.
#[derive(Deserialize)]
#[serde(transparent)]
struct EncodedPackageVersion(String);

/// The only package role relevant to the atmosphere backport guard.
#[derive(Deserialize)]
#[serde(from = "String")]
enum CargoPackageRole {
    Bevy,
    Other,
}

#[derive(Debug, Error)]
enum BevyGuardError {
    #[error("the atmosphere backport requires one resolved Bevy version")]
    MissingEngine,
    #[error("multiple Bevy versions require reviewing the atmosphere backport guard")]
    MultipleEngines,
    #[error("invalid selected Bevy version: {0}")]
    InvalidEngineVersion(#[from] semver::Error),
    #[error(
        "Bevy {version} contains #24884; remove the atmosphere extraction backport \
         and its build guard before compiling with this engine"
    )]
    BackportObsolete { version: Version },
    #[error("cannot locate the Cargo manifest directory: {0}")]
    ManifestDirectory(#[from] env::VarError),
    #[error("cannot read workspace Cargo.lock: {0}")]
    ReadLockfile(#[from] io::Error),
    #[error("cannot decode workspace Cargo.lock: {0}")]
    DecodeLockfile(#[from] toml::de::Error),
}

impl ResolvedDependencies {
    fn selected_engine_version(self) -> Result<Version> {
        let mut engines = self
            .package
            .into_iter()
            .filter_map(|package| match package.name {
                CargoPackageRole::Bevy => Some(package.version),
                CargoPackageRole::Other => None,
            });
        let version = engines.next().ok_or(BevyGuardError::MissingEngine)?;
        if engines.next().is_some() {
            return Err(BevyGuardError::MultipleEngines);
        }
        Ok(version.parse()?)
    }

    fn require_backport(self) -> Result<()> {
        let version = self.selected_engine_version()?;
        if version >= NATIVE_ATMOSPHERE_FIX_VERSION {
            return Err(BevyGuardError::BackportObsolete { version });
        }
        Ok(())
    }
}

impl EncodedPackageVersion {
    fn parse(self) -> std::result::Result<Version, semver::Error> {
        Version::parse(&self.0)
    }
}

impl From<String> for CargoPackageRole {
    fn from(name: String) -> Self {
        // Serde's String input preserves Cargo's string-only wire diagnostics.
        match name.as_str() {
            "bevy" => Self::Bevy,
            _ => Self::Other,
        }
    }
}

#[cfg(not(test))]
fn main() -> ExitCode {
    match run_guard() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(test))]
fn run_guard() -> Result<()> {
    let lock = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("../..")
        .join("Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock.display());
    let resolved: ResolvedDependencies = toml::from_str(&fs::read_to_string(lock)?)?;
    resolved.require_backport()
}

#[cfg(test)]
mod tests {
    use super::{BevyGuardError, ResolvedDependencies, Result};
    use semver::Version;
    use std::error::Error;

    fn validate_fixture(lockfile: &str) -> Result<()> {
        toml::from_str::<ResolvedDependencies>(lockfile)?.require_backport()
    }

    #[test]
    fn selected_versions_keep_the_backport() {
        for version in ["0.19.1", "0.19.1+build", "0.20.0-rc.1"] {
            let lockfile = format!("[[package]]\nname = \"bevy\"\nversion = \"{version}\"\n");
            assert!(validate_fixture(&lockfile).is_ok(), "{version}");
        }
    }

    #[test]
    fn stable_fix_versions_require_removal_with_the_selected_version() {
        for version in ["0.20.0", "0.20.0+build", "0.21.0", "0.21.0-rc.1"] {
            let lockfile = format!("[[package]]\nname = \"bevy\"\nversion = \"{version}\"\n");
            let error = validate_fixture(&lockfile).unwrap_err();
            assert!(matches!(error, BevyGuardError::BackportObsolete { .. }));
            assert_eq!(
                error.to_string(),
                format!(
                    "Bevy {version} contains #24884; remove the atmosphere extraction backport \
                     and its build guard before compiling with this engine"
                )
            );
        }
    }

    #[test]
    fn missing_engine_has_a_structured_actionable_error() {
        let error =
            validate_fixture("[[package]]\nname = \"serde\"\nversion = \"invalid\"\n").unwrap_err();
        assert!(matches!(error, BevyGuardError::MissingEngine));
        assert_eq!(
            error.to_string(),
            "the atmosphere backport requires one resolved Bevy version"
        );
    }

    #[test]
    fn duplicates_take_precedence_over_invalid_versions() {
        for (first, second) in [
            ("invalid", "0.19.1"),
            ("0.19.1", "invalid"),
            ("invalid", "invalid"),
            ("0.19.1", "0.19.1"),
        ] {
            let lockfile = format!(
                "[[package]]\nname = \"bevy\"\nversion = \"{first}\"\n\n\
                 [[package]]\nname = \"bevy\"\nversion = \"{second}\"\n"
            );
            let error = validate_fixture(&lockfile).unwrap_err();
            assert!(matches!(error, BevyGuardError::MultipleEngines));
            assert_eq!(
                error.to_string(),
                "multiple Bevy versions require reviewing the atmosphere backport guard"
            );
        }
    }

    #[test]
    fn invalid_selected_version_retains_its_parse_source_and_context() {
        let error =
            validate_fixture("[[package]]\nname = \"bevy\"\nversion = \"invalid\"\n").unwrap_err();
        assert!(matches!(error, BevyGuardError::InvalidEngineVersion(_)));
        assert!(error.source().is_some());
        assert!(
            error
                .to_string()
                .starts_with("invalid selected Bevy version: ")
        );
    }

    #[test]
    fn only_exact_bevy_names_select_the_engine() {
        for other in ["Bevy", "BEVY", "bevy_ecs", ""] {
            let lockfile = format!(
                "[[package]]\nname = \"{other}\"\nversion = \"invalid\"\n\n\
                 [[package]]\nname = \"bevy\"\nversion = \"0.19.1\"\n"
            );
            assert!(validate_fixture(&lockfile).is_ok(), "{other}");
        }
    }

    #[test]
    fn malformed_wire_fields_remain_string_only() {
        for malformed in ["19", "true", "[]", "{}", "{ bevy = {} }"] {
            for field in ["name", "version"] {
                let lockfile = if field == "name" {
                    format!("[[package]]\nname = {malformed}\nversion = \"0.19.1\"\n")
                } else {
                    format!("[[package]]\nname = \"bevy\"\nversion = {malformed}\n")
                };
                let error = validate_fixture(&lockfile).unwrap_err();
                assert!(matches!(error, BevyGuardError::DecodeLockfile(_)));
                assert!(error.to_string().contains("expected a string"), "{error}");
            }
        }
    }

    #[test]
    fn selected_version_keeps_prerelease_and_build_components() {
        let resolved: ResolvedDependencies =
            toml::from_str("[[package]]\nname = \"bevy\"\nversion = \"0.20.0-rc.1+build\"\n")
                .unwrap();
        assert_eq!(
            resolved.selected_engine_version().unwrap(),
            Version::parse("0.20.0-rc.1+build").unwrap()
        );
    }
}
