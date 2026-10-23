use std::borrow::Cow;

use fabelgeist_numpy_storage::{Npz, ZipArchive};

fn fixture(label: &str) -> &'static [u8] {
    match label {
        "stored-deflated" => include_bytes!("fixtures/record-kinds/stored-deflated.zip"),
        "nearest-end-is-empty" => include_bytes!("fixtures/record-kinds/nearest-end-is-empty.zip"),
        "nearest-end-selects-second" => {
            include_bytes!("fixtures/record-kinds/nearest-end-selects-second.zip")
        }
        "end-with-comment-candidate" => {
            include_bytes!("fixtures/record-kinds/end-with-comment-candidate.zip")
        }
        "end-window-inclusive" => include_bytes!("fixtures/record-kinds/end-window-inclusive.zip"),
        "end-window-excluded" => include_bytes!("fixtures/record-kinds/end-window-excluded.zip"),
        "unknown-central-3735928559" => {
            include_bytes!("fixtures/record-kinds/unknown-central-3735928559.zip")
        }
        "unknown-local-3735928559" => {
            include_bytes!("fixtures/record-kinds/unknown-local-3735928559.zip")
        }
        "central-second-is-local" => {
            include_bytes!("fixtures/record-kinds/central-second-is-local.zip")
        }
        "local-is-central" => include_bytes!("fixtures/record-kinds/local-is-central.zip"),
        "zip64-count-saturated" => {
            include_bytes!("fixtures/record-kinds/zip64-count-saturated.zip")
        }
        "zip64-offset-saturated" => {
            include_bytes!("fixtures/record-kinds/zip64-offset-saturated.zip")
        }
        "zip64-locator-mismatch" => {
            include_bytes!("fixtures/record-kinds/zip64-locator-mismatch.zip")
        }
        "zip64-end-mismatch" => include_bytes!("fixtures/record-kinds/zip64-end-mismatch.zip"),
        "zip64-locator-known-wrong" => {
            include_bytes!("fixtures/record-kinds/zip64-locator-known-wrong.zip")
        }
        "zip64-end-known-wrong" => {
            include_bytes!("fixtures/record-kinds/zip64-end-known-wrong.zip")
        }
        "zip64-record-outside" => include_bytes!("fixtures/record-kinds/zip64-record-outside.zip"),
        "zip64-not-triggered" => include_bytes!("fixtures/record-kinds/zip64-not-triggered.zip"),
        "duplicate-members" => include_bytes!("fixtures/record-kinds/duplicate-members.zip"),
        "stored-central-deflate-local" => {
            include_bytes!("fixtures/record-kinds/stored-central-deflate-local.zip")
        }
        "deflate-central-stored-local" => {
            include_bytes!("fixtures/record-kinds/deflate-central-stored-local.zip")
        }
        "different-local-name-extra" => {
            include_bytes!("fixtures/record-kinds/different-local-name-extra.zip")
        }
        "zip64-entry-extra" => include_bytes!("fixtures/record-kinds/zip64-entry-extra.zip"),
        "short-end-no-locator" => include_bytes!("fixtures/record-kinds/short-end-no-locator.zip"),
        "central-name-overrun" => include_bytes!("fixtures/record-kinds/central-name-overrun.zip"),
        _ => panic!("unknown authored fixture {label}"),
    }
}

struct DirectoryCase {
    label: &'static str,
    names: &'static [&'static str],
}

#[test]
fn directory_and_zip64_signatures_keep_existing_selection_and_fallbacks() {
    let cases = [
        DirectoryCase {
            label: "stored-deflated",
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "nearest-end-is-empty",
            names: &[],
        },
        DirectoryCase {
            label: "nearest-end-selects-second",
            names: &["second.npy"],
        },
        DirectoryCase {
            label: "end-with-comment-candidate",
            names: &[],
        },
        DirectoryCase {
            label: "end-window-inclusive",
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "unknown-central-3735928559",
            names: &[],
        },
        DirectoryCase {
            label: "central-second-is-local",
            names: &["values.npy"],
        },
        DirectoryCase {
            label: "zip64-count-saturated",
            names: &["values.npy"],
        },
        DirectoryCase {
            label: "zip64-offset-saturated",
            names: &["values.npy"],
        },
        DirectoryCase {
            label: "zip64-locator-mismatch",
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-end-mismatch",
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-locator-known-wrong",
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-end-known-wrong",
            // After fallback, this central signature admits an empty third member.
            names: &["values.npy", "second.npy", ""],
        },
        DirectoryCase {
            label: "zip64-record-outside",
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-not-triggered",
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "short-end-no-locator",
            names: &[],
        },
    ];
    for case in cases {
        let archive = ZipArchive::from_bytes(fixture(case.label).to_vec()).unwrap();
        assert_eq!(
            archive.names().collect::<Vec<_>>(),
            case.names,
            "{}",
            case.label
        );
    }
    let error = ZipArchive::from_bytes(fixture("end-window-excluded").to_vec())
        .err()
        .unwrap();
    assert_eq!(
        error.to_string(),
        "not a zip archive: no end-of-central-directory record"
    );
}

