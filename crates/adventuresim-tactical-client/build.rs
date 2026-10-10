//! Retire the atmosphere backport when the selected engine contains its fix.
//! This check uses the resolved workspace dependency, without network access.
use semver::Version;
use serde::Deserialize;
use std::{env, error::Error, fs, io, path::PathBuf};

const ENGINE_PACKAGE: &str = "bevy";
const NATIVE_ATMOSPHERE_FIX_VERSION: Version = Version::new(0, 20, 0);

#[derive(Deserialize)]
struct ResolvedDependencies {
    package: Vec<ResolvedPackage>,
}

/// Cargo's lockfile decoding boundary; semantic version admission follows.
#[derive(Deserialize)]
struct ResolvedPackage {
    name: String,
    version: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let lock = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("../..")
        .join("Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock.display());
    let resolved: ResolvedDependencies = toml::from_str(&fs::read_to_string(lock)?)?;
    let mut engines = resolved
        .package
        .iter()
        .filter(|package| package.name == ENGINE_PACKAGE);
    let selected = engines.next().ok_or_else(|| {
        io::Error::other("the atmosphere backport requires one resolved Bevy version")
    })?;
    if engines.next().is_some() {
        return Err(io::Error::other(
            "multiple Bevy versions require reviewing the atmosphere backport guard",
        )
        .into());
    }
    let version = Version::parse(&selected.version)?;
    if version >= NATIVE_ATMOSPHERE_FIX_VERSION {
        return Err(io::Error::other(format!(
            "Bevy {version} contains #24884; remove the atmosphere extraction backport \
             and its build guard before compiling with this engine"
        ))
        .into());
    }
    Ok(())
}
