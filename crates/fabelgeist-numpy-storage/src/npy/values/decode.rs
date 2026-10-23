//! NumPy's primitive reader at the fixed-width wire boundary.

use std::io;
use std::slice;

use npyz::{Deserialize, TypeRead};

use super::super::decode_error::{NpyDecodeError, Result};
use super::{Dtype, NpyArray, StoredValue};

pub(super) enum StoredValues<'a> {
    Float32 {
        words: slice::Iter<'a, [u8; 4]>,
        reader: <f32 as Deserialize>::TypeReader,
    },
    Float64 {
        words: slice::Iter<'a, [u8; 8]>,
        reader: <f64 as Deserialize>::TypeReader,
    },
    Integer32 {
        words: slice::Iter<'a, [u8; 4]>,
        reader: <i32 as Deserialize>::TypeReader,
    },
    Integer64 {
        words: slice::Iter<'a, [u8; 8]>,
        reader: <i64 as Deserialize>::TypeReader,
    },
    Byte(slice::Iter<'a, u8>),
}

impl<'a> TryFrom<&'a NpyArray> for StoredValues<'a> {
    type Error = NpyDecodeError;

    fn try_from(array: &'a NpyArray) -> Result<Self> {
        // Decode complete words from the current stored kind. The independent
        // byte-state view still classifies every byte, including bool magnitudes.
        Ok(match array.dtype {
            Dtype::F32 => Self::Float32 {
                words: array.bytes.as_chunks::<4>().0.iter(),
                reader: reader::<f32>("<f4")?,
            },
            Dtype::F64 => Self::Float64 {
                words: array.bytes.as_chunks::<8>().0.iter(),
                reader: reader::<f64>("<f8")?,
            },
            Dtype::I32 => Self::Integer32 {
                words: array.bytes.as_chunks::<4>().0.iter(),
                reader: reader::<i32>("<i4")?,
            },
            Dtype::I64 => Self::Integer64 {
                words: array.bytes.as_chunks::<8>().0.iter(),
                reader: reader::<i64>("<i8")?,
            },
            Dtype::U8 | Dtype::Bool => Self::Byte(array.bytes.iter()),
        })
    }
}

impl Iterator for StoredValues<'_> {
    type Item = io::Result<StoredValue>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Float32 { words, reader } => words
                .next()
                .map(|word| reader.read_one(word.as_slice()).map(StoredValue::Float32)),
            Self::Float64 { words, reader } => words
                .next()
                .map(|word| reader.read_one(word.as_slice()).map(StoredValue::Float64)),
            Self::Integer32 { words, reader } => words
                .next()
                .map(|word| reader.read_one(word.as_slice()).map(StoredValue::Integer32)),
            Self::Integer64 { words, reader } => words
                .next()
                .map(|word| reader.read_one(word.as_slice()).map(StoredValue::Integer64)),
            Self::Byte(bytes) => bytes.next().copied().map(StoredValue::Byte).map(Ok),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Float32 { words, .. } | Self::Integer32 { words, .. } => words.size_hint(),
            Self::Float64 { words, .. } | Self::Integer64 { words, .. } => words.size_hint(),
            Self::Byte(bytes) => bytes.size_hint(),
        }
    }
}

impl ExactSizeIterator for StoredValues<'_> {}

fn reader<T: Deserialize>(descriptor: &str) -> Result<T::TypeReader> {
    let dtype = npyz::DType::new_scalar(descriptor.parse()?);
    Ok(T::reader(&dtype)?)
}
