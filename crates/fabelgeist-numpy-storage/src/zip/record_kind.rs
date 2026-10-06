//! Record-signature interpretation at the native ZIP byte boundary.

/// A serialized signature retained when its record role is unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ZipSignature(u32);

/// The five record roles interpreted by this reader, or an unknown signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ZipRecordKind {
    EndDirectory,
    CentralMember,
    LocalMember,
    Zip64End,
    Zip64Locator,
    Unknown(ZipSignature),
}

impl ZipRecordKind {
    /// Decodes and classifies the native little-endian signature at `offset`.
    ///
    /// Byte offsets and scalar framing retain the existing decoder contract.
    /// Unknown tags are admitted so each caller keeps its mismatch behavior.
    pub(super) fn read_at(data: &[u8], offset: usize) -> Self {
        let signature = ZipSignature(super::u32_at(data, offset));
        // The native code is projected only at this format interpretation port.
        match signature.0 {
            0x0605_4b50 => Self::EndDirectory,
            0x0201_4b50 => Self::CentralMember,
            0x0403_4b50 => Self::LocalMember,
            0x0606_4b50 => Self::Zip64End,
            0x0706_4b50 => Self::Zip64Locator,
            _ => Self::Unknown(signature),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SignatureCase {
        bytes: [u8; 4],
        expected: ZipRecordKind,
    }

    #[test]
    fn serialized_signatures_select_record_roles() {
        let cases = [
            SignatureCase {
                bytes: *b"PK\x05\x06",
                expected: ZipRecordKind::EndDirectory,
            },
            SignatureCase {
                bytes: *b"PK\x01\x02",
                expected: ZipRecordKind::CentralMember,
            },
            SignatureCase {
                bytes: *b"PK\x03\x04",
                expected: ZipRecordKind::LocalMember,
            },
            SignatureCase {
                bytes: *b"PK\x06\x06",
                expected: ZipRecordKind::Zip64End,
            },
            SignatureCase {
                bytes: *b"PK\x06\x07",
                expected: ZipRecordKind::Zip64Locator,
            },
        ];
        for case in cases {
            let mut framed = vec![0x99];
            framed.extend(case.bytes);
            framed.push(0x88);
            assert_eq!(ZipRecordKind::read_at(&framed, 1), case.expected);
        }
    }

    #[test]
    fn unknown_signatures_retain_native_bits_and_mismatch_known_roles() {
        for code in [0u32, 1, u32::MAX, 0xdead_beef, 0x504b_0506] {
            let kind = ZipRecordKind::read_at(&code.to_le_bytes(), 0);
            assert_eq!(kind, ZipRecordKind::Unknown(ZipSignature(code)));
            assert_ne!(kind, ZipRecordKind::EndDirectory);
            assert_ne!(kind, ZipRecordKind::CentralMember);
            assert_ne!(kind, ZipRecordKind::LocalMember);
            assert_ne!(kind, ZipRecordKind::Zip64End);
            assert_ne!(kind, ZipRecordKind::Zip64Locator);
        }
        assert_ne!(
            ZipRecordKind::read_at(&0u32.to_le_bytes(), 0),
            ZipRecordKind::read_at(&u32::MAX.to_le_bytes(), 0),
        );
    }
}
