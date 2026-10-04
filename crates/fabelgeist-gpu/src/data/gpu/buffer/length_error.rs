//! A logical result must fit within its retained native allocation.

use super::BufferByteLength;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("logical buffer length {requested} exceeds its {available}-byte allocation")]
pub struct BufferLengthError {
    pub requested: BufferByteLength,
    pub available: BufferByteLength,
}
