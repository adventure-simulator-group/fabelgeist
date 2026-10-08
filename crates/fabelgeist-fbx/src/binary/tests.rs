use super::{FbxDecodeError, Reader};
use std::error::Error as _;

#[test]
fn read_overflow_preserves_rejected_native_provenance_and_cursor() {
    // No allocation or huge input is needed to exercise checked native addition.
    let mut reader = Reader {
        data: &[],
        position: usize::MAX,
        version: 7500,
    };
    let error = reader.take(1).unwrap_err();
    assert!(matches!(
        error,
        FbxDecodeError::ReadOverflow {
            native_offset: usize::MAX,
            native_length: 1
        }
    ));
    assert_eq!(error.to_string(), "FBX read overflow");
    assert!(error.source().is_none());
    assert_eq!(reader.position, usize::MAX);
}

#[test]
fn failed_read_keeps_cursor_and_successful_retry_consumes_exact_bytes() {
    let data = include_bytes!("../../tests/fixtures/partial-header.fbx");
    let mut reader = Reader::from_binary(data).unwrap();
    let error = reader.take(6).unwrap_err();
    assert!(matches!(
        error,
        FbxDecodeError::TruncatedRead {
            native_offset: 27,
            native_length: 6
        }
    ));
    assert_eq!(reader.position, 27);
    assert_eq!(reader.take_array::<5>().unwrap(), [0; 5]);
    assert_eq!(reader.position, data.len());
    assert!(matches!(
        reader.u8(),
        Err(FbxDecodeError::TruncatedRead {
            native_length: 1,
            ..
        })
    ));
}
