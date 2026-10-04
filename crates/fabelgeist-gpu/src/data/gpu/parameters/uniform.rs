//! Reflection-driven packing at the little-endian GPU serialization boundary.
use super::{PassParameter, PassParameterName, PassParameters};
use crate::data::{ReadbackError, UniformMember};

/// The general pass skips unavailable values in its initially zeroed payload.
/// The cached path requires every member and supports its established subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniformPackingPolicy {
    General,
    Cached,
}

#[derive(Debug)]
pub enum UniformPackingError {
    Missing {
        member: PassParameterName,
    },
    Unsupported {
        member: PassParameterName,
        value: Box<PassParameter>,
    },
    HostLayout {
        member: PassParameterName,
        cause: ReadbackError,
    },
}
impl std::fmt::Display for UniformPackingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { member } => {
                write!(formatter, "Kernel: uniform `{member}` was not supplied")
            }
            Self::Unsupported { member, value } => write!(
                formatter,
                "Kernel: uniform `{member}` is a {:?}, which the cached dispatch path does not pack; use ComputePass::record for this kernel",
                std::mem::discriminant(value.as_ref())
            ),
            Self::HostLayout { member, cause } => write!(
                formatter,
                "Uniform `{member}` cannot address host bytes: {cause}"
            ),
        }
    }
}
impl std::error::Error for UniformPackingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::HostLayout { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

/// A borrowed uniform payload. It does not confer shader type compatibility.
pub struct UniformBytes<'a>(&'a mut [u8]);
impl<'a> From<&'a mut [u8]> for UniformBytes<'a> {
    fn from(bytes: &'a mut [u8]) -> Self {
        Self(bytes)
    }
}
impl UniformBytes<'_> {
    /// Upload the serialized payload at its nominal arena address.
    pub fn upload(
        &self,
        context: &crate::globals::WgpuContext,
        buffer: &wgpu::Buffer,
        offset: crate::data::BufferByteOffset,
    ) {
        context
            .queue
            .write_buffer(buffer, u64::from(offset), self.0);
    }

    pub fn pack(
        &mut self,
        members: &[UniformMember],
        parameters: &PassParameters,
        policy: UniformPackingPolicy,
    ) -> Result<(), UniformPackingError> {
        for member in members {
            let Some(value) = parameters.get(&member.name) else {
                match policy {
                    UniformPackingPolicy::General => continue,
                    UniformPackingPolicy::Cached => {
                        return Err(UniformPackingError::Missing {
                            member: member.name.clone(),
                        });
                    }
                }
            };
            self.write_member(member, value, policy)?;
        }
        Ok(())
    }

    fn write_member(
        &mut self,
        member: &UniformMember,
        value: &PassParameter,
        policy: UniformPackingPolicy,
    ) -> Result<(), UniformPackingError> {
        let start = usize::try_from(member.offset).map_err(
            |cause: ReadbackError| -> UniformPackingError {
                UniformPackingError::HostLayout {
                    member: member.name.clone(),
                    cause,
                }
            },
        )?;
        // This callback serializes native component words directly to the
        // payload. Scalars and vectors retain the allocation-free cached path.
        let mut put = |values: &[f32]| {
            if let Some(end) = start.checked_add(values.len() * 4)
                && let Some(out) = self.0.get_mut(start..end)
            {
                for (word, value) in out.as_chunks_mut::<4>().0.iter_mut().zip(values) {
                    word.copy_from_slice(&value.to_le_bytes());
                }
            }
        };
        match value {
            PassParameter::Number(n) => {
                if let Some(out) = self.0.get_mut(start..start + 4) {
                    out.copy_from_slice(&n.encode());
                }
            }
            PassParameter::Unsigned(u) => {
                if let Some(out) = self.0.get_mut(start..start + 4) {
                    out.copy_from_slice(&u.encode());
                }
            }
            PassParameter::Vec2(v) => put(&[v.x, v.y]),
            PassParameter::Vec3(v) => put(&[v.x, v.y, v.z]),
            PassParameter::Vec4(v) => put(&[v.x, v.y, v.z, v.w]),
            PassParameter::Mat4(m) => put(m.columns.as_flattened()),
            PassParameter::Mat2(m) if policy == UniformPackingPolicy::General => {
                put(m.columns.as_flattened())
            }
            PassParameter::Mat3(m) if policy == UniformPackingPolicy::General => {
                let size = usize::try_from(member.size).map_err(
                    |cause: ReadbackError| -> UniformPackingError {
                        UniformPackingError::HostLayout {
                            member: member.name.clone(),
                            cause,
                        }
                    },
                )?;
                if let Some(end) = start.checked_add(size)
                    && let Some(out) = self.0.get_mut(start..end)
                {
                    let stride = size / 3;
                    for (column, values) in m.columns.iter().enumerate() {
                        for (row, value) in values.iter().enumerate() {
                            let address = column * stride + row * 4;
                            if let Some(word) = out.get_mut(address..address + 4) {
                                word.copy_from_slice(&value.to_le_bytes());
                            }
                        }
                    }
                }
            }
            PassParameter::Transform(t) if policy == UniformPackingPolicy::General => {
                put(t.to_mat4().columns.as_flattened())
            }
            _ => {
                return match policy {
                    UniformPackingPolicy::General => Ok(()),
                    UniformPackingPolicy::Cached => Err(UniformPackingError::Unsupported {
                        member: member.name.clone(),
                        value: Box::new(value.clone()),
                    }),
                };
            }
        }
        Ok(())
    }
}
