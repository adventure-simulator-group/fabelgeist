//! Admission of the supported external NumPy header dictionary and vocabulary.
use super::RejectedNpyValue;
use super::{Dtype, NpyDecodeError, NpyDimension, NpyElementWidth, NpyHeaderField, NpyShape};
use fabelgeist_storage::StorageView;

pub(super) struct NpyHeader<'a>(&'a str);
impl<'a> TryFrom<&'a StorageView<'_>> for NpyHeader<'a> {
    type Error = NpyDecodeError;
    fn try_from(view: &'a StorageView<'_>) -> Result<Self, NpyDecodeError> {
        Ok(Self(
            std::str::from_utf8(view.as_ref()).map_err(NpyDecodeError::HeaderEncoding)?,
        ))
    }
}
#[derive(Clone, Copy)]
pub(super) struct HeaderValue<'a>(&'a str);
#[derive(Clone, Copy)]
enum ScalarField {
    Dtype,
    FortranOrder,
}
impl NpyHeader<'_> {
    /// Scalar fields end at the next comma; composite shapes use their own grammar.
    fn scalar(&self, field: ScalarField) -> Option<HeaderValue<'_>> {
        let key = match field {
            ScalarField::Dtype => "'descr'",
            ScalarField::FortranOrder => "'fortran_order'",
        };
        let (_, rest) = self.0.split_once(key)?;
        let rest = rest.trim_start().strip_prefix(':')?;
        Some(HeaderValue(rest.split(',').next()?.trim()))
    }
    pub(super) fn dtype(&self) -> Result<Dtype, NpyDecodeError> {
        Dtype::admit(
            self.scalar(ScalarField::Dtype)
                .ok_or(NpyDecodeError::MissingField(NpyHeaderField::Dtype))?,
        )
    }
    pub(super) fn admit_order(&self) -> Result<(), NpyDecodeError> {
        if let Some(order) = self.scalar(ScalarField::FortranOrder)
            && order.0.starts_with("True")
        {
            return Err(NpyDecodeError::FortranOrder);
        }
        Ok(())
    }
    pub(super) fn shape(&self) -> Result<NpyShape, NpyDecodeError> {
        let (_, shape_value) = self
            .0
            .split_once("'shape'")
            .ok_or(NpyDecodeError::MissingField(NpyHeaderField::Shape))?;
        let (_, shape_body) = shape_value
            .split_once('(')
            .ok_or(NpyDecodeError::MissingField(NpyHeaderField::Shape))?;
        let (shape_text, _) = shape_body
            .split_once(')')
            .ok_or(NpyDecodeError::MissingField(NpyHeaderField::Shape))?;
        let mut dimensions = Vec::new();
        for token in shape_text.split(',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            let extent = token.parse::<usize>().map_err(
                |source: std::num::ParseIntError| -> NpyDecodeError {
                    NpyDecodeError::ShapeDimension {
                        input: RejectedNpyValue::from(token),
                        source,
                    }
                },
            )?;
            dimensions.push(NpyDimension::from(extent));
        }
        NpyShape::try_from(dimensions)
    }
}
impl Dtype {
    fn admit(descr: HeaderValue<'_>) -> Result<Self, NpyDecodeError> {
        Ok(match descr.0.trim_matches(['\'', '"']) {
            "<f4" | "=f4" | "f4" => Self::F32,
            "<f8" | "=f8" | "f8" => Self::F64,
            "<i4" | "=i4" | "i4" => Self::I32,
            "<i8" | "=i8" | "i8" => Self::I64,
            "|u1" | "u1" => Self::U8,
            "|b1" | "b1" => Self::Bool,
            other => return Err(NpyDecodeError::Dtype(RejectedNpyValue::from(other))),
        })
    }
    pub fn storage_width(self) -> NpyElementWidth {
        match self {
            Self::U8 | Self::Bool => NpyElementWidth::Byte,
            Self::F32 | Self::I32 => NpyElementWidth::Word32,
            Self::F64 | Self::I64 => NpyElementWidth::Word64,
        }
    }
}
