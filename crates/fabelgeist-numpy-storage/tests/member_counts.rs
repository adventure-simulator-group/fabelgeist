use std::borrow::Cow;

use fabelgeist_numpy_storage::{ArchiveMemberName, Npz, NpzArrayName, ZipArchive};

struct DirectoryCase {
    label: &'static str,
    bytes: &'static [u8],
    names: Option<&'static [&'static str]>,
}

#[test]
fn selected_counts_and_checked_admission_preserve_directories_and_numpy_keys() {
    let cases = [
        DirectoryCase {
            label: "ordinary-0",
            bytes: include_bytes!("fixtures/member-count/ordinary-0.zip"),
            names: Some(&[]),
        },
        DirectoryCase {
            label: "ordinary-1",
            bytes: include_bytes!("fixtures/member-count/ordinary-1.zip"),
            names: Some(&["values.npy"]),
        },
        DirectoryCase {
            label: "ordinary-2",
            bytes: include_bytes!("fixtures/member-count/ordinary-2.zip"),
            names: Some(&["values.npy", "second.npy"]),
        },
        DirectoryCase {
            label: "ordinary-3",
            bytes: include_bytes!("fixtures/member-count/ordinary-3.zip"),
            names: Some(&["values.npy", "second.npy"]),
        },
        DirectoryCase {
            label: "ordinary-4095",
            bytes: include_bytes!("fixtures/member-count/ordinary-4095.zip"),
            names: Some(&["values.npy", "second.npy"]),
        },
        DirectoryCase {
            label: "ordinary-4096",
            bytes: include_bytes!("fixtures/member-count/ordinary-4096.zip"),
            names: Some(&["values.npy", "second.npy"]),
        },
        DirectoryCase {
            label: "ordinary-4097",
            bytes: include_bytes!("fixtures/member-count/ordinary-4097.zip"),
            names: Some(&["values.npy", "second.npy"]),
        },
        DirectoryCase {
            label: "ordinary-65534",
            bytes: include_bytes!("fixtures/member-count/ordinary-65534.zip"),
            names: Some(&["values.npy", "second.npy"]),
        },
        DirectoryCase {
            label: "ordinary-65535",
            bytes: include_bytes!("fixtures/member-count/ordinary-65535.zip"),
            names: Some(&["values.npy", "second.npy"]),
        },
        DirectoryCase {
            label: "zip64-0",
            bytes: include_bytes!("fixtures/member-count/zip64-0.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-1",
            bytes: include_bytes!("fixtures/member-count/zip64-1.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-2",
            bytes: include_bytes!("fixtures/member-count/zip64-2.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-3",
            bytes: include_bytes!("fixtures/member-count/zip64-3.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-4095",
            bytes: include_bytes!("fixtures/member-count/zip64-4095.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-4096",
            bytes: include_bytes!("fixtures/member-count/zip64-4096.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-4097",
            bytes: include_bytes!("fixtures/member-count/zip64-4097.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-65535",
            bytes: include_bytes!("fixtures/member-count/zip64-65535.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-65536",
            bytes: include_bytes!("fixtures/member-count/zip64-65536.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-4294967295",
            bytes: include_bytes!("fixtures/member-count/zip64-4294967295.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-4294967296",
            bytes: include_bytes!("fixtures/member-count/zip64-4294967296.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-4294967297",
            bytes: include_bytes!("fixtures/member-count/zip64-4294967297.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-9223372036854775808",
            bytes: include_bytes!("fixtures/member-count/zip64-9223372036854775808.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-18446744073709551614",
            bytes: include_bytes!("fixtures/member-count/zip64-18446744073709551614.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-18446744073709551615",
            bytes: include_bytes!("fixtures/member-count/zip64-18446744073709551615.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-offset-trigger",
            bytes: include_bytes!("fixtures/member-count/zip64-offset-trigger.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-offset-trigger-zero-count",
            bytes: include_bytes!("fixtures/member-count/zip64-offset-trigger-zero-count.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-ordinary-not-triggered",
            bytes: include_bytes!("fixtures/member-count/zip64-ordinary-not-triggered.zip"),
            names: Some(&["values.npy"]),
        },
        DirectoryCase {
            label: "zip64-locator-mismatch",
            bytes: include_bytes!("fixtures/member-count/zip64-locator-mismatch.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-end-mismatch",
            bytes: include_bytes!("fixtures/member-count/zip64-end-mismatch.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-record-outside",
            bytes: include_bytes!("fixtures/member-count/zip64-record-outside.zip"),
            names: None,
        },
        DirectoryCase {
            label: "zip64-disc-count-disagrees",
            bytes: include_bytes!("fixtures/member-count/zip64-disc-count-disagrees.zip"),
            names: None,
        },
        DirectoryCase {
            label: "ordinary-disc-count-disagrees",
            bytes: include_bytes!("fixtures/member-count/ordinary-disc-count-disagrees.zip"),
            names: Some(&["values.npy", "second.npy"]),
        },
        DirectoryCase {
            label: "duplicate-count-2",
            bytes: include_bytes!("fixtures/member-count/duplicate-count-2.zip"),
            names: Some(&["values.npy", "values.npy"]),
        },
        DirectoryCase {
            label: "ordinary-1-local-mismatch",
            bytes: include_bytes!("fixtures/member-count/ordinary-1-local-mismatch.zip"),
            names: Some(&["values.npy"]),
        },
        DirectoryCase {
            label: "ordinary-large-central-mismatch",
            bytes: include_bytes!("fixtures/member-count/ordinary-large-central-mismatch.zip"),
            names: None,
        },
        DirectoryCase {
            label: "ordinary-large-partial-directory",
            bytes: include_bytes!("fixtures/member-count/ordinary-large-partial-directory.zip"),
            names: None,
        },
        DirectoryCase {
            label: "ordinary-zero-invalid-name",
            bytes: include_bytes!("fixtures/member-count/ordinary-zero-invalid-name.zip"),
            names: Some(&[]),
        },
        DirectoryCase {
            label: "zero-without-central-directory",
            bytes: include_bytes!("fixtures/member-count/zero-without-central-directory.zip"),
            names: None,
        },
        DirectoryCase {
            label: "saturated-without-locator",
            bytes: include_bytes!("fixtures/member-count/saturated-without-locator.zip"),
            names: Some(&[]),
        },
        DirectoryCase {
            label: "ordinary-zero-then-later-end",
            bytes: include_bytes!("fixtures/member-count/ordinary-zero-then-later-end.zip"),
            names: Some(&[]),
        },
    ];
    for case in cases {
        let archive = ZipArchive::from_bytes(case.bytes.to_vec())
            .map(|archive| {
                archive
                    .names()
                    .map(|name| name.to_string())
                    .collect::<Vec<_>>()
            })
            .map_err(|error| {
                matches!(
                    error,
                    fabelgeist_numpy_storage::zip::ZipReadError::Archive { .. }
                )
            });
        assert_eq!(
            archive,
            case.names
                .map(|names| names
                    .iter()
                    .map(|name| name.to_string())
                    .collect::<Vec<_>>())
                .ok_or(true),
            "{}",
            case.label
        );
        let npz = Npz::from_bytes(case.bytes.to_vec());
        assert_eq!(
            npz.as_ref()
                .map(|npz| npz.names().map(|name| name.to_string()).collect::<Vec<_>>())
                .map_err(|error| matches!(
                    error.downcast_ref::<fabelgeist_numpy_storage::zip::ZipReadError>(),
                    Some(fabelgeist_numpy_storage::zip::ZipReadError::Archive { .. })
                )),
            case.names
                .map(|names| names
                    .iter()
                    .map(|name| name.to_string())
                    .collect::<Vec<_>>())
                .ok_or(true),
            "{}",
            case.label
        );
        let expected_keys = case
            .names
            .map(|names| {
                names
                    .iter()
                    .map(|name| name.strip_suffix(".npy").unwrap().to_owned())
                    .collect::<Vec<_>>()
            })
            .ok_or(true);
        assert_eq!(
            npz.as_ref()
                .map(|npz| npz.keys().map(|name| name.to_string()).collect::<Vec<_>>())
                .map_err(|error| matches!(
                    error.downcast_ref::<fabelgeist_numpy_storage::zip::ZipReadError>(),
                    Some(fabelgeist_numpy_storage::zip::ZipReadError::Archive { .. })
                )),
            expected_keys,
            "{}",
            case.label
        );
    }
}

#[test]
fn inconsistent_zip64_count_is_rejected_for_owned_and_mapped_archives() {
    let bytes = include_bytes!("fixtures/member-count/zip64-18446744073709551615.zip");
    assert!(matches!(
        ZipArchive::from_bytes(bytes.to_vec()),
        Err(fabelgeist_numpy_storage::zip::ZipReadError::Archive { .. })
    ));
    assert!(Npz::from_bytes(bytes.to_vec()).is_err());
    let path = std::env::temp_dir().join(format!(
        "zip-member-count-test-{}-{}.zip",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, bytes).unwrap();
    assert!(matches!(
        ZipArchive::open(&path),
        Err(fabelgeist_numpy_storage::zip::ZipReadError::Archive { .. })
    ));
    assert!(Npz::open(&path).is_err());
    std::fs::remove_file(path).unwrap();
    let bytes = include_bytes!("fixtures/member-count/ordinary-1-local-mismatch.zip");
    let archive = ZipArchive::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(
        archive
            .bytes(&ArchiveMemberName::from("values.npy"))
            .unwrap_err()
            .to_string(),
        "corrupt zip entry values.npy"
    );
    let npz = Npz::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(
        npz.array(&NpzArrayName::new("values"))
            .err()
            .unwrap()
            .to_string(),
        "corrupt zip entry values.npy"
    );
}

#[test]
fn capacity_cap_does_not_limit_real_members_or_change_duplicate_selection() {
    let bytes = include_bytes!("fixtures/member-count/beyond-initial-capacity.zip");
    let archive = ZipArchive::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(archive.names().count(), 4097);
    let npz = Npz::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(npz.keys().count(), 4097);
    let bytes = include_bytes!("fixtures/member-count/duplicate-count-2.zip");
    let archive = ZipArchive::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(
        archive
            .names()
            .map(|name| name.to_string())
            .collect::<Vec<_>>(),
        ["values.npy", "values.npy"]
    );
    assert!(matches!(
        archive
            .bytes(&ArchiveMemberName::from("values.npy"))
            .unwrap(),
        Cow::Borrowed(_)
    ));
    let npz = Npz::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(
        Vec::<f32>::from(fabelgeist_numpy_storage::NpyFloatValues::from(
            &npz.array(&NpzArrayName::new("values")).unwrap()
        ))[0],
        1.0
    );
}

#[test]
fn nonzero_count_rejects_a_malformed_name_frame() {
    assert!(matches!(
        ZipArchive::from_bytes(
            include_bytes!("fixtures/member-count/central-name-overrun.zip").to_vec(),
        ),
        Err(fabelgeist_numpy_storage::zip::ZipReadError::Archive { .. })
    ));
}
