use super::*;
use crate::{DirectoryNamespace, FileContents, WorkspaceRecordViolation};
use std::error::Error;

#[tokio::test]
async fn persisted_roots_preserve_absolute_addresses_and_boundary_whitespace() {
    let storage = tempfile::tempdir().unwrap();
    let project_path = storage.path().join(" project root  ");
    std::fs::create_dir(&project_path).unwrap();
    let directory = NativeDirectory::from(project_path.clone());
    let store = WorkspaceStore::from(NativeDirectory::from(storage.path().join("configuration")));
    assert!(matches!(
        store.restore().await.unwrap(),
        WorkspaceRestoration::NoSavedRoot
    ));
    store.save(&directory).await.unwrap();
    match store.restore().await.unwrap() {
        WorkspaceRestoration::Restored(root) => {
            let DirectoryNamespace::Native(restored) = root.namespace();
            assert_eq!(restored, directory);
            assert!(restored.as_ref().is_absolute());
        }
        _ => panic!("expected the saved workspace"),
    }
    std::fs::remove_dir(&project_path).unwrap();
    assert!(
        matches!(store.restore().await.unwrap(), WorkspaceRestoration::MissingRoot(root) if root == directory)
    );
    std::fs::write(&project_path, b"replacement file").unwrap();
    assert!(
        matches!(store.restore().await.unwrap_err(), WorkspaceError::NotDirectory { directory: root } if root == directory)
    );
}

#[cfg(unix)]
#[tokio::test]
async fn native_filename_bytes_survive_persistence_and_only_display_is_lossy() {
    use crate::DirectoryEntry;
    use std::os::unix::ffi::OsStringExt;
    let storage = tempfile::tempdir().unwrap();
    let path = storage
        .path()
        .join(std::ffi::OsString::from_vec(b"project-\xff \n".to_vec()));
    std::fs::create_dir(&path).unwrap();
    let directory = NativeDirectory::from(path.clone());
    assert_eq!(directory.name().to_string(), "project-� \n");
    let store = WorkspaceStore::from(NativeDirectory::from(storage.path().join("configuration")));
    store.save(&directory).await.unwrap();
    match store.restore().await.unwrap() {
        WorkspaceRestoration::Restored(root) => {
            let DirectoryNamespace::Native(restored) = root.namespace();
            assert_eq!(restored.as_ref(), path);
        }
        _ => panic!("expected the exact native root"),
    }
}

#[cfg(unix)]
#[tokio::test]
async fn malformed_records_retain_exact_payloads_and_admission_classification() {
    let storage = tempfile::tempdir().unwrap();
    let store = WorkspaceStore::from(NativeDirectory::from(storage.path().to_path_buf()));
    for (contents, expected) in [
        (FileContents::default(), WorkspaceRecordViolation::Empty),
        (
            FileContents::from(b"/path\0suffix".to_vec()),
            WorkspaceRecordViolation::InteriorNul,
        ),
        (
            FileContents::from(b"relative/path".to_vec()),
            WorkspaceRecordViolation::RelativeAddress,
        ),
    ] {
        store.record.write(&contents).await.unwrap();
        match store.restore().await.unwrap_err() {
            WorkspaceError::InvalidRecord {
                contents: rejected,
                violation,
            } => {
                assert_eq!(rejected, contents);
                assert_eq!(violation, expected);
            }
            _ => panic!("expected invalid saved root admission"),
        }
    }
}

#[tokio::test]
async fn storage_failures_retain_store_record_and_original_io_causes() {
    let storage = tempfile::tempdir().unwrap();
    let project = NativeDirectory::from(storage.path().to_path_buf());
    let blocked = storage.path().join("blocked");
    std::fs::write(&blocked, b"not a directory").unwrap();
    let store = WorkspaceStore::from(NativeDirectory::from(blocked.clone()));
    let failure = store.save(&project).await.unwrap_err();
    match &failure {
        WorkspaceError::CreateStore { directory, .. } => assert_eq!(directory.as_ref(), blocked),
        _ => panic!("expected store creation failure"),
    }
    assert!(failure.source().unwrap().is::<std::io::Error>());

    let store = WorkspaceStore::from(project);
    std::fs::create_dir(storage.path().join("workspace.path")).unwrap();
    let failure = store.restore().await.unwrap_err();
    assert!(matches!(failure, WorkspaceError::Record(_)));
    assert!(
        failure
            .source()
            .unwrap()
            .source()
            .unwrap()
            .is::<std::io::Error>()
    );
}
