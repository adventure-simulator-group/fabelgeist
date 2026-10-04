//! Ordered decoded NumPy numbers and byte states, retained until SDK conversion.
use super::{Dtype, NpyArray, NpyElementCount, NpyElementOrdinal, NpyElementWidth};
use fabelgeist_storage::StorageByteLength;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NpyFloatValue(f32);
impl From<f32> for NpyFloatValue {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
impl From<NpyFloatValue> for f32 {
    fn from(value: NpyFloatValue) -> Self {
        value.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpyIntegerValue(i64);
impl From<i64> for NpyIntegerValue {
    fn from(value: i64) -> Self {
        Self(value)
    }
}
impl From<NpyIntegerValue> for i64 {
    fn from(value: NpyIntegerValue) -> Self {
        value.0
    }
}
impl TryFrom<NpyIntegerValue> for usize {
    type Error = std::num::TryFromIntError;
    fn try_from(value: NpyIntegerValue) -> Result<Self, std::num::TryFromIntError> {
        Self::try_from(value.0)
    }
}
impl std::fmt::Display for NpyIntegerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpyByteState {
    Zero,
    Nonzero,
}
impl From<u8> for NpyByteState {
    fn from(byte: u8) -> Self {
        if byte == 0 { Self::Zero } else { Self::Nonzero }
    }
}

pub struct NpyFloatValues(Vec<NpyFloatValue>);
impl From<&NpyArray> for NpyFloatValues {
    fn from(array: &NpyArray) -> Self {
        Self(
            StoredValues::from(array)
                .map(StoredValue::floating)
                .collect(),
        )
    }
}
impl NpyFloatValues {
    pub fn values(&self) -> &[NpyFloatValue] {
        &self.0
    }
    pub fn element_count(&self) -> NpyElementCount {
        NpyElementCount::from(self.0.len())
    }
}
impl From<NpyFloatValues> for Vec<f32> {
    fn from(values: NpyFloatValues) -> Self {
        values.0.into_iter().map(f32::from).collect()
    }
}
pub struct NpyIntegerValues(Vec<NpyIntegerValue>);
impl From<&NpyArray> for NpyIntegerValues {
    fn from(array: &NpyArray) -> Self {
        Self(
            StoredValues::from(array)
                .map(StoredValue::integer)
                .collect(),
        )
    }
}
impl NpyIntegerValues {
    pub fn values(&self) -> &[NpyIntegerValue] {
        &self.0
    }
    pub fn element_count(&self) -> NpyElementCount {
        NpyElementCount::from(self.0.len())
    }
    pub fn value(&self, ordinal: NpyElementOrdinal) -> Option<NpyIntegerValue> {
        self.0.get(usize::from(ordinal)).copied()
    }
}
impl From<NpyIntegerValues> for Vec<i64> {
    fn from(values: NpyIntegerValues) -> Self {
        values.0.into_iter().map(i64::from).collect()
    }
}
pub struct NpyByteStates(Vec<NpyByteState>);
impl From<&NpyArray> for NpyByteStates {
    fn from(array: &NpyArray) -> Self {
        Self(
            array
                .bytes
                .iter()
                .copied()
                .map(NpyByteState::from)
                .collect(),
        )
    }
}
impl NpyByteStates {
    pub fn states(&self) -> &[NpyByteState] {
        &self.0
    }
    pub fn byte_length(&self) -> StorageByteLength {
        StorageByteLength::from(self.0.len())
    }
}

enum StoredValue {
    Float32(f32),
    Float64(f64),
    Integer32(i32),
    Integer64(i64),
    Byte(u8),
}
impl StoredValue {
    fn floating(self) -> NpyFloatValue {
        NpyFloatValue(match self {
            Self::Float32(value) => value,
            Self::Float64(value) => value as f32,
            Self::Integer32(value) => value as f32,
            Self::Integer64(value) => value as f32,
            Self::Byte(value) => value as f32,
        })
    }
    fn integer(self) -> NpyIntegerValue {
        NpyIntegerValue(match self {
            Self::Float32(value) => value as i64,
            Self::Float64(value) => value as i64,
            Self::Integer32(value) => value as i64,
            Self::Integer64(value) => value,
            Self::Byte(value) => value as i64,
        })
    }
}
struct StoredValues<'a> {
    dtype: Dtype,
    words: std::slice::ChunksExact<'a, u8>,
}
impl<'a> From<&'a NpyArray> for StoredValues<'a> {
    fn from(array: &'a NpyArray) -> Self {
        let width = match array.dtype.storage_width() {
            NpyElementWidth::Byte => 1,
            NpyElementWidth::Word32 => 4,
            NpyElementWidth::Word64 => 8,
        };
        Self {
            dtype: array.dtype,
            words: array.bytes.chunks_exact(width),
        }
    }
}
impl Iterator for StoredValues<'_> {
    type Item = StoredValue;
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.words.size_hint()
    }
    fn next(&mut self) -> Option<StoredValue> {
        let word = self.words.next()?;
        // Array admission seals the dtype and exactly sized payload; the SDK
        // iterator supplies words at that dtype's fixed width.
        Some(match self.dtype {
            Dtype::F32 => StoredValue::Float32(f32::from_le_bytes(
                word.try_into().expect("admitted f32 word"),
            )),
            Dtype::F64 => StoredValue::Float64(f64::from_le_bytes(
                word.try_into().expect("admitted f64 word"),
            )),
            Dtype::I32 => StoredValue::Integer32(i32::from_le_bytes(
                word.try_into().expect("admitted i32 word"),
            )),
            Dtype::I64 => StoredValue::Integer64(i64::from_le_bytes(
                word.try_into().expect("admitted i64 word"),
            )),
            Dtype::U8 | Dtype::Bool => StoredValue::Byte(word[0]),
        })
    }
}
impl ExactSizeIterator for StoredValues<'_> {}

#[cfg(test)]
mod tests;
