//! Retire the atmosphere backport when the selected engine contains its fix.
//! This check uses the resolved workspace dependency, without network access.
use semver::Version;
use serde::{Deserialize, Deserializer};
use std::{env, error::Error, fs, io, path::PathBuf};

const NATIVE_ATMOSPHERE_FIX_VERSION: Version = Version::new(0, 20, 0);

#[derive(Deserialize)]
struct ResolvedDependencies {
    package: Vec<ResolvedPackage>,
}

/// Cargo's wire text is confined to lockfile decoding.
#[derive(Deserialize)]
struct LockfilePackage {
    name: String,
    version: String,
}

/// Only the engine's identity and semantic version matter to this guard.
enum ResolvedPackage {
    Bevy {
        version: Result<Version, semver::Error>,
    },
    Other,
}

impl ResolvedDependencies {
    fn selected_engine_version(self) -> Result<Version, Box<dyn Error>> {
        let mut engines = self
            .package
            .into_iter()
            .filter_map(|package| match package {
                ResolvedPackage::Bevy { version } => Some(version),
                ResolvedPackage::Other => None,
            });
        let version = engines.next().ok_or_else(|| {
            io::Error::other("the atmosphere backport requires one resolved Bevy version")
        })?;
        if engines.next().is_some() {
            return Err(io::Error::other(
                "multiple Bevy versions require reviewing the atmosphere backport guard",
            )
            .into());
        }
        Ok(version?)
    }
}

impl<'de> Deserialize<'de> for ResolvedPackage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let package = LockfilePackage::deserialize(deserializer)?;
        Ok(match package.name.as_str() {
            // Preserve duplicate-engine precedence over invalid-version errors.
            "bevy" => Self::Bevy {
                version: Version::parse(&package.version),
            },
            _ => Self::Other,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lock = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("../..")
        .join("Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock.display());
    let resolved: ResolvedDependencies = toml::from_str(&fs::read_to_string(lock)?)?;
    let version = resolved.selected_engine_version()?;
    if version >= NATIVE_ATMOSPHERE_FIX_VERSION {
        return Err(io::Error::other(format!(
            "Bevy {version} contains #24884; remove the atmosphere extraction backport \
             and its build guard before compiling with this engine"
        ))
        .into());
    }
    Ok(())
}
