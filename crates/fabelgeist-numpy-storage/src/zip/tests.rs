use super::fixture::*;
use super::*;
use fabelgeist_fs::FileContents;
use fabelgeist_storage::{
    StorageBoundsError, StorageByteLength, StorageByteOffset, StorageByteSpan, StorageView,
};

#[test]
fn stored_deflated_and_zip64_members_match_in_memory_and_mapped_storage() {
    for directory in [FixtureDirectory::Ordinary, FixtureDirectory::Zip64] {
        for compression in [FixtureCompression::Stored, FixtureCompression::Deflated] {
            let name = ArchiveMemberName::from("nested/weights.npy");
            let payload = FileContents::from(vec![0, 0xff, 1, 2, 3, 0, 0xff]);
            let fixture = ArchiveFixture::from_members(
                &[FixtureMember {
                    name: name.clone(),
                    payload: payload.clone(),
                    compression,
                }],
                directory,
            );
            let mapped = fixture.mapped();
            for archive in [
                ZipArchive::open(&mapped.file).unwrap(),
                ZipArchive::from_bytes(fixture.contents).unwrap(),
            ] {
                assert_eq!(archive.names().collect::<Vec<_>>(), [&name]);
                assert_eq!(archive.contains(&name), ArchiveMemberPresence::Present);
                assert_eq!(
                    archive.uncompressed_size(&name),
                    Some(StorageByteLength::from(payload.as_ref().len()))
                );
                assert_eq!(
                    archive.bytes(&name).unwrap().view().as_ref(),
                    payload.as_ref()
                );
            }
        }
    }
}
#[test]
fn exact_non_utf8_names_do_not_collide_through_lossy_display() {
    let first = ArchiveMemberName::from(vec![b'a', 0xff]);
    let second = ArchiveMemberName::from(vec![b'a', 0xfe]);
    assert_eq!(first.to_string(), second.to_string());
    assert_ne!(first, second);
    let fixture = ArchiveFixture::from_members(
        &[
            FixtureMember {
                name: first.clone(),
                payload: FileContents::from(vec![1]),
                compression: FixtureCompression::Stored,
            },
            FixtureMember {
                name: second.clone(),
                payload: FileContents::from(vec![2]),
                compression: FixtureCompression::Stored,
            },
        ],
        FixtureDirectory::Ordinary,
    );
    let archive = ZipArchive::from_bytes(fixture.contents).unwrap();
    assert_eq!(archive.bytes(&first).unwrap().view().as_ref(), [1]);
    assert_eq!(archive.bytes(&second).unwrap().view().as_ref(), [2]);
}
#[test]
fn corrupt_directory_admission_never_returns_partial_entries() {
    for fault in [
        FixtureFault::DirectoryCount,
        FixtureFault::NameLength,
        FixtureFault::Zip64Extra,
    ] {
        let directory = if matches!(fault, FixtureFault::Zip64Extra) {
            FixtureDirectory::Zip64
        } else {
            FixtureDirectory::Ordinary
        };
        let fixture = ArchiveFixture::from_members(
            &[FixtureMember {
                name: ArchiveMemberName::from("weights.npy"),
                payload: FileContents::from(vec![1, 2, 3]),
                compression: FixtureCompression::Stored,
            }],
            directory,
        );
        let error = match ZipArchive::from_bytes(fixture.corrupted(fault)) {
            Err(error) => error,
            Ok(_) => panic!("corrupt directory was admitted"),
        };
        match error {
            ZipReadError::Directory { source, .. } => assert!(matches!(
                source,
                ZipDirectoryError::Bounds(_)
                    | ZipDirectoryError::Record(_)
                    | ZipDirectoryError::MissingZip64(_)
            )),
            _ => panic!("expected directory failure"),
        }
    }
}
#[test]
fn member_failures_retain_name_and_precise_classification() {
    let name = ArchiveMemberName::from("weights.npy");
    for fault in [
        FixtureFault::LocalOffset,
        FixtureFault::CompressedLength,
        FixtureFault::DecodedLength,
        FixtureFault::Checksum,
        FixtureFault::Encryption,
        FixtureFault::Compression,
        FixtureFault::Deflate,
    ] {
        let compression = if matches!(fault, FixtureFault::Deflate) {
            FixtureCompression::Deflated
        } else {
            FixtureCompression::Stored
        };
        let fixture = ArchiveFixture::from_members(
            &[FixtureMember {
                name: name.clone(),
                payload: FileContents::from(vec![1, 2, 3, 4]),
                compression,
            }],
            FixtureDirectory::Ordinary,
        );
        let archive = ZipArchive::from_bytes(fixture.corrupted(fault)).unwrap();
        let error = archive.bytes(&name).unwrap_err();
        assert!(std::error::Error::source(&error).is_some());
        match error {
            ZipReadError::Member {
                name: original,
                source,
            } => {
                assert_eq!(original, name);
                assert!(match fault {
                    FixtureFault::LocalOffset | FixtureFault::CompressedLength =>
                        matches!(source, ZipMemberError::Bounds(_)),
                    FixtureFault::DecodedLength => matches!(source, ZipMemberError::Size { .. }),
                    FixtureFault::Checksum => matches!(source, ZipMemberError::Checksum { .. }),
                    FixtureFault::Encryption => matches!(source, ZipMemberError::Encrypted),
                    FixtureFault::Compression =>
                        matches!(source, ZipMemberError::UnsupportedCompression(_)),
                    FixtureFault::Deflate => matches!(source, ZipMemberError::Inflate(_)),
                    _ => unreachable!(),
                });
            }
            _ => panic!("expected member error"),
        }
    }
}
#[test]
fn every_truncated_archive_prefix_rejects_without_panicking() {
    let fixture = ArchiveFixture::from_members(
        &[FixtureMember {
            name: ArchiveMemberName::from("weights.npy"),
            payload: FileContents::from(vec![1, 2, 3, 4]),
            compression: FixtureCompression::Stored,
        }],
        FixtureDirectory::Zip64,
    );
    for end in 0..fixture.contents.as_ref().len() {
        assert!(
            ZipArchive::from_bytes(FileContents::from(
                fixture.contents.as_ref()[..end].to_vec()
            ))
            .is_err(),
            "prefix {end}"
        );
    }
}
#[test]
fn checked_byte_ranges_reject_overflow_and_preserve_empty_end_views() {
    let bytes = [1, 2, 3];
    let view = StorageView::from(bytes.as_slice());
    assert!(
        view.portion(StorageByteSpan {
            offset: StorageByteOffset::from(3u64),
            length: StorageByteLength::default()
        })
        .unwrap()
        .as_ref()
        .is_empty()
    );
    assert!(matches!(
        view.portion(StorageByteSpan {
            offset: StorageByteOffset::from(u64::MAX),
            length: StorageByteLength::from(1u64)
        }),
        Err(StorageBoundsError::Overflow(_))
    ));
    assert!(matches!(
        view.portion(StorageByteSpan {
            offset: StorageByteOffset::from(2u64),
            length: StorageByteLength::from(2u64)
        }),
        Err(StorageBoundsError::Outside { .. })
    ));
}
