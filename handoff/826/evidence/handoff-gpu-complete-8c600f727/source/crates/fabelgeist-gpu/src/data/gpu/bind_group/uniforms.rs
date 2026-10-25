use crate::data::{
    BindGroupReflection, BufferByteLength, BufferByteOffset, ComputePassError, PassParameter,
    PassParameters, UniformMember,
};

type UniformPackingResult<T> = Result<T, ComputePassError>;

/// Pack provided value variants at Naga's reflected byte offsets.
/// The block starts zero initialized; missing and unsupported values are skipped.
pub(super) fn pack_uniforms(
    reflection: &BindGroupReflection,
    parameters: &PassParameters,
) -> UniformPackingResult<Vec<u8>> {
    let size = usize::try_from(u64::from(reflection.uniform_buffer_size))
        .map_err(ComputePassError::UniformHostAddress)?;
    let mut data = vec![0u8; size];
    for member in &reflection.uniform_members {
        if let Some(value) = parameters.get(&member.name) {
            write_value(&mut data, member, value)?;
        }
    }
    Ok(data)
}

fn write_value(
    data: &mut [u8],
    member: &UniformMember,
    value: &PassParameter,
) -> UniformPackingResult<()> {
    // Host slice addresses and component strides are native byte-encoding ports.
    let offset =
        usize::try_from(u64::from(member.offset)).map_err(ComputePassError::UniformHostAddress)?;
    match value {
        PassParameter::Number(value) => write_bytes(data, offset, &(*value as f32).to_le_bytes()),
        PassParameter::Unsigned(value) => write_bytes(data, offset, &value.to_le_bytes()),
        PassParameter::Vec2(value) => write_vector(data, offset, [value.x, value.y]),
        PassParameter::Vec3(value) => write_vector(data, offset, [value.x, value.y, value.z]),
        PassParameter::Vec4(value) => {
            write_vector(data, offset, [value.x, value.y, value.z, value.w])
        }
        PassParameter::Mat2(value) => write_matrix(data, offset, &value.columns, 16, 8)?,
        PassParameter::Mat3(value) => {
            let span = usize::try_from(u64::from(member.size))
                .map_err(ComputePassError::UniformHostAddress)?;
            write_matrix(data, offset, &value.columns, span, span / 3)?;
        }
        PassParameter::Mat4(value) => write_matrix(data, offset, &value.columns, 64, 16)?,
        PassParameter::Transform(value) => {
            write_matrix(data, offset, &value.to_mat4().columns, 64, 16)?;
        }
        _ => {}
    }
    Ok(())
}

fn write_bytes(data: &mut [u8], offset: usize, bytes: &[u8]) {
    if let Some(end) = offset.checked_add(bytes.len())
        && let Some(target) = data.get_mut(offset..end)
    {
        target.copy_from_slice(bytes);
    }
}

fn write_vector<const N: usize>(data: &mut [u8], offset: usize, values: [f32; N]) {
    if let Some(end) = N
        .checked_mul(size_of::<f32>())
        .and_then(|span| offset.checked_add(span))
        && end <= data.len()
    {
        for (index, value) in values.iter().enumerate() {
            write_bytes(
                data,
                offset + index * size_of::<f32>(),
                &value.to_le_bytes(),
            );
        }
    }
}

// Matrix encoding keeps the existing spans and strides in bytes. Mat3 uses
// reflected member span/3; the other provided variants retain fixed layouts.
fn write_matrix<const N: usize>(
    data: &mut [u8],
    start: usize,
    columns: &[[f32; N]; N],
    span: usize,
    column_stride: usize,
) -> UniformPackingResult<()> {
    let end = start
        .checked_add(span)
        .ok_or(ComputePassError::UniformPackingOverflow)?;
    if end > data.len() {
        return Ok(());
    }
    for (column, values) in columns.iter().enumerate() {
        for (row, value) in values.iter().enumerate() {
            let offset = column
                .checked_mul(column_stride)
                .and_then(|offset| start.checked_add(offset))
                .and_then(|offset| {
                    row.checked_mul(size_of::<f32>())
                        .and_then(|row| offset.checked_add(row))
                })
                .ok_or(ComputePassError::UniformPackingOverflow)?;
            let end = offset
                .checked_add(size_of::<f32>())
                .ok_or(ComputePassError::UniformPackingOverflow)?;
            let capacity = BufferByteLength::from(data.len());
            let target =
                data.get_mut(offset..end)
                    .ok_or(ComputePassError::UniformPackingOutOfBounds {
                        offset: BufferByteOffset::from(offset as u64),
                        size: BufferByteLength::from(size_of::<f32>()),
                        capacity,
                    })?;
            target.copy_from_slice(&value.to_le_bytes());
        }
    }
    Ok(())
}
