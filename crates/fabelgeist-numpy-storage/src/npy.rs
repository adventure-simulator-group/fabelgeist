//! `.npy` array decoding.

pub use metadata::{
    NpyDimension, NpyElementCount, NpyElementWidth, NpyLayoutError, NpyOccupancy, NpyPayloadLength,
    NpyRank, NpyShape,
};

pub use values::{
    NpyByteState, NpyByteStates, NpyFloatValue, NpyFloatValues, NpyIntegerValue, NpyIntegerValues,
};

use std::path::Path;

use anyhow::{Context, Result, bail};
use burn::tensor::{Device, Int, Tensor, TensorData};

mod metadata;
mod values;

/// One admitted array with immutable shape, element type, and payload metadata.
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::NpyArray;
/// fn replace_shape(array: &mut NpyArray) { array.shape = array.shape.clone(); }
/// ```
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::{Dtype, NpyArray};
/// fn reinterpret_dtype(array: &mut NpyArray) { array.dtype = Dtype::U8; }
/// ```
pub struct NpyArray {
    shape: NpyShape,
    dtype: Dtype,
    bytes: Vec<u8>,
}

/// The NumPy element types this crate decodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dtype {
    F32,
    F64,
    I32,
    I64,
    U8,
    Bool,
}

impl Dtype {
    fn parse(descr: &str) -> Result<Self> {
        Ok(match descr.trim_matches(['\'', '"']) {
            "<f4" | "=f4" | "f4" => Dtype::F32,
            "<f8" | "=f8" | "f8" => Dtype::F64,
            "<i4" | "=i4" | "i4" => Dtype::I32,
            "<i8" | "=i8" | "i8" => Dtype::I64,
            "|u1" | "u1" => Dtype::U8,
            "|b1" | "b1" => Dtype::Bool,
            other => bail!("unsupported NumPy dtype {other:?}"),
        })
    }

    pub fn storage_width(self) -> NpyElementWidth {
        match self {
            Dtype::U8 | Dtype::Bool => NpyElementWidth::Byte,
            Dtype::F32 | Dtype::I32 => NpyElementWidth::Word32,
            Dtype::F64 | Dtype::I64 => NpyElementWidth::Word64,
        }
    }
}

impl NpyArray {
    pub fn shape(&self) -> &NpyShape {
        &self.shape
    }
    pub fn dtype(&self) -> Dtype {
        self.dtype
    }
    pub fn element_count(&self) -> NpyElementCount {
        self.shape.element_count()
    }
    pub fn occupancy(&self) -> NpyOccupancy {
        self.element_count().occupancy()
    }

    /// Element bytes, exactly as stored.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Uploads the array to a device as a float tensor.
    pub fn to_tensor<const D: usize>(
        &self,
        device: &Device,
    ) -> std::result::Result<Tensor<D>, NpyLayoutError> {
        Ok(Tensor::from_data(
            TensorData::new(
                Vec::<f32>::from(NpyFloatValues::from(self)),
                self.shape.tensor_layout::<D>()?.dimensions,
            ),
            device,
        ))
    }

    /// Uploads the array to a device as an integer tensor.
    pub fn to_int_tensor<const D: usize>(
        &self,
        device: &Device,
    ) -> std::result::Result<Tensor<D, Int>, NpyLayoutError> {
        Ok(Tensor::from_data(
            TensorData::new(
                Vec::<i64>::from(NpyIntegerValues::from(self)),
                self.shape.tensor_layout::<D>()?.dimensions,
            ),
            device,
        ))
    }
}

impl NpyArray {
    /// Parses a `.npy` buffer: header dictionary first, then the element payload.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        const MAGIC: &[u8] = b"\x93NUMPY";
        if bytes.len() < 10 || &bytes[..6] != MAGIC {
            bail!("not a .npy array");
        }
        let (header_len, header_start) = if bytes[6] == 1 {
            (u16::from_le_bytes([bytes[8], bytes[9]]) as usize, 10)
        } else {
            (
                u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize,
                12,
            )
        };
        if header_start + header_len > bytes.len() {
            bail!("truncated .npy header");
        }
        let header = std::str::from_utf8(&bytes[header_start..header_start + header_len])
            .context("non-UTF-8 .npy header")?;

        let dtype = Dtype::parse(
            header_value(header, "'descr'").context("missing 'descr' in .npy header")?,
        )?;
        if header_value(header, "'fortran_order'").is_some_and(|v| v.starts_with("True")) {
            bail!("Fortran-ordered .npy arrays are not supported");
        }

