use super::*;

#[test]
fn native_selection_prefers_the_direct_definition_and_reads_exact_role_payloads() {
    let root = tempfile::tempdir().unwrap();
    let nested = root.path().join("assets");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(nested.join("compact_v6_1.model"), []).unwrap();
    let directory = MhrAssetDirectory::from(root.path().to_path_buf());
    assert_eq!(
        directory.resolve().unwrap(),
        MhrAssetDirectory::from(nested)
    );
    std::fs::write(root.path().join("compact_v6_1.model"), []).unwrap();
    assert_eq!(directory.resolve().unwrap(), directory);
    for lod in CharacterLod::ALL {
        for asset in [MhrAsset::Rig(lod), MhrAsset::CorrectiveBasis(lod)] {
            let payload = [0, 0xff, 0x0a, 4];
            std::fs::write(root.path().join(asset.file_name().as_ref()), payload).unwrap();
            assert_eq!(directory.read(asset).unwrap().as_ref(), payload);
        }
    }
}

#[test]
fn missing_roots_and_native_reads_preserve_directory_role_and_provider_cause() {
    let root = tempfile::tempdir().unwrap();
    let directory = MhrAssetDirectory::from(root.path().to_path_buf());
    let failure = directory.resolve().unwrap_err();
    assert!(std::error::Error::source(&failure).is_none());
    match failure {
        MhrAssetDirectoryError::Missing {
            directory: original,
            nested,
        } => {
            assert_eq!(original, directory);
            assert_eq!(nested, MhrAssetDirectory::from(root.path().join("assets")));
        }
        _ => panic!("expected a missing definition in both namespaces"),
    }
    let role = MhrAsset::Rig(CharacterLod::Reduced);
    let error = directory.read(role).unwrap_err();
    assert!(std::error::Error::source(&error).is_some());
    match error {
        MhrAssetReadError::Native {
            directory: original,
            asset,
            source,
        } => {
            assert_eq!(original, directory);
            assert_eq!(asset, role);
            assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        }
        _ => panic!("expected the native read cause"),
    }
}

#[cfg(unix)]
#[test]
fn an_inspection_failure_does_not_claim_that_assets_are_missing() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("file");
    std::fs::write(&file, []).unwrap();
    let directory = MhrAssetDirectory::from(file);
    let error = directory.resolve().unwrap_err();
    assert!(std::error::Error::source(&error).is_some());
    match error {
        MhrAssetDirectoryError::Inspect {
            directory: original,
            source,
        } => {
            assert_eq!(original, directory);
            assert_eq!(source.kind(), std::io::ErrorKind::NotADirectory);
        }
        _ => panic!("expected an inspection failure"),
    }
}
