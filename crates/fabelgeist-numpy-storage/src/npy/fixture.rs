//! Shared authored NumPy records, including intentionally rejected header states.
use super::{Dtype, NpyDimension};
use fabelgeist_fs::FileContents;
use fabelgeist_storage::{StorageByteLength, StorageView};

#[derive(Clone, Copy)]
pub(crate) enum FixtureDtype {
    Supported(Dtype),
    Complex64,
}
impl From<Dtype> for FixtureDtype {
    fn from(dtype: Dtype) -> Self {
        Self::Supported(dtype)
    }
}
impl std::fmt::Display for FixtureDtype {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Supported(Dtype::F32) => "<f4",
            Self::Supported(Dtype::F64) => "<f8",
            Self::Supported(Dtype::I32) => "<i4",
            Self::Supported(Dtype::I64) => "<i8",
            Self::Supported(Dtype::U8) => "|u1",
            Self::Supported(Dtype::Bool) => "|b1",
            Self::Complex64 => "<c8",
        })
    }
}
pub(crate) enum FixtureShape {
    Tuple(Vec<NpyDimension>),
    BareDimension(NpyDimension),
    InvalidDimension,
}
impl From<Vec<NpyDimension>> for FixtureShape {
    fn from(dimensions: Vec<NpyDimension>) -> Self {
        Self::Tuple(dimensions)
    }
}
impl std::fmt::Display for FixtureShape {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tuple(dimensions) => {
                for (slot, dimension) in dimensions.iter().copied().enumerate() {
                    if slot != 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{}", usize::from(dimension))?;
                }
                if dimensions.len() == 1 {
                    formatter.write_str(",")?;
                }
                Ok(())
            }
            Self::BareDimension(dimension) => write!(formatter, "{}", usize::from(*dimension)),
            Self::InvalidDimension => formatter.write_str("negative"),
        }
    }
}
#[derive(Debug)]
pub(crate) struct FixtureEncodeError {
    pub length: StorageByteLength,
    pub source: std::num::TryFromIntError,
}
impl std::fmt::Display for FixtureEncodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "NumPy v1 fixture header {}: {}",
            self.length, self.source
        )
    }
}
impl std::error::Error for FixtureEncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
pub(crate) struct NpyFixture(FileContents);
impl NpyFixture {
    pub(crate) fn from_parts(
        dtype: FixtureDtype,
        shape: FixtureShape,
        payload: FileContents,
    ) -> Result<Self, FixtureEncodeError> {
        let header =
            format!("{{'descr': '{dtype}', 'fortran_order': False, 'shape': ({shape}), }}");
        let mut padded = header.into_bytes();
        while (10 + padded.len()) % 64 != 63 {
            padded.push(b' ');
        }
        padded.push(b'\n');
        let length = u16::try_from(padded.len()).map_err(
            |source: std::num::TryFromIntError| -> FixtureEncodeError {
                FixtureEncodeError {
                    length: StorageByteLength::from(padded.len()),
                    source,
                }
            },
        )?;
        let mut out = b"\x93NUMPY\x01\x00".to_vec();
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&padded);
        out.extend_from_slice(payload.as_ref());
        Ok(Self(FileContents::from(out)))
    }
    pub(crate) fn view(&self) -> StorageView<'_> {
        StorageView::from(self.0.as_ref())
    }
}
impl From<NpyFixture> for FileContents {
    fn from(fixture: NpyFixture) -> Self {
        fixture.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_fixtures_keep_the_frozen_header_padding_and_payload() {
        // CRCs were captured from the previous encoder, before this migration.
        let mut floats = Vec::new();
        for value in [1.0f32, -2.5, 3.25] {
            floats.extend_from_slice(&value.to_le_bytes());
        }
        let mut integers = Vec::new();
        for value in [1i64, 2, 3, 4] {
            integers.extend_from_slice(&value.to_le_bytes());
        }
        for (dtype, shape, payload, length, checksum) in [
            (
                Dtype::F32.into(),
                FixtureShape::from(vec![NpyDimension::from(3)]),
                FileContents::from(floats),
                StorageByteLength::from(140u64),
                0x2dc8_0182,
            ),
            (
                Dtype::I64.into(),
                FixtureShape::from(vec![NpyDimension::from(2), NpyDimension::from(2)]),
                FileContents::from(integers),
                StorageByteLength::from(160u64),
                0x5b4b_44a0,
            ),
            (
                Dtype::Bool.into(),
                FixtureShape::from(vec![NpyDimension::from(4)]),
                FileContents::from(vec![1, 0, 1, 1]),
                StorageByteLength::from(132u64),
                0x3a54_f482,
            ),
            (
                Dtype::F32.into(),
                FixtureShape::from(vec![]),
                FileContents::from(1.5f32.to_le_bytes().to_vec()),
                StorageByteLength::from(132u64),
                0x5de6_1325,
            ),
            (
                FixtureDtype::Complex64,
                FixtureShape::InvalidDimension,
                FileContents::default(),
                StorageByteLength::from(128u64),
                0x664a_a08d,
            ),
        ] {
            let fixture = NpyFixture::from_parts(dtype, shape, payload).unwrap();
            assert_eq!(fixture.view().length(), length);
            assert_eq!(crc32fast::hash(fixture.view().as_ref()), checksum);
        }
    }

    #[test]
    fn oversized_fixture_headers_retain_length_and_native_conversion_cause() {
        let failure = match NpyFixture::from_parts(
            Dtype::F32.into(),
            FixtureShape::from(vec![NpyDimension::from(usize::MAX); 7000]),
            FileContents::default(),
        ) {
            Err(error) => error,
            Ok(_) => panic!("oversized v1 header admitted"),
        };
        assert!(failure.length > StorageByteLength::from(u64::from(u16::MAX)));
        assert!(
            std::error::Error::source(&failure)
                .unwrap()
                .is::<std::num::TryFromIntError>()
        );
        assert!(failure.to_string().contains("NumPy v1 fixture header"));
    }
}
