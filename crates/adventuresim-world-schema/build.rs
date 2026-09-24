use sha2::{Digest, Sha256};
use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

#[path = "src/name_catalog_schema.rs"]
mod name_catalog_schema;
#[path = "src/name_catalog_validation.rs"]
mod name_catalog_validation;

const NAME_CATALOG_EXTENSION: &str = "yaml";

fn main() {
    let root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest directory")).join("../..");
    let content = root.join("content/names");
    println!("cargo:rerun-if-changed={}", content.display());

    let mut files = Vec::new();
    collect_yaml_files(&content, &mut files);
    files.sort();
    assert!(!files.is_empty(), "content/names contains no YAML");

    let mut catalog = name_catalog_schema::NameCatalogDocument::default();
    let mut digest = Sha256::new();
    for file in files {
        println!("cargo:rerun-if-changed={}", file.display());
        let relative = file
            .strip_prefix(&root)
            .expect("name catalog belongs to repository")
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("could not read {relative}: {error}"));
        digest.update(relative.as_bytes());
        digest.update([0]);
        digest.update(text.as_bytes());
        let document: name_catalog_schema::NameCatalogDocument = serde_json::from_str(&text)
            .unwrap_or_else(|error| {
                panic!("{relative}: name YAML must use the JSON-compatible subset: {error}")
            });
        extend_catalog(&mut catalog, document);
    }
    apply_repertoire_fragments(&mut catalog);
    name_catalog_validation::validate(&catalog)
        .unwrap_or_else(|error| panic!("name catalog: {error}"));

    let json = serde_json::to_string(&catalog).expect("validated name catalog serializes");
    let digest = format!("{:x}", digest.finalize());
    fs::write(
        Path::new(&env::var("OUT_DIR").expect("build output directory")).join("name_catalog.rs"),
        format!(
            "pub const NAME_CATALOG_JSON: &str = {json:?};\n\
             pub const NAME_CATALOG_DIGEST: &str = {digest:?};\n"
        ),
    )
    .expect("write compiled name catalog");
}

fn extend_catalog(
    catalog: &mut name_catalog_schema::NameCatalogDocument,
    other: name_catalog_schema::NameCatalogDocument,
) {
    catalog.sources.extend(other.sources);
    catalog.given_families.extend(other.given_families);
    catalog.given_forms.extend(other.given_forms);
    catalog.surnames.extend(other.surnames);
    catalog.surname_forms.extend(other.surname_forms);
    catalog.observations.extend(other.observations);
    catalog.derivations.extend(other.derivations);
    catalog.repertoires.extend(other.repertoires);
    catalog
        .repertoire_fragments
        .extend(other.repertoire_fragments);
}

fn apply_repertoire_fragments(catalog: &mut name_catalog_schema::NameCatalogDocument) {
    for fragment in std::mem::take(&mut catalog.repertoire_fragments) {
        let repertoire = catalog
            .repertoires
            .iter_mut()
            .find(|repertoire| repertoire.id == fragment.repertoire_id)
            .unwrap_or_else(|| {
                panic!(
                    "name repertoire fragment references unknown repertoire {}",
                    fragment.repertoire_id
                )
            });
        repertoire.male_families.extend(fragment.male_families);
        repertoire.female_families.extend(fragment.female_families);
        repertoire.everyday_forms.extend(fragment.everyday_forms);
        repertoire.surnames.extend(fragment.surnames);
    }
}

fn collect_yaml_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("content/names must exist") {
        let path = entry.expect("name catalog directory entry").path();
        if path.is_dir() {
            collect_yaml_files(&path, files);
        } else if path
            .extension()
            .is_some_and(|extension| extension == OsStr::new(NAME_CATALOG_EXTENSION))
        {
            files.push(path);
        }
    }
}
