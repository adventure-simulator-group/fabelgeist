use crate::{
    DirectoryEntry, DirectoryListingError, Entry, FileContents, NativeDirectory,
    NativeListingFailure,
};
use std::error::Error;

#[tokio::test]
async fn listing_admits_immediate_children_with_usable_file_contents() {
    let storage = tempfile::tempdir().unwrap();
    let root = NativeDirectory::from(storage.path().to_path_buf());
    assert!(
        root.list_entries()
            .await
            .unwrap()
            .into_iter()
            .next()
            .is_none()
    );

    std::fs::write(storage.path().join("payload.bin"), [0, 255, 128, 1]).unwrap();
    std::fs::create_dir(storage.path().join("nested")).unwrap();
    std::fs::write(storage.path().join("nested/child"), b"nested contents").unwrap();
    let mut files = Vec::new();
    let mut directories = Vec::new();
    for entry in root.list_entries().await.unwrap() {
        match entry {
            Entry::File(file) => {
                assert_eq!(
                    file.read().await.unwrap(),
                    FileContents::from(vec![0, 255, 128, 1])
                );
                files.push(file.name());
            }
            Entry::Directory(directory) => directories.push(directory.name()),
        }
    }
    assert_eq!(
        files,
        [crate::EntryLabel::from(String::from("payload.bin"))]
    );
    assert_eq!(
        directories,
        [crate::EntryLabel::from(String::from("nested"))]
    );
}

#[tokio::test]
async fn opening_failures_retain_directory_stage_and_original_io_error() {
    let storage = tempfile::tempdir().unwrap();
    let root = NativeDirectory::from(storage.path().join("missing"));
    let failure = root.list_entries().await.unwrap_err();
    match &failure {
        DirectoryListingError::Native { directory, failure } => {
            assert_eq!(directory, &root);
            assert!(matches!(failure, NativeListingFailure::OpenDirectory(_)));
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
    std::fs::write(storage.path().join("file"), b"contents").unwrap();
    let failure = NativeDirectory::from(storage.path().join("file"))
        .list_entries()
        .await
        .unwrap_err();
    assert_eq!(
        failure
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::NotADirectory
    );
}

#[cfg(unix)]
#[tokio::test]
async fn enumeration_follows_symlinks_and_omits_other_filesystem_object_kinds() {
    let storage = tempfile::tempdir().unwrap();
    std::fs::write(storage.path().join("file"), b"contents").unwrap();
    std::fs::create_dir(storage.path().join("directory")).unwrap();
    std::os::unix::fs::symlink("file", storage.path().join("file-link")).unwrap();
    std::os::unix::fs::symlink("directory", storage.path().join("directory-link")).unwrap();
    std::os::unix::fs::symlink("/dev/null", storage.path().join("device-link")).unwrap();
    let root = NativeDirectory::from(storage.path().to_path_buf());
    let mut files = Vec::new();
    let mut directories = Vec::new();
    for entry in root.list_entries().await.unwrap() {
        match entry {
            Entry::File(file) => files.push(file.name()),
            Entry::Directory(directory) => directories.push(directory.name()),
        }
    }
    files.sort();
    directories.sort();
    assert_eq!(
        files,
        [
            crate::EntryLabel::from(String::from("file")),
            crate::EntryLabel::from(String::from("file-link"))
        ]
    );
    assert_eq!(
        directories,
        [
            crate::EntryLabel::from(String::from("directory")),
            crate::EntryLabel::from(String::from("directory-link"))
        ]
    );
}

#[cfg(unix)]
#[tokio::test]
async fn a_child_metadata_failure_is_observable_with_the_child_address() {
    let storage = tempfile::tempdir().unwrap();
    let path = storage.path().join("broken-link");
    std::os::unix::fs::symlink("missing-target", &path).unwrap();
    let root = NativeDirectory::from(storage.path().to_path_buf());
    let failure = root.list_entries().await.unwrap_err();
    match &failure {
        DirectoryListingError::Native { directory, failure } => {
            assert_eq!(directory, &root);
            match failure {
                NativeListingFailure::InspectChild { path: child, .. } => assert_eq!(child, &path),
                _ => panic!("expected a child metadata failure"),
            }
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
}