        let shape_text = header
            .split_once("'shape'")
            .and_then(|(_, rest)| rest.split_once('('))
            .and_then(|(_, rest)| rest.split_once(')'))
            .map(|(inside, _)| inside)
            .context("missing 'shape' in .npy header")?;
        let dimensions: Vec<NpyDimension> = shape_text
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(|token| {
                token
                    .parse::<usize>()
                    .map(NpyDimension::from)
                    .context("bad .npy shape")
            })
            .collect::<Result<_>>()?;

        let shape = NpyShape::try_from(dimensions)?;
        let length = shape.element_count().payload_length(dtype)?;
        let start = header_start + header_len;
        let payload = length.admit(&bytes[start..])?;

        Ok(NpyArray {
            shape,
            dtype,
            bytes: payload.to_vec(),
        })
    }

    /// Reads a standalone `.npy` file.
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        Self::from_bytes(&bytes).with_context(|| format!("parsing {}", path.display()))
    }
}

/// The scalar value following `key:` in the header dictionary, up to the next
/// comma. Only used for `descr` and `fortran_order`, neither of which is a
/// composite value.
fn header_value<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    let (_, rest) = header.split_once(key)?;
    let rest = rest.trim_start().strip_prefix(':')?;
    Some(rest.split(',').next()?.trim())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn encode(dtype: &str, shape: &str, payload: &[u8]) -> Vec<u8> {
        let header =
            format!("{{'descr': '{dtype}', 'fortran_order': False, 'shape': ({shape}), }}");
        let mut padded = header.into_bytes();
        while (10 + padded.len()) % 64 != 63 {
            padded.push(b' ');
        }
        padded.push(b'\n');
        let mut out = b"\x93NUMPY\x01\x00".to_vec();
        out.extend_from_slice(&(padded.len() as u16).to_le_bytes());
        out.extend_from_slice(&padded);
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn parses_a_float_array() {
        let payload: Vec<u8> = [1.0f32, -2.5, 3.25]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let array = NpyArray::from_bytes(&encode("<f4", "3,", &payload)).unwrap();
        assert_eq!(array.shape().dimensions(), [NpyDimension::from(3)]);
        assert_eq!(array.dtype(), Dtype::F32);
        assert_eq!(
            Vec::<f32>::from(crate::NpyFloatValues::from(&array)),
            [1.0, -2.5, 3.25]
        );
    }

    #[test]
    fn parses_a_two_dimensional_int_array() {
        let payload: Vec<u8> = [1i64, 2, 3, 4]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let array = NpyArray::from_bytes(&encode("<i8", "2, 2", &payload)).unwrap();
        assert_eq!(
            array.shape().dimensions(),
            [NpyDimension::from(2), NpyDimension::from(2)]
        );
        assert_eq!(
            Vec::<i64>::from(crate::NpyIntegerValues::from(&array)),
            [1, 2, 3, 4]
        );
        assert_eq!(
            Vec::<f32>::from(crate::NpyFloatValues::from(&array)),
            [1.0, 2.0, 3.0, 4.0]
        );
    }

    #[test]
    fn parses_a_bool_array() {
        let array = NpyArray::from_bytes(&encode("|b1", "4,", &[1, 0, 1, 1])).unwrap();
        assert_eq!(
            Vec::<bool>::from(crate::NpyByteStates::from(&array)),
            [true, false, true, true]
        );
    }

    #[test]
    fn parses_a_scalar_shape() {
        let array = NpyArray::from_bytes(&encode("<f4", "", &1.5f32.to_le_bytes())).unwrap();
        assert_eq!(array.shape().rank(), NpyRank::from(0));
        assert_eq!(array.element_count(), NpyElementCount::from(1));
        assert_eq!(array.occupancy(), NpyOccupancy::Nonempty);
    }

    #[test]
    fn rejects_fortran_order_and_bad_dtypes() {
        let mut header = b"{'descr': '<f4', 'fortran_order': True, 'shape': (1,), }\n".to_vec();
        let mut bytes = b"\x93NUMPY\x01\x00".to_vec();
        bytes.extend_from_slice(&(header.len() as u16).to_le_bytes());
        bytes.append(&mut header);
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        assert!(NpyArray::from_bytes(&bytes).is_err());

        assert!(NpyArray::from_bytes(&encode("<c8", "1,", &[0; 8])).is_err());
    }

    #[test]
    fn rejects_a_truncated_payload() {
        assert!(NpyArray::from_bytes(&encode("<f4", "4,", &[0; 8])).is_err());
    }
}
