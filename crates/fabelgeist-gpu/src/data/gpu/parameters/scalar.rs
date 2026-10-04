//! Scalar representations admitted for the shader's uniform ABI.

/// A numeric pass value, rounded to binary32 only when packed.
///
/// NaN, infinity, signed zero and overflow retain the original cast policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UniformNumber(f64);

/// A uniform's unsigned word; signed admission preserves the original cast.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UniformUnsigned(u32);

impl From<f32> for UniformNumber {
    fn from(value: f32) -> Self {
        Self(value as f64)
    }
}
impl From<f64> for UniformNumber {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
impl From<u32> for UniformUnsigned {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<i32> for UniformUnsigned {
    fn from(value: i32) -> Self {
        Self(value as u32)
    }
}
impl UniformNumber {
    pub(super) fn encode(self) -> [u8; 4] {
        (self.0 as f32).to_le_bytes()
    }
}
impl UniformUnsigned {
    pub(super) fn encode(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}
