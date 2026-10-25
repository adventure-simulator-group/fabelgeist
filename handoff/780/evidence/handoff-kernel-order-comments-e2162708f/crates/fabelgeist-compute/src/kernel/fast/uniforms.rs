use crate::prelude::*;
use fabelgeist_gpu::data::gpu::shader::UniformMember;

/// Pack supported provided variants at the reflected member offsets.
///
/// Missing parameters and unsupported variants return typed errors. Like the
/// static packer, this skips an entire provided value if it does not fit in the
/// block, leaving that destination unchanged. The supplied variant determines
/// the representation; this does not check the reflected member's type or span.
pub(super) fn write_uniform(
    members: &[UniformMember],
    parameters: &PassParameters,
    out: &mut [u8],
) -> Result<()> {
    for member in members {
        let Some(value) = parameters.get(&member.name) else {
            return Err(ComputePassError::MissingParameter {
                name: member.name.clone(),
                kind: ComputeBindingKind::Uniform,
            }
            .into());
        };
        let offset = usize::try_from(u64::from(member.offset))
            .map_err(ComputePassError::UniformHostAddress)?;

        let mut put = |values: &[f32]| {
            let end = offset + values.len() * 4;
            if end <= out.len() {
                for (index, value) in values.iter().enumerate() {
                    out[offset + index * 4..offset + index * 4 + 4]
                        .copy_from_slice(&value.to_le_bytes());
                }
            }
        };

        match value {
            PassParameter::Number(n) => put(&[*n as f32]),
            PassParameter::Unsigned(u) => {
                if offset + 4 <= out.len() {
                    out[offset..offset + 4].copy_from_slice(&u.to_le_bytes());
                }
            }
            PassParameter::Vec2(v) => put(&[v.x, v.y]),
            PassParameter::Vec3(v) => put(&[v.x, v.y, v.z]),
            PassParameter::Vec4(v) => put(&[v.x, v.y, v.z, v.w]),
            PassParameter::Mat4(m) => {
                let mut values = Vec::with_capacity(16);
                for column in 0..4 {
                    for row in 0..4 {
                        values.push(m.columns[column][row]);
                    }
                }
                put(&values);
            }
            _ => {
                return Err(ComputePassError::UnsupportedUniform {
                    name: member.name.clone(),
                }
                .into());
            }
        }
    }
    Ok(())
}
