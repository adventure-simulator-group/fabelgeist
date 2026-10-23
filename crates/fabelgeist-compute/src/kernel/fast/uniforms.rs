use crate::prelude::*;
use fabelgeist_gpu::data::gpu::shader::UniformMember;

/// Pack the uniform block from named parameters, by reflected offsets.
///
/// Covers the scalar, vector and matrix kinds a solver passes. Anything else
/// is reported rather than silently skipped -- a uniform quietly left at zero
/// is a wrong simulation, not an error, and those are the expensive ones.
pub(super) fn write_uniform(
    members: &[UniformMember],
    parameters: &PassParameters,
    out: &mut [u8],
) -> Result<()> {
    for member in members {
        let Some(value) = parameters.get(&member.name) else {
            return Err(anyhow!(
                "Kernel: uniform `{}` was not supplied",
                member.name
            ));
        };
        let offset = member.offset as usize;

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
            other => {
                return Err(anyhow!(
                    "Kernel: uniform `{}` is a {:?}, which the cached dispatch path does not pack; \
                     use ComputePass::record for this kernel",
                    member.name,
                    std::mem::discriminant(other)
                ));
            }
        }
    }
    Ok(())
}