#[test]
fn wrong_local_signatures_retain_entry_errors() {
    for label in ["unknown-local-3735928559", "local-is-central"] {
        let archive = ZipArchive::from_bytes(fixture(label).to_vec()).unwrap();
        assert!(archive.contains("values.npy"));
        assert_eq!(
            archive.bytes("values.npy").unwrap_err().to_string(),
            "corrupt zip entry values.npy"
        );
        let npz = Npz::from_bytes(fixture(label).to_vec()).unwrap();
        assert_eq!(
            npz.array("values").err().unwrap().to_string(),
            "corrupt zip entry values.npy"
        );
    }
}

fn assert_member_storage_and_values(archive: &ZipArchive, npz: &Npz) {
    let stored = archive.bytes("values.npy").unwrap();
    assert!(matches!(&stored, Cow::Borrowed(_)));
    assert_eq!(
        stored.as_ptr(),
        archive.bytes("values.npy").unwrap().as_ptr()
    );
    assert!(matches!(
        archive.bytes("second.npy").unwrap(),
        Cow::Owned(_)
    ));
    assert_eq!(npz.keys().collect::<Vec<_>>(), ["values", "second"]);
    assert_eq!(npz.array("values").unwrap().shape, [4]);
    assert_eq!(
        npz.array("values")
            .unwrap()
            .to_f32()
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        [0x3f800000, 0x80000000, 0x7fc01234, 0x40200000]
    );
    assert_eq!(npz.array("second").unwrap().to_f32(), [5.0, 6.0, 7.0, 8.0]);
}

#[test]
fn owned_and_mapped_archives_keep_borrowed_and_decoded_members() {
    let bytes = fixture("stored-deflated");
    assert_member_storage_and_values(
        &ZipArchive::from_bytes(bytes.to_vec()).unwrap(),
        &Npz::from_bytes(bytes.to_vec()).unwrap(),
    );
    let path = std::env::temp_dir().join(format!(
        "zip-record-kinds-test-{}-{}.zip",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    std::fs::write(&path, bytes).unwrap();
    assert_member_storage_and_values(
        &ZipArchive::open(&path).unwrap(),
        &Npz::open(&path).unwrap(),
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn duplicate_and_disagreeing_headers_keep_member_semantics() {
    let bytes = fixture("duplicate-members");
    let archive = ZipArchive::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(
        archive.names().collect::<Vec<_>>(),
        ["values.npy", "values.npy"]
    );
    assert!(matches!(
        archive.bytes("values.npy").unwrap(),
        Cow::Borrowed(_)
    ));
    let npz = Npz::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(npz.array("values").unwrap().to_f32()[0], 1.0);
    for label in [
        "stored-central-deflate-local",
        "deflate-central-stored-local",
        "different-local-name-extra",
        "zip64-entry-extra",
    ] {
        let archive = ZipArchive::from_bytes(fixture(label).to_vec()).unwrap();
        assert_eq!(archive.uncompressed_size("values.npy"), Some(96));
        assert_eq!(
            matches!(archive.bytes("values.npy").unwrap(), Cow::Owned(_)),
            label == "deflate-central-stored-local"
        );
        let npz = Npz::from_bytes(fixture(label).to_vec()).unwrap();
        assert_eq!(npz.array("values").unwrap().to_f32()[0], 1.0, "{label}");
    }
}

// This is an inherited framing limitation, outside signature interpretation.
#[test]
#[should_panic(expected = "range end index 65838 out of range for slice of length 391")]
fn malformed_name_frame_keeps_existing_panic() {
    let _ = ZipArchive::from_bytes(fixture("central-name-overrun").to_vec());
}
