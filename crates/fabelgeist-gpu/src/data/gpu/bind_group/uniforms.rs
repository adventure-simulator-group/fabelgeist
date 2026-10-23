use crate::data::{BindGroupReflection, PassParameter, PassParameters, UniformMember};

/// Pack Naga's reflected byte offsets using the existing static-layout policy.
pub(super) fn pack_uniforms(
    reflection: &BindGroupReflection,
    parameters: &PassParameters,
) -> Vec<u8> {
    let mut data = vec![0u8; reflection.uniform_buffer_size as usize];
    for member in &reflection.uniform_members {
        if let Some(value) = parameters.get(&member.name) {
            write_value(&mut data, member, value);
        }
    }
    data
}

fn write_value(data: &mut [u8], member: &UniformMember, value: &PassParameter) {
    let offset = member.offset as usize;
    match value {
        PassParameter::Number(value) => write_bytes(data, offset, &(*value as f32).to_le_bytes()),
        PassParameter::Unsigned(value) => write_bytes(data, offset, &value.to_le_bytes()),
        PassParameter::Vec2(value) => write_vector(data, offset, [value.x, value.y]),
        PassParameter::Vec3(value) => write_vector(data, offset, [value.x, value.y, value.z]),
        PassParameter::Vec4(value) => {
            write_vector(data, offset, [value.x, value.y, value.z, value.w])
        }
        PassParameter::Mat2(value) => write_matrix(data, member, &value.columns, 16, 8),
        PassParameter::Mat3(value) => write_matrix(
            data,
            member,
            &value.columns,
            member.size as usize,
            (member.size / 3) as usize,
        ),
        PassParameter::Mat4(value) => write_matrix(data, member, &value.columns, 64, 16),
        PassParameter::Transform(value) => {
            write_matrix(data, member, &value.to_mat4().columns, 64, 16)
        }
        _ => {}
    }
}

fn write_bytes(data: &mut [u8], offset: usize, bytes: &[u8]) {
    if let Some(target) = data.get_mut(offset..offset + bytes.len()) {
        target.copy_from_slice(bytes);
    }
}

fn write_vector<const N: usize>(data: &mut [u8], offset: usize, values: [f32; N]) {
    if offset + N * size_of::<f32>() <= data.len() {
        for (index, value) in values.iter().enumerate() {
            write_bytes(
                data,
                offset + index * size_of::<f32>(),
                &value.to_le_bytes(),
            );
        }
    }
}

// These spans and strides are the native scalar/matrix ABI, in bytes. Mat3
// inherits Naga's padded column stride; Mat2/Mat4 retain their existing layout.
fn write_matrix<const N: usize>(
    data: &mut [u8],
    member: &UniformMember,
    columns: &[[f32; N]; N],
    span: usize,
    column_stride: usize,
) {
    let start = member.offset as usize;
    if start + span <= data.len() {
        for (column, values) in columns.iter().enumerate() {
            for (row, value) in values.iter().enumerate() {
                write_bytes(
                    data,
                    start + column * column_stride + row * size_of::<f32>(),
                    &value.to_le_bytes(),
                );
            }
        }
    }
}
