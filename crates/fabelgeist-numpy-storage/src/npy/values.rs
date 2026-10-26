//! Ordered decoded NumPy numbers and byte states, retained until SDK conversion.
use super::{Dtype, NpyArray};

/// A decoded storage number projected to f32 precision.
/// Integer conversion uses the original stored number, independently of this
/// projection.
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
/// A decoded storage number converted directly to i64 with Rust cast semantics.
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
impl std::fmt::Display for NpyIntegerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
/// Classification of one payload byte, independently of logical element width.
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

/// Ordered floating projections of an array's current payload and dtype.
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::NpyFloatValues;
/// let values: NpyFloatValues = vec![1.0f32];
/// ```
///
/// ```no_run
/// use fabelgeist_numpy_storage::{NpyArray, NpyFloatValues, NpyIntegerValues};
/// fn decode(array: &NpyArray) {
///     let floats = NpyFloatValues::from(array);
///     let integers = NpyIntegerValues::from(array);
///     let native_floats = Vec::<f32>::from(floats);
///     let native_integers = Vec::<i64>::from(integers);
/// }
/// ```
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
}
impl From<NpyFloatValues> for Vec<f32> {
    fn from(values: NpyFloatValues) -> Self {
        values.0.into_iter().map(f32::from).collect()
    }
}
/// Ordered integer conversions of an array's current payload and dtype.
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::{NpyFloatValues, NpyIntegerValues};
/// fn decode(values: NpyIntegerValues) {
///     let floats: NpyFloatValues = values;
/// }
/// ```
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
}
impl From<NpyIntegerValues> for Vec<i64> {
    fn from(values: NpyIntegerValues) -> Self {
        values.0.into_iter().map(i64::from).collect()
    }
}
/// Ordered zero/nonzero classifications of every payload byte.
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
enum StoredValues<'a> {
    Float32(std::slice::Iter<'a, [u8; 4]>),
    Float64(std::slice::Iter<'a, [u8; 8]>),
    Integer32(std::slice::Iter<'a, [u8; 4]>),
    Integer64(std::slice::Iter<'a, [u8; 8]>),
    Byte(std::slice::Iter<'a, u8>),
}
impl<'a> From<&'a NpyArray> for StoredValues<'a> {
    fn from(array: &'a NpyArray) -> Self {
        // The current dtype selects native fixed-width words. As in
        // chunks_exact, an incomplete final word is not decoded.
        match array.dtype {
            Dtype::F32 => Self::Float32(array.bytes.as_chunks::<4>().0.iter()),
            Dtype::F64 => Self::Float64(array.bytes.as_chunks::<8>().0.iter()),
            Dtype::I32 => Self::Integer32(array.bytes.as_chunks::<4>().0.iter()),
            Dtype::I64 => Self::Integer64(array.bytes.as_chunks::<8>().0.iter()),
            Dtype::U8 | Dtype::Bool => Self::Byte(array.bytes.iter()),
        }
    }
}
impl Iterator for StoredValues<'_> {
    type Item = StoredValue;
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Float32(words) | Self::Integer32(words) => words.size_hint(),
            Self::Float64(words) | Self::Integer64(words) => words.size_hint(),
            Self::Byte(bytes) => bytes.size_hint(),
        }
    }
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Float32(words) => words
                .next()
                .map(|word| StoredValue::Float32(f32::from_le_bytes(*word))),
            Self::Float64(words) => words
                .next()
                .map(|word| StoredValue::Float64(f64::from_le_bytes(*word))),
            Self::Integer32(words) => words
                .next()
                .map(|word| StoredValue::Integer32(i32::from_le_bytes(*word))),
            Self::Integer64(words) => words
                .next()
                .map(|word| StoredValue::Integer64(i64::from_le_bytes(*word))),
            Self::Byte(bytes) => bytes.next().copied().map(StoredValue::Byte),
        }
    }
}
impl ExactSizeIterator for StoredValues<'_> {}

impl From<NpyByteStates> for Vec<bool> {
    fn from(states: NpyByteStates) -> Self {
        states
            .0
            .into_iter()
            .map(|state| state == NpyByteState::Nonzero)
            .collect()
    }
}

#[cfg(test)]
mod tests;
