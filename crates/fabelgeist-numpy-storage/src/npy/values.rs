//! Ordered decoded NumPy numbers and byte states, retained until SDK conversion.
use derive_more::{Display, From, Into};

use super::decode_error::{NpyDecodeError, Result};
use super::{Dtype, NpyArray};

use decode::StoredValues;

mod decode;

/// A decoded storage number projected to f32 precision.
/// Integer conversion uses the original stored number, independently of this
/// projection.
#[derive(Clone, Copy, Debug, From, Into, PartialEq)]
pub struct NpyFloatValue(f32);

/// A decoded storage number converted directly to i64 with Rust cast semantics.
#[derive(Clone, Copy, Debug, Display, From, Into, PartialEq, Eq)]
pub struct NpyIntegerValue(i64);

/// Ordered floating projections of an array's current payload and dtype.
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::NpyFloatValues;
/// let values: NpyFloatValues = vec![1.0f32];
/// ```
///
/// ```no_run
/// use fabelgeist_numpy_storage::{NpyArray, NpyDecodeError, NpyFloatValues, NpyIntegerValues};
/// fn decode(array: &NpyArray) -> Result<(), NpyDecodeError> {
///     let floats = NpyFloatValues::try_from(array)?;
///     let integers = NpyIntegerValues::try_from(array)?;
///     let native_floats = Vec::<f32>::from(floats);
///     let native_integers = Vec::<i64>::from(integers);
///     Ok(())
/// }
/// ```
pub struct NpyFloatValues(Vec<NpyFloatValue>);

/// Ordered integer conversions of an array's current payload and dtype.
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::{NpyFloatValues, NpyIntegerValues};
/// fn decode(values: NpyIntegerValues) {
///     let floats: NpyFloatValues = values;
/// }
/// ```
pub struct NpyIntegerValues(Vec<NpyIntegerValue>);

/// Ordered zero/nonzero classifications of every payload byte.
pub struct NpyByteStates(Vec<NpyByteState>);

/// Classification of one payload byte, independently of logical element width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpyByteState {
    Zero,
    Nonzero,
}

enum StoredValue {
    Float32(f32),
    Float64(f64),
    Integer32(i32),
    Integer64(i64),
    Byte(u8),
}

impl From<u8> for NpyByteState {
    fn from(byte: u8) -> Self {
        if byte == 0 { Self::Zero } else { Self::Nonzero }
    }
}

impl TryFrom<&NpyArray> for NpyFloatValues {
    type Error = NpyDecodeError;

    fn try_from(array: &NpyArray) -> Result<Self> {
        Ok(Self(
            StoredValues::try_from(array)?
                .map(|value| value.map(StoredValue::floating))
                .collect::<std::io::Result<_>>()?,
        ))
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

impl TryFrom<&NpyArray> for NpyIntegerValues {
    type Error = NpyDecodeError;

    fn try_from(array: &NpyArray) -> Result<Self> {
        Ok(Self(
            StoredValues::try_from(array)?
                .map(|value| value.map(StoredValue::integer))
                .collect::<std::io::Result<_>>()?,
        ))
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
