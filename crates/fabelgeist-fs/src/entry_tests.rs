use crate::{
    DirectoryEntry, EntryAccessError, EntryLookupIntent, EntryName, EntryNameViolation,
    EntryOperation, NativeDirectory,
};
use std::error::Error;

#[test]
fn child_admission_rejects_paths_traversal_drive_prefixes_and_nul() {
    for (spelling, violation) in [
        ("", EntryNameViolation::Empty),
        (".", EntryNameViolation::DotComponent),
        ("..", EntryNameViolation::DotComponent),
        ("nested/file", EntryNameViolation::PathSeparator),
        ("nested\\file", EntryNameViolation::PathSeparator),
        ("C:child", EntryNameViolation::DrivePrefix),
        ("z:", EntryNameViolation::DrivePrefix),
        ("file\0name", EntryNameViolation::InteriorNul),
    ] {
        assert_eq!(
            EntryName::try_from(spelling).unwrap_err().violation(),
            violation
        );
    }
    for spelling in [
        "résumé.glb",
        "file name",
        ".hidden",
        "file..name",
        "version:one",
    ] {
        let name = EntryName::try_from(spelling).unwrap();
        assert_eq!(name.as_ref(), spelling);
    }
}

#[tokio::test]
async fn existing_intent_does_not_create_children() {
    let storage = tempfile::tempdir().unwrap();
    let root = NativeDirectory::from(storage.path().to_path_buf());
    let file = EntryName::try_from("missing.txt").unwrap();
    let directory = EntryName::try_from("missing-directory").unwrap();
    root.get_file(&file, EntryLookupIntent::Existing)
        .await
        .unwrap();
    root.get_directory(&directory, EntryLookupIntent::Existing)
        .await
        .unwrap();
    assert!(!storage.path().join(file.as_ref()).exists());
    assert!(!storage.path().join(directory.as_ref()).exists());
}

#[tokio::test]
async fn creation_creates_missing_children_and_preserves_existing_bytes() {
    let storage = tempfile::tempdir().unwrap();
    let root = NativeDirectory::from(storage.path().to_path_buf());
    let file = EntryName::try_from("payload.txt").unwrap();
    let directory = EntryName::try_from("nested").unwrap();
    let intent = EntryLookupIntent::CreateIfMissing;
    root.get_file(&file, intent).await.unwrap();
    root.get_directory(&directory, intent).await.unwrap();
    assert!(storage.path().join(file.as_ref()).is_file());
    assert!(storage.path().join(directory.as_ref()).is_dir());
    std::fs::write(storage.path().join(file.as_ref()), b"retained payload").unwrap();
    root.get_file(&file, intent).await.unwrap();
    root.get_directory(&directory, intent).await.unwrap();
    assert_eq!(
        std::fs::read(storage.path().join(file.as_ref())).unwrap(),
        b"retained payload"
    );
}

#[tokio::test]
async fn access_failures_retain_operation_child_parent_and_io_cause() {
    let storage = tempfile::tempdir().unwrap();
    let missing_parent = storage.path().join("missing-parent");
    let root = NativeDirectory::from(missing_parent.clone());
    let file = EntryName::try_from("child.txt").unwrap();
    let failure = root
        .get_file(&file, EntryLookupIntent::CreateIfMissing)
        .await
        .unwrap_err();
    match &failure {
        EntryAccessError::Native {
            operation,
            directory,
            entry,
            source,
        } => {
            assert_eq!(*operation, EntryOperation::LookupFile);
            assert_eq!(directory, &missing_parent);
            assert_eq!(entry, &file);
            assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        }
    }
    assert_eq!(
        failure
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::NotFound
    );
    let root = NativeDirectory::from(storage.path().to_path_buf());
    let failure = root.delete_entry(&file).await.unwrap_err();
    match failure {
        EntryAccessError::Native {
            operation, entry, ..
        } => {
            assert_eq!(operation, EntryOperation::Delete);
            assert_eq!(entry, file);
        }
    }
}

#[tokio::test]
async fn deleting_a_directory_removes_its_children() {
    let storage = tempfile::tempdir().unwrap();
    let root = NativeDirectory::from(storage.path().to_path_buf());
    let directory = EntryName::try_from("nested").unwrap();
    let child = root
        .get_directory(&directory, EntryLookupIntent::CreateIfMissing)
        .await
        .unwrap();
    child
        .get_file(
            &EntryName::try_from("child").unwrap(),
            EntryLookupIntent::CreateIfMissing,
        )
        .await
        .unwrap();
    root.delete_entry(&directory).await.unwrap();
    assert!(!storage.path().join(directory.as_ref()).exists());
}
