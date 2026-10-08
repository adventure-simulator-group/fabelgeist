use std::{
    fs, io,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use fabelgeist_mhr::{Mhr, MhrAssetDirectoryError, MhrConfig};

// Native fixture namespace only; this owns its temporary files, not model paths.
struct Fixtures {
    native_root: PathBuf,
}

impl Fixtures {
    fn new() -> Self {
        static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
        let native_root = std::env::temp_dir().join(format!(
            "fabelgeist-mhr-directory-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&native_root).unwrap();
        Self { native_root }
    }
}

impl Drop for Fixtures {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.native_root).unwrap();
    }
}

#[test]
fn missing_and_uninspectable_inputs_keep_native_provenance_without_a_cause() {
    let fixtures = Fixtures::new();
    let file_root = fixtures.native_root.join("file");
    fs::write(&file_root, []).unwrap();
    let device = Default::default();
    for native_input in [
        fixtures.native_root.join("missing"),
        fixtures.native_root.clone(),
        file_root,
    ] {
        let failure = Mhr::from_files(&native_input, MhrConfig::default(), &device)
            .err()
            .expect("fixture must reject before model creation");
        let directory_error = failure.downcast_ref::<MhrAssetDirectoryError>().unwrap();
        let MhrAssetDirectoryError::Missing { native_directory } = directory_error;
        assert_eq!(native_directory, &native_input);
        assert!(std::error::Error::source(directory_error).is_none());
        assert!(failure.downcast_ref::<io::Error>().is_none());
        assert_eq!(failure.chain().count(), 1);
        assert_eq!(
            failure.to_string(),
            format!(
                "no MHR assets in {}: expected compact_v6_1.model there or under assets/",
                native_input.display()
            )
        );
    }
}

#[test]
fn nested_then_direct_selection_preserves_next_read_context_and_native_source() {
    let fixtures = Fixtures::new();
    let nested = fixtures.native_root.join("assets");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("compact_v6_1.model"), []).unwrap();
    let device = Default::default();
    let nested_failure = Mhr::from_files(&fixtures.native_root, MhrConfig::default(), &device)
        .err()
        .expect("fixture has no rig");
    assert_eq!(
        nested_failure.to_string(),
        format!("reading {}", nested.join("lod4.fbx").display())
    );
    assert!(
        nested_failure
            .downcast_ref::<MhrAssetDirectoryError>()
            .is_none()
    );
    let native_source = nested_failure.downcast_ref::<io::Error>().unwrap();
    assert_eq!(native_source.kind(), io::ErrorKind::NotFound);
    assert!(std::ptr::eq(
        native_source,
        nested_failure
            .chain()
            .last()
            .unwrap()
            .downcast_ref::<io::Error>()
            .unwrap()
    ));

    fs::write(fixtures.native_root.join("compact_v6_1.model"), []).unwrap();
    let direct_failure = Mhr::from_files(&fixtures.native_root, MhrConfig::default(), &device)
        .err()
        .expect("direct fixture has no rig");
    assert_eq!(
        direct_failure.to_string(),
        format!(
            "reading {}",
            fixtures.native_root.join("lod4.fbx").display()
        )
    );
    assert_eq!(
        direct_failure.downcast_ref::<io::Error>().unwrap().kind(),
        io::ErrorKind::NotFound
    );
    assert!(
        direct_failure
            .downcast_ref::<MhrAssetDirectoryError>()
            .is_none()
    );
}

#[test]
fn invalid_lod_still_rejects_before_native_directory_selection() {
    let fixtures = Fixtures::new();
    let config = MhrConfig {
        lod: 3,
        pose_correctives: false,
    };
    let failure = Mhr::from_files(&fixtures.native_root, config, &Default::default())
        .err()
        .expect("invalid configuration must fail first");
    assert_eq!(failure.to_string(), "LOD 3 is out of range 4..=6");
    assert!(failure.downcast_ref::<MhrAssetDirectoryError>().is_none());
    assert!(failure.downcast_ref::<io::Error>().is_none());
    assert_eq!(failure.chain().count(), 1);
}

#[cfg(unix)]
#[test]
fn symlinks_and_nontext_paths_keep_the_native_file_predicate_policy() {
    use std::{
        ffi::OsString,
        os::unix::{ffi::OsStringExt, fs::symlink},
    };
    let fixtures = Fixtures::new();
    let device = Default::default();
    let target = fixtures.native_root.join("definition-target");
    fs::write(&target, []).unwrap();
    let definition = fixtures.native_root.join("compact_v6_1.model");
    symlink("definition-target", &definition).unwrap();
    let selected = Mhr::from_files(&fixtures.native_root, MhrConfig::default(), &device)
        .err()
        .unwrap();
    assert_eq!(
        selected.downcast_ref::<io::Error>().unwrap().kind(),
        io::ErrorKind::NotFound
    );
    assert!(selected.downcast_ref::<MhrAssetDirectoryError>().is_none());
    fs::remove_file(&definition).unwrap();
    symlink("missing-target", &definition).unwrap();
    let dangling = Mhr::from_files(&fixtures.native_root, MhrConfig::default(), &device)
        .err()
        .unwrap();
    assert!(dangling.downcast_ref::<MhrAssetDirectoryError>().is_some());
    for native_input in [
        fixtures
            .native_root
            .join(OsString::from_vec(b"missing-\xff".to_vec())),
        fixtures.native_root.join("embedded\0nul"),
    ] {
        let failure = Mhr::from_files(&native_input, MhrConfig::default(), &device)
            .err()
            .unwrap();
        let MhrAssetDirectoryError::Missing { native_directory } =
            failure.downcast_ref::<MhrAssetDirectoryError>().unwrap();
        assert_eq!(native_directory, &native_input);
        assert_eq!(
            failure.to_string(),
            format!(
                "no MHR assets in {}: expected compact_v6_1.model there or under assets/",
                native_input.display()
            )
        );
        assert!(failure.downcast_ref::<io::Error>().is_none());
    }
}
