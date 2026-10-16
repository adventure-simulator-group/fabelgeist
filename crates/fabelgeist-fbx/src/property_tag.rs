//! Supported property tags decoded from one-byte wire fields.

use derive_more::TryFrom;

/// A supported scalar, array, string or raw-byte property tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFrom)]
#[repr(u8)]
#[try_from(repr)]
pub(super) enum FbxPropertyTag {
    Integer16 = b'Y',
    Boolean = b'C',
    Integer32 = b'I',
    Integer64 = b'L',
    Float32 = b'F',
    Float64 = b'D',
    ArrayFloat32 = b'f',
    ArrayFloat64 = b'd',
    ArrayInteger32 = b'i',
    ArrayInteger64 = b'l',
    ArrayBoolean = b'b',
    String = b'S',
    RawBytes = b'R',
}
