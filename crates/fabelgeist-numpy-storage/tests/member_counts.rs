use std::borrow::Cow;

use fabelgeist_numpy_storage::{Npz, ZipArchive};

struct DirectoryCase {
    label: &'static str,
    bytes: &'static [u8],
    names: &'static [&'static str],
}

#[test]
fn declared_counts_preserve_public_directories_and_numpy_keys() {
    let cases = [
        DirectoryCase {
            label: "ordinary-0",
            bytes: include_bytes!("fixtures/member-count/ordinary-0.zip"),
            names: &[],
        },
        DirectoryCase {
            label: "ordinary-1",
            bytes: include_bytes!("fixtures/member-count/ordinary-1.zip"),
            names: &["values.npy"],
        },
        DirectoryCase {
            label: "ordinary-2",
            bytes: include_bytes!("fixtures/member-count/ordinary-2.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "ordinary-3",
            bytes: include_bytes!("fixtures/member-count/ordinary-3.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "ordinary-4095",
            bytes: include_bytes!("fixtures/member-count/ordinary-4095.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "ordinary-4096",
            bytes: include_bytes!("fixtures/member-count/ordinary-4096.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "ordinary-4097",
            bytes: include_bytes!("fixtures/member-count/ordinary-4097.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "ordinary-65534",
            bytes: include_bytes!("fixtures/member-count/ordinary-65534.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "ordinary-65535",
            bytes: include_bytes!("fixtures/member-count/ordinary-65535.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-0",
            bytes: include_bytes!("fixtures/member-count/zip64-0.zip"),
            names: &[],
        },
        DirectoryCase {
            label: "zip64-1",
            bytes: include_bytes!("fixtures/member-count/zip64-1.zip"),
            names: &["values.npy"],
        },
        DirectoryCase {
            label: "zip64-2",
            bytes: include_bytes!("fixtures/member-count/zip64-2.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-3",
            bytes: include_bytes!("fixtures/member-count/zip64-3.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-4095",
            bytes: include_bytes!("fixtures/member-count/zip64-4095.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-4096",
            bytes: include_bytes!("fixtures/member-count/zip64-4096.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-4097",
            bytes: include_bytes!("fixtures/member-count/zip64-4097.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-65535",
            bytes: include_bytes!("fixtures/member-count/zip64-65535.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-65536",
            bytes: include_bytes!("fixtures/member-count/zip64-65536.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-4294967295",
            bytes: include_bytes!("fixtures/member-count/zip64-4294967295.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-4294967296",
            bytes: include_bytes!("fixtures/member-count/zip64-4294967296.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-4294967297",
            bytes: include_bytes!("fixtures/member-count/zip64-4294967297.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-9223372036854775808",
            bytes: include_bytes!("fixtures/member-count/zip64-9223372036854775808.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-18446744073709551614",
            bytes: include_bytes!("fixtures/member-count/zip64-18446744073709551614.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-18446744073709551615",
            bytes: include_bytes!("fixtures/member-count/zip64-18446744073709551615.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-offset-trigger",
            bytes: include_bytes!("fixtures/member-count/zip64-offset-trigger.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-offset-trigger-zero-count",
            bytes: include_bytes!("fixtures/member-count/zip64-offset-trigger-zero-count.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-ordinary-not-triggered",
            bytes: include_bytes!("fixtures/member-count/zip64-ordinary-not-triggered.zip"),
            names: &["values.npy"],
        },
        DirectoryCase {
            label: "zip64-locator-mismatch",
            bytes: include_bytes!("fixtures/member-count/zip64-locator-mismatch.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-end-mismatch",
            bytes: include_bytes!("fixtures/member-count/zip64-end-mismatch.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-record-outside",
            bytes: include_bytes!("fixtures/member-count/zip64-record-outside.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "zip64-disc-count-disagrees",
            bytes: include_bytes!("fixtures/member-count/zip64-disc-count-disagrees.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "ordinary-disc-count-disagrees",
            bytes: include_bytes!("fixtures/member-count/ordinary-disc-count-disagrees.zip"),
            names: &["values.npy", "second.npy"],
        },
        DirectoryCase {
            label: "duplicate-count-2",
            bytes: include_bytes!("fixtures/member-count/duplicate-count-2.zip"),
            names: &["values.npy", "values.npy"],
        },
        DirectoryCase {
            label: "ordinary-1-local-mismatch",
            bytes: include_bytes!("fixtures/member-count/ordinary-1-local-mismatch.zip"),
            names: &["values.npy"],
        },
        DirectoryCase {
            label: "ordinary-large-central-mismatch",
            bytes: include_bytes!("fixtures/member-count/ordinary-large-central-mismatch.zip"),
            names: &[],
        },
        DirectoryCase {
            label: "ordinary-large-partial-directory",
            bytes: include_bytes!("fixtures/member-count/ordinary-large-partial-directory.zip"),
            names: &["values.npy"],
        },
        DirectoryCase {
            label: "ordinary-zero-invalid-name",
            bytes: include_bytes!("fixtures/member-count/ordinary-zero-invalid-name.zip"),
            names: &[],
        },
        DirectoryCase {
            label: "zero-without-central-directory",
            bytes: include_bytes!("fixtures/member-count/zero-without-central-directory.zip"),
            names: &[],
        },
        DirectoryCase {
            label: "saturated-without-locator",
            bytes: include_bytes!("fixtures/member-count/saturated-without-locator.zip"),
            names: &[],
        },
        DirectoryCase {
            label: "ordinary-zero-then-later-end",
            bytes: include_bytes!("fixtures/member-count/ordinary-zero-then-later-end.zip"),
            names: &[],
        },
    ];
    for case in cases {
        let archive = ZipArchive::from_bytes(case.bytes.to_vec()).unwrap();
        assert_eq!(
            archive.names().collect::<Vec<_>>(),
            case.names,
            "{}",
            case.label
        );
        let npz = Npz::from_bytes(case.bytes.to_vec()).unwrap();
        assert_eq!(
            npz.names().collect::<Vec<_>>(),
            case.names,
            "{}",
            case.label
        );
        let expected_keys: Vec<_> = case
            .names
            .iter()
            .map(|name| name.strip_suffix(".npy").unwrap())
            .collect();
        assert_eq!(
            npz.keys().collect::<Vec<_>>(),
            expected_keys,
            "{}",
            case.label
        );
    }
}

fn assert_member_values_and_storage(archive: &ZipArchive, npz: &Npz) {
    let first = archive.bytes("values.npy").unwrap();
    assert!(matches!(&first, Cow::Borrowed(_)));
    assert_eq!(
        first.as_ptr(),
        archive.bytes("values.npy").unwrap().as_ptr()
    );
    assert!(matches!(
        archive.bytes("second.npy").unwrap(),
        Cow::Owned(_)
    ));
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
fn huge_selected_count_keeps_owned_mapped_storage_values_and_errors() {
    let bytes = include_bytes!("fixtures/member-count/zip64-18446744073709551615.zip");
    let archive = ZipArchive::from_bytes(bytes.to_vec()).unwrap();
    let npz = Npz::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(
        archive.names().collect::<Vec<_>>(),
        ["values.npy", "second.npy"]
    );
    assert_member_values_and_storage(&archive, &npz);
    assert_eq!(
        archive.bytes("missing").unwrap_err().to_string(),
        "archive has no member \"missing\""
    );
    let path = std::env::temp_dir().join(format!(
        "zip-member-count-test-{}-{}.zip",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, bytes).unwrap();
    assert_member_values_and_storage(
        &ZipArchive::open(&path).unwrap(),
        &Npz::open(&path).unwrap(),
    );
    std::fs::remove_file(path).unwrap();
    let bytes = include_bytes!("fixtures/member-count/ordinary-1-local-mismatch.zip");
    let archive = ZipArchive::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(
        archive.bytes("values.npy").unwrap_err().to_string(),
        "corrupt zip entry values.npy"
    );
    let npz = Npz::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(
        npz.array("values").err().unwrap().to_string(),
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
        archive.names().collect::<Vec<_>>(),
        ["values.npy", "values.npy"]
    );
    assert!(matches!(
        archive.bytes("values.npy").unwrap(),
        Cow::Borrowed(_)
    ));
    let npz = Npz::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(npz.array("values").unwrap().to_f32()[0], 1.0);
}

// This inherited framing limitation is outside directory cardinality.
#[test]
#[should_panic(expected = "range end index 65838 out of range for slice of length 391")]
fn nonzero_count_keeps_the_existing_malformed_name_panic() {
    let _ = ZipArchive::from_bytes(
        include_bytes!("fixtures/member-count/central-name-overrun.zip").to_vec(),
    );
}
