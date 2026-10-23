use std::borrow::Cow;
use std::panic::{AssertUnwindSafe, catch_unwind};

use fabelgeist_numpy_storage::{
    ArchiveByteOffset, ArchiveMemberName, CompressedByteLength, DeclaredUncompressedByteLength,
    Npz, NpzArrayName, ZipArchive,
};

struct WireCase {
    compressed: u32,
    declared: u32,
    offset: u32,
    extra: Vec<u8>,
}

fn zip(case: WireCase) -> Vec<u8> {
    let mut bytes = include_bytes!("fixtures/member-count/ordinary-1.zip").to_vec();
    let directory = bytes.windows(4).position(|v| v == b"PK\x01\x02").unwrap();
    let name_len =
        u16::from_le_bytes(bytes[directory + 28..directory + 30].try_into().unwrap()) as usize;
    let old_extra_len =
        u16::from_le_bytes(bytes[directory + 30..directory + 32].try_into().unwrap()) as usize;
    let extra_start = directory + 46 + name_len;
    bytes[directory + 20..directory + 24].copy_from_slice(&case.compressed.to_le_bytes());
    bytes[directory + 24..directory + 28].copy_from_slice(&case.declared.to_le_bytes());
    bytes[directory + 42..directory + 46].copy_from_slice(&case.offset.to_le_bytes());
    bytes[directory + 30..directory + 32].copy_from_slice(&(case.extra.len() as u16).to_le_bytes());
    bytes.splice(extra_start..extra_start + old_extra_len, case.extra);
    bytes
}

