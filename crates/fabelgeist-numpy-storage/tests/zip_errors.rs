use std::{borrow::Cow, error::Error, io, path::Path};

use fabelgeist_numpy_storage::{
    Npz, ZipArchive,
    zip::{Result, ZipFileOperation, ZipReadError},
};

const STORED: &[u8] = include_bytes!("fixtures/zip-errors/stored.zip");
const DEFLATED: &[u8] = include_bytes!("fixtures/zip-errors/deflated.zip");

fn rejected<T>(result: Result<T>) -> ZipReadError {
    result
        .err()
        .expect("fixture must reject at the observed boundary")
}

#[test]
fn absent_end_and_member_have_distinct_public_classifications() {
    let error = rejected(ZipArchive::from_bytes(
        include_bytes!("fixtures/zip-errors/no-end.zip").to_vec(),
    ));
    assert!(matches!(error, ZipReadError::MissingEndRecord));
    assert_eq!(
        error.to_string(),
        "not a zip archive: no end-of-central-directory record"
    );
    assert!(error.source().is_none());

    let archive = ZipArchive::from_bytes(STORED.to_vec()).unwrap();
    let query = "absent\0\nλ";
    let error = rejected(archive.bytes(query));
    assert!(
        matches!(&error, ZipReadError::MissingMember { native_query } if native_query == query)
    );
    assert_eq!(
        error.to_string(),
        format!("archive has no member {query:?}")
    );
    assert!(error.source().is_none());
}

#[test]
fn local_header_and_payload_extent_keep_their_earlier_failure_precedence() {
    // Native fixture encoding port: the end record names the authored central
    // header; only its compression word is replaced with a rejected wire code.
    // Earlier local-header/extent checks must still win over that method.
    let mut corrupt = include_bytes!("fixtures/zip-errors/corrupt-local.zip").to_vec();
    let directory = u32::from_le_bytes(
        corrupt[corrupt.len() - 6..corrupt.len() - 2]
            .try_into()
            .unwrap(),
    ) as usize;
    corrupt[directory + 10..directory + 12].copy_from_slice(&u16::MAX.to_le_bytes());
    let archive = ZipArchive::from_bytes(corrupt).unwrap();
    let error = rejected(archive.bytes("values.npy"));
    assert!(
        matches!(&error, ZipReadError::CorruptMember { native_name } if native_name == "values.npy")
    );
    assert_eq!(error.to_string(), "corrupt zip entry values.npy");
    assert!(error.source().is_none());

    let mut past = include_bytes!("fixtures/zip-errors/past-end.zip").to_vec();
    let directory =
        u32::from_le_bytes(past[past.len() - 6..past.len() - 2].try_into().unwrap()) as usize;
    past[directory + 10..directory + 12].copy_from_slice(&u16::MAX.to_le_bytes());
    let archive = ZipArchive::from_bytes(past).unwrap();
    let error = rejected(archive.bytes("values.npy"));
    assert!(
        matches!(&error, ZipReadError::MemberPastEnd { native_name } if native_name == "values.npy")
    );
    assert_eq!(
        error.to_string(),
        "zip entry values.npy runs past the end of the archive"
    );
    assert!(error.source().is_none());
}

#[test]
fn unsupported_method_retains_the_rejected_native_word() {
    let archive = ZipArchive::from_bytes(
        include_bytes!("fixtures/zip-errors/unsupported-method.zip").to_vec(),
    )
    .unwrap();
    let error = rejected(archive.bytes("values.npy"));
    assert!(matches!(
        error,
        ZipReadError::UnsupportedCompression {
            native_code: u16::MAX
        }
    ));
    assert_eq!(
        error.to_string(),
        "unsupported zip compression method 65535"
    );
    assert!(error.source().is_none());
}

#[test]
fn inflate_failure_retains_the_native_cause_through_npz_anyhow() {
    let input = include_bytes!("fixtures/zip-errors/bad-deflate.zip");
    let archive = ZipArchive::from_bytes(input.to_vec()).unwrap();
    let error = rejected(archive.bytes("values.npy"));
    let ZipReadError::Inflate {
        native_name,
        source,
    } = &error
    else {
        panic!("unexpected classification: {error:?}")
    };
    assert_eq!(native_name, "values.npy");
    assert_eq!(source.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(source.to_string(), "corrupt deflate stream");
    let retained = error.source().unwrap().downcast_ref::<io::Error>().unwrap();
    assert!(std::ptr::eq(source, retained));
    assert_eq!(error.to_string(), "inflating zip entry values.npy");

    let npz = Npz::from_bytes(input.to_vec()).unwrap();
    let outer = npz.array("values").err().unwrap();
    assert!(matches!(
        outer.downcast_ref::<ZipReadError>(),
        Some(ZipReadError::Inflate { .. })
    ));
    assert!(outer.downcast_ref::<io::Error>().is_none());
    assert_eq!(
        outer.chain().map(ToString::to_string).collect::<Vec<_>>(),
        ["inflating zip entry values.npy", "corrupt deflate stream"]
    );
    assert_eq!(
        outer
            .chain()
            .find_map(|cause| cause.downcast_ref::<io::Error>())
            .unwrap()
            .kind(),
        io::ErrorKind::InvalidInput
    );
}

#[test]
fn native_open_failure_retains_path_operation_and_original_io() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/zip-errors/not-present.zip");
    let error = rejected(ZipArchive::open(&path));
    let ZipReadError::Native {
        operation,
        native_path,
        source,
    } = &error
    else {
        panic!("unexpected classification: {error:?}")
    };
    assert_eq!(*operation, ZipFileOperation::Open);
    assert_eq!(native_path, &path);
    assert_eq!(source.kind(), io::ErrorKind::NotFound);
    assert_eq!(error.to_string(), format!("opening {}", path.display()));
    assert!(std::ptr::eq(
        source,
        error.source().unwrap().downcast_ref::<io::Error>().unwrap()
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn native_directory_map_failure_is_distinct_from_open() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/zip-errors");
    let error = rejected(ZipArchive::open(&path));
    let ZipReadError::Native {
        operation,
        native_path,
        source,
    } = &error
    else {
        panic!("unexpected classification: {error:?}")
    };
    assert_eq!(*operation, ZipFileOperation::Map);
    assert_eq!(native_path, &path);
    assert_eq!(source.raw_os_error(), Some(19));
    assert_eq!(
        error.to_string(),
        format!("memory-mapping {}", path.display())
    );
    assert!(std::ptr::eq(
        source,
        error.source().unwrap().downcast_ref::<io::Error>().unwrap()
    ));
}

#[test]
fn successful_stored_and_deflated_reads_preserve_borrowing_and_float_bits() {
    let stored = ZipArchive::from_bytes(STORED.to_vec()).unwrap();
    let deflated = ZipArchive::from_bytes(DEFLATED.to_vec()).unwrap();
    let borrowed = stored.bytes("values.npy").unwrap();
    let repeated = stored.bytes("values.npy").unwrap();
    let decoded = deflated.bytes("values.npy").unwrap();
    assert!(matches!(borrowed, Cow::Borrowed(_)));
    assert!(matches!(decoded, Cow::Owned(_)));
    assert!(std::ptr::eq(borrowed.as_ptr(), repeated.as_ptr()));
    assert_eq!(borrowed, decoded);
    for input in [STORED, DEFLATED] {
        let npz = Npz::from_bytes(input.to_vec()).unwrap();
        let array = npz.array("values").unwrap();
        assert_eq!(
            array
                .to_f32()
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            [0, 0x8000_0000, 1, 0x7f80_0000, 0xff80_0000, 0x7fc0_1234]
        );
    }
}
