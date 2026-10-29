//! Ordered decoded NumPy numbers and byte states, retained until SDK conversion.
use derive_more::Into;

use super::{Dtype, NpyArray, NpyElementCount};

/// Ordered floating projections of an array's current payload and dtype.
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::NpyFloat32Values;
/// let values: NpyFloat32Values = vec![1.0f32];
/// ```
///
/// ```no_run
/// use fabelgeist_numpy_storage::{NpyArray, NpyFloat32Values, NpyIntegerValues};
/// fn decode(array: &NpyArray) {
///     let floats = NpyFloat32Values::from(array);
///     let integers = NpyIntegerValues::from(array);
///     let native_floats = Vec::<f32>::from(floats);
///     let native_integers = Vec::<i64>::from(integers);
/// }
/// ```
#[derive(Into)]
pub struct NpyFloat32Values(Vec<f32>);

/// Ordered integer conversions of an array's current payload and dtype.
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::{NpyFloat32Values, NpyIntegerValues};
/// fn decode(values: NpyIntegerValues) {
///     let floats: NpyFloat32Values = values;
/// }
/// ```
#[derive(Into)]
pub struct NpyIntegerValues(Vec<i64>);

/// Ordered zero/nonzero classifications of every payload byte.
#[derive(Into)]
pub struct NpyByteStates(Vec<bool>);

enum StoredValue {
    Float32(f32),
    Float64(f64),
    Integer32(i32),
    Integer64(i64),
    Byte(u8),
}

enum StoredValues<'a> {
    Float32(std::slice::Iter<'a, [u8; 4]>),
    Float64(std::slice::Iter<'a, [u8; 8]>),
    Integer32(std::slice::Iter<'a, [u8; 4]>),
    Integer64(std::slice::Iter<'a, [u8; 8]>),
    Byte(std::slice::Iter<'a, u8>),
}

impl From<&NpyArray> for NpyFloat32Values {
    fn from(array: &NpyArray) -> Self {
        Self(
            StoredValues::from(array)
                .map(StoredValue::floating)
                .collect(),
        )
    }
}

impl NpyFloat32Values {
    pub fn element_count(&self) -> NpyElementCount {
        NpyElementCount::from(self.0.len())
    }

    pub fn values(&self) -> &[f32] {
        &self.0
    }
}

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
    pub fn element_count(&self) -> NpyElementCount {
        NpyElementCount::from(self.0.len())
    }

    pub fn values(&self) -> &[i64] {
        &self.0
    }
}

impl From<&NpyArray> for NpyByteStates {
    fn from(array: &NpyArray) -> Self {
        Self(array.bytes.iter().map(|byte| *byte != 0).collect())
    }
}

impl NpyByteStates {
    pub fn states(&self) -> &[bool] {
        &self.0
    }
}

impl StoredValue {
    fn floating(self) -> f32 {
        match self {
            Self::Float32(value) => value,
            Self::Float64(value) => value as f32,
            Self::Integer32(value) => value as f32,
            Self::Integer64(value) => value as f32,
            Self::Byte(value) => value as f32,
        }
    }
    fn integer(self) -> i64 {
        match self {
            Self::Float32(value) => value as i64,
            Self::Float64(value) => value as i64,
            Self::Integer32(value) => value as i64,
            Self::Integer64(value) => value,
            Self::Byte(value) => value as i64,
        }
    }
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

#[cfg(test)]
mod tests;