fn extra(values: &[u64]) -> Vec<u8> {
    let mut bytes = vec![1, 0];
    bytes.extend_from_slice(&((values.len() * 8) as u16).to_le_bytes());
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn wire() -> WireCase {
    WireCase {
        compressed: 96,
        declared: 96,
        offset: 0,
        extra: Vec::new(),
    }
}

#[test]
fn every_saturation_combination_replaces_only_requested_fields_in_order() {
    for mask in 0..8 {
        let mut case = wire();
        let mut values = Vec::new();
        if mask & 1 != 0 {
            case.declared = u32::MAX;
            values.push(3);
        }
        if mask & 2 != 0 {
            case.compressed = u32::MAX;
            values.push(96);
        }
        if mask & 4 != 0 {
            case.offset = u32::MAX;
            values.push(0);
        }
        case.extra = extra(&values);
        let bytes = zip(case);
        let archive = ZipArchive::from_bytes(bytes.clone()).unwrap();
        let expected =
            DeclaredUncompressedByteLength::from_bytes(if mask & 1 != 0 { 3 } else { 96 });
        assert_eq!(
            archive.uncompressed_size(&ArchiveMemberName::from("values.npy")),
            Some(expected)
        );
        assert!(matches!(
            archive
                .bytes(&ArchiveMemberName::from("values.npy"))
                .unwrap(),
            Cow::Borrowed(_)
        ));
        let npz = Npz::from_bytes(bytes).unwrap();
        assert_eq!(
            npz.uncompressed_size(&NpzArrayName::new("values")),
            Some(expected)
        );
        assert_eq!(
            npz.uncompressed_size(&NpzArrayName::new("values.npy")),
            Some(expected)
        );
        assert_eq!(
            Vec::<f32>::from(
                fabelgeist_numpy_storage::NpyFloatValues::try_from(
                    &npz.array(&NpzArrayName::new("values")).unwrap()
                )
                .unwrap()
            )
            .len(),
            4
        );
    }
}

#[test]
fn partial_zip64_fields_retain_unreplaced_saturation_and_first_extra() {
    let mut case = wire();
    case.declared = u32::MAX;
    case.compressed = u32::MAX;
    case.offset = u32::MAX;
    case.extra = extra(&[17]);
    let archive = ZipArchive::from_bytes(zip(case)).unwrap();
    assert_eq!(
        archive.uncompressed_size(&ArchiveMemberName::from("values.npy")),
        Some(DeclaredUncompressedByteLength::from_bytes(17))
    );
    assert!(
        archive
            .bytes(&ArchiveMemberName::from("values.npy"))
            .is_err()
    );
    let mut case = wire();
    case.declared = u32::MAX;
    case.extra = extra(&[17]);
    case.extra.extend_from_slice(&extra(&[23]));
    let archive = ZipArchive::from_bytes(zip(case)).unwrap();
    assert_eq!(
        archive.uncompressed_size(&ArchiveMemberName::from("values.npy")),
        Some(DeclaredUncompressedByteLength::from_bytes(17))
    );
    let mut case = wire();
    case.declared = u32::MAX;
    case.extra = vec![1, 0, 7, 0, 1, 2, 3, 4, 5, 6, 7];
    let archive = ZipArchive::from_bytes(zip(case)).unwrap();
    assert_eq!(
        archive.uncompressed_size(&ArchiveMemberName::from("values.npy")),
        Some(DeclaredUncompressedByteLength::from_bytes(u64::from(
            u32::MAX
        )))
    );
}

#[test]
fn declared_full_width_zero_and_mismatch_do_not_measure_payloads() {
    for value in [0, 1, u64::from(u32::MAX), 1 << 32, 1 << 63, u64::MAX] {
        let mut case = wire();
        case.declared = u32::MAX;
        case.extra = extra(&[value]);
        let archive = ZipArchive::from_bytes(zip(case)).unwrap();
        let size = archive
            .uncompressed_size(&ArchiveMemberName::from("values.npy"))
            .unwrap();
        assert_eq!(size, DeclaredUncompressedByteLength::from_bytes(value));
        assert_eq!(size.bytes(), value);
        assert_eq!(
            archive
                .bytes(&ArchiveMemberName::from("values.npy"))
                .unwrap()
                .len(),
            96
        );
        assert!(
            archive
                .uncompressed_size(&ArchiveMemberName::from("missing"))
                .is_none()
        );
    }
}

#[test]
fn inherited_numpy_storage_and_first_duplicate_selection_remain_intact() {
    let bytes = include_bytes!("fixtures/member-count/ordinary-2.zip");
    let archive = ZipArchive::from_bytes(bytes.to_vec()).unwrap();
    assert!(matches!(
        archive
            .bytes(&ArchiveMemberName::from("values.npy"))
            .unwrap(),
        Cow::Borrowed(_)
    ));
    assert!(matches!(
        archive
            .bytes(&ArchiveMemberName::from("second.npy"))
            .unwrap(),
        Cow::Owned(_)
    ));
    let npz = Npz::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(
        Vec::<f32>::from(
            fabelgeist_numpy_storage::NpyFloatValues::try_from(
                &npz.array(&NpzArrayName::new("values")).unwrap()
            )
            .unwrap()
        )
        .iter()
        .map(|v| v.to_bits())
        .collect::<Vec<_>>(),
        [0x3f800000, 0x80000000, 0x7fc01234, 0x40200000]
    );
    let bytes = include_bytes!("fixtures/member-count/duplicate-count-2.zip");
    let npz = Npz::from_bytes(bytes.to_vec()).unwrap();
    assert_eq!(
        Vec::<f32>::from(
            fabelgeist_numpy_storage::NpyFloatValues::try_from(
                &npz.array(&NpzArrayName::new("values")).unwrap()
            )
            .unwrap()
        )[0],
        1.0
    );
    assert_eq!(
        npz.uncompressed_size(&NpzArrayName::new("values")),
        Some(DeclaredUncompressedByteLength::from_bytes(96))
    );
}

#[test]
fn native_numeric_formatter_options_survive_each_bespoke_quantity() {
    for value in [0, 3, 1 << 32, u64::MAX] {
        let decoded = DeclaredUncompressedByteLength::from_bytes(value);
        let compressed = CompressedByteLength::from_bytes(value);
        let offset = ArchiveByteOffset::from_archive_start(value);
        for actual in [
            format!("{decoded:>24.3}"),
            format!("{compressed:>24.3}"),
            format!("{offset:>24.3}"),
        ] {
            assert_eq!(actual, format!("{value:>24.3}"));
        }
        for actual in [
            format!("{decoded:#024?}"),
            format!("{compressed:#024?}"),
            format!("{offset:#024?}"),
        ] {
            assert_eq!(actual, format!("{value:#024?}"));
        }
    }
}

#[test]
fn inherited_offset_cast_and_error_panic_policy_remain_observable() {
    let mut case = wire();
    case.offset = u32::MAX;
    case.extra = extra(&[1 << 32]);
    let archive = ZipArchive::from_bytes(zip(case)).unwrap();
    assert_eq!(
        archive
            .bytes(&ArchiveMemberName::from("values.npy"))
            .unwrap_err()
            .to_string(),
        "corrupt zip entry values.npy"
    );
    let mut case = wire();
    case.offset = u32::MAX;
    case.extra = extra(&[u64::MAX]);
    let archive = ZipArchive::from_bytes(zip(case)).unwrap();
    assert!(
        catch_unwind(AssertUnwindSafe(
            || archive.bytes(&ArchiveMemberName::from("values.npy"))
        ))
        .is_err()
    );
}
