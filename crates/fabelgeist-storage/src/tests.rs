use super::*;

#[test]
fn spans_accept_exact_end_and_reject_outside_and_overflow() {
    let data = [10, 20, 30, 40];
    let view = StorageView::from(data.as_slice());
    let span = StorageByteSpan {
        offset: StorageByteOffset::from(2),
        length: StorageByteLength::from(2u64),
    };
    assert_eq!(view.portion(span).unwrap().as_ref(), [30, 40]);
    let outside = StorageByteSpan {
        length: StorageByteLength::from(3u64),
        ..span
    };
    assert_eq!(
        view.portion(outside).unwrap_err(),
        StorageBoundsError::Outside {
            requested: outside,
            available: StorageByteLength::from(4u64),
        }
    );
    let overflow = StorageByteSpan {
        offset: StorageByteOffset::from(u64::MAX),
        length: StorageByteLength::from(1u64),
    };
    assert_eq!(
        view.portion(overflow).unwrap_err(),
        StorageBoundsError::Overflow(overflow)
    );
}

#[test]
fn empty_end_views_and_outside_tails_are_distinct() {
    let view = StorageView::from([9, 7].as_slice());
    let end = view.length().end_offset();
    assert_eq!(view.tail_at(end).unwrap().as_ref(), []);
    assert_eq!(
        view.portion(StorageByteSpan {
            offset: end,
            length: StorageByteLength::default(),
        })
        .unwrap()
        .as_ref(),
        []
    );
    assert!(matches!(
        view.tail_at(StorageByteOffset::from(3)),
        Err(StorageBoundsError::Outside { .. })
    ));
}

#[test]
fn distances_are_forward_only_and_additions_are_checked() {
    let start = StorageByteOffset::from(4);
    let end = start.advance(StorageByteLength::from(7u64)).unwrap();
    assert_eq!(
        end.distance_from(start),
        Some(StorageByteLength::from(7u64))
    );
    assert_eq!(start.distance_from(end), None);
    assert_eq!(end.preceding(StorageByteLength::from(7u64)), Some(start));
    assert_eq!(start.preceding(StorageByteLength::from(5u64)), None);
    assert!(matches!(
        StorageByteLength::from(u64::MAX).checked_add(StorageByteLength::from(1u64)),
        Err(StorageBoundsError::Overflow(_))
    ));
    assert_eq!(u64::from(StorageByteLength::from(u64::MAX)), u64::MAX);
}

#[test]
fn little_endian_fields_admit_the_complete_width() {
    let data = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
    let view = StorageView::from(data.as_slice());
    assert_eq!(
        view.decode_u16(StorageByteOffset::default()).unwrap(),
        0x2301
    );
    assert_eq!(
        view.decode_u32(StorageByteOffset::default()).unwrap(),
        0x6745_2301
    );
    assert_eq!(
        view.decode_u64(StorageByteOffset::default()).unwrap(),
        0xefcd_ab89_6745_2301
    );
    assert!(matches!(
        view.decode_u64(StorageByteOffset::from(1)),
        Err(StorageBoundsError::Outside { .. })
    ));
}
