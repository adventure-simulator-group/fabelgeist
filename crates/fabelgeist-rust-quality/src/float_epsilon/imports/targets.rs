//! Cargo target roots and declared path dependencies define crate identities.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use super::{super::ParsedFile, normalize};

const TARGET_PATH: &str = "path";
const MAIN_SOURCE: &str = "main.rs";

pub(super) struct Root {
    pub file: usize,
    pub package: PathBuf,
    pub library: Option<String>,
    pub externs: BTreeMap<String, Vec<PathBuf>>,
}

struct Package {
    directory: PathBuf,
    metadata: toml::Value,
    library_name: String,
    library_path: PathBuf,
    runtime: Vec<PathDependency>,
    build: Vec<PathDependency>,
}

struct PathDependency {
    binding: String,
    package: PathBuf,
    renamed: bool,
}

#[derive(Clone, Copy)]
enum RootKind {
    Library,
    Build,
    Executable,
}

impl Package {
    fn load(
        base: Option<&Path>,
        directory: PathBuf,
        workspace: &toml::Value,
    ) -> Result<Self, String> {
        let metadata = manifest(base.map(|base| base.join(&directory).join("Cargo.toml")))?;
        let default_name = directory
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let package_name = metadata
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(toml::Value::as_str)
            .unwrap_or(default_name);
        let library = metadata.get("lib");
        let library_name = library
            .and_then(|p| p.get("name"))
            .and_then(toml::Value::as_str)
            .unwrap_or(package_name)
            .replace('-', "_");
        let library_path = normalize(
            &directory.join(
                library
                    .and_then(|p| p.get(TARGET_PATH))
                    .and_then(toml::Value::as_str)
                    .unwrap_or("src/lib.rs"),
            ),
        );
        let runtime = dependencies(
            &metadata,
            &directory,
            workspace,
            &["dependencies", "dev-dependencies"],
        );
        let build = dependencies(&metadata, &directory, workspace, &["build-dependencies"]);
        Ok(Self {
            directory,
            metadata,
            library_name,
            library_path,
            runtime,
            build,
        })
    }

    fn add_root(
        &self,
        file: usize,
        kind: RootKind,
        has_library: bool,
        libraries: &BTreeMap<PathBuf, String>,
        roots: &mut BTreeMap<usize, Root>,
    ) {
        let dependencies = match kind {
            RootKind::Build => &self.build,
            _ => &self.runtime,
        };
        let mut externs = BTreeMap::<String, Vec<_>>::new();
        for dependency in dependencies {
            let name = if dependency.renamed {
                &dependency.binding
            } else {
                libraries
                    .get(&dependency.package)
                    .unwrap_or(&dependency.binding)
            };
            externs
                .entry(name.clone())
                .or_default()
                .push(dependency.package.clone());
        }
        if matches!(kind, RootKind::Executable) && has_library {
            externs
                .entry(self.library_name.clone())
                .or_default()
                .push(self.directory.clone());
        }
        roots.entry(file).or_insert_with(|| Root {
            file,
            package: self.directory.clone(),
            library: matches!(kind, RootKind::Library).then(|| self.library_name.clone()),
            externs,
        });
    }

    fn roots(
        &self,
        paths: &BTreeMap<PathBuf, usize>,
        libraries: &BTreeMap<PathBuf, String>,
        roots: &mut BTreeMap<usize, Root>,
    ) {
        let library_file = paths.get(&self.library_path);
        if let Some(file) = library_file {
            self.add_root(*file, RootKind::Library, true, libraries, roots);
        }
        for (path, kind) in [
            ("src/main.rs", RootKind::Executable),
            ("build.rs", RootKind::Build),
        ] {
            if let Some(file) = paths.get(&self.directory.join(path)) {
                self.add_root(*file, kind, library_file.is_some(), libraries, roots);
            }
        }
        for kind in ["bin", "test", "example", "bench"] {
            for target in self
                .metadata
                .get(kind)
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(file) = target
                    .get(TARGET_PATH)
                    .and_then(toml::Value::as_str)
                    .and_then(|path| paths.get(&normalize(&self.directory.join(path))))
                {
                    self.add_root(
                        *file,
                        RootKind::Executable,
                        library_file.is_some(),
                        libraries,
                        roots,
                    );
                }
            }
        }
        for directory in ["src/bin", "tests", "examples", "benches"] {
            for (path, file) in paths {
                if let Ok(relative) = path.strip_prefix(self.directory.join(directory))
                    && (relative.components().count() == 1
                        || (relative.components().count() == 2
                            && relative.file_name().is_some_and(|name| name == MAIN_SOURCE)))
                {
                    self.add_root(
                        *file,
                        RootKind::Executable,
                        library_file.is_some(),
                        libraries,
                        roots,
                    );
                }
            }
        }
    }
}

pub(super) fn roots(base: Option<&Path>, files: &[ParsedFile]) -> Result<Vec<Root>, String> {
    let paths = files
        .iter()
        .enumerate()
        .map(|(index, file)| (PathBuf::from(&file.path), index))
        .collect::<BTreeMap<_, _>>();
    let packages = paths
        .keys()
        .map(|path| path.components().take(2).collect::<PathBuf>())
        .collect::<BTreeSet<_>>();
    let workspace = manifest(base.map(|base| base.join("Cargo.toml")))?;
    let packages = packages
        .into_iter()
        .map(|directory| Package::load(base, directory, &workspace))
        .collect::<Result<Vec<_>, _>>()?;
    let libraries = packages
        .iter()
        .map(|package| (package.directory.clone(), package.library_name.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut roots = BTreeMap::new();
    for package in packages {
        package.roots(&paths, &libraries, &mut roots);
    }
    Ok(roots.into_values().collect())
}

fn manifest(path: Option<PathBuf>) -> Result<toml::Value, String> {
    match path.filter(|path| path.is_file()) {
        Some(path) => {
            toml::from_str(&fs::read_to_string(&path).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())
        }
        None => Ok(toml::Value::Table(Default::default())),
    }
}

fn dependencies(
    metadata: &toml::Value,
    directory: &Path,
    workspace: &toml::Value,
    sections: &[&str],
) -> Vec<PathDependency> {
    let workspace_dependencies = workspace
        .get("workspace")
        .and_then(|value| value.get("dependencies"));
    let mut result = Vec::new();
    let tables = std::iter::once(metadata).chain(
        metadata
            .get("target")
            .and_then(toml::Value::as_table)
            .into_iter()
            .flat_map(|table| table.values()),
    );
    for table in tables {
        for section in sections {
            for (name, dependency) in table
                .get(*section)
                .and_then(toml::Value::as_table)
                .into_iter()
                .flatten()
            {
                let inherited =
                    dependency.get("workspace").and_then(toml::Value::as_bool) == Some(true);
                let value = if inherited {
                    workspace_dependencies
                        .and_then(|dependencies| dependencies.get(name))
                        .unwrap_or(dependency)
                } else {
                    dependency
                };
                if let Some(path) = value.get(TARGET_PATH).and_then(toml::Value::as_str) {
                    let package = if inherited {
                        PathBuf::from(path)
                    } else {
                        directory.join(path)
                    };
                    result.push(PathDependency {
                        binding: name.replace('-', "_"),
                        package: normalize(&package),
                        renamed: value.get("package").is_some(),
                    });
                }
            }
        }
    }
    result
}
