//! Borrowed host element bytes admitted for a native buffer upload.

use crate::globals::WgpuContext;

use super::{BufferByteLength, BufferByteOffset};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferUploadOccupancy {
    Empty,
    Populated,
}

/// Exact `NoUninit` element representation, retained through queue submission.
///
/// ```compile_fail
/// use crate::prelude::{BufferUpload, BufferByteLength};
/// let data = BufferUpload::from_elements(&[7u32]);
/// let _: BufferByteLength = data;
/// ```
#[derive(Clone, Copy, Debug)]
pub struct BufferUpload<'a>(&'a [u8]);

const EMPTY_BINDING_WORD: [u8; std::mem::size_of::<u32>()] = [0; std::mem::size_of::<u32>()];

impl<'a> BufferUpload<'a> {
    pub fn from_elements<T: bytemuck::NoUninit>(elements: &'a [T]) -> Self {
        Self(bytemuck::cast_slice(elements))
    }

    pub fn length(self) -> BufferByteLength {
        BufferByteLength::from(self.0.len())
    }

    pub fn occupancy(self) -> BufferUploadOccupancy {
        if self.0.is_empty() {
            BufferUploadOccupancy::Empty
        } else {
            BufferUploadOccupancy::Populated
        }
    }

    /// Keep the established scalar-word binding for an empty armor input.
    pub fn with_empty_word(self) -> Self {
        match self.occupancy() {
            BufferUploadOccupancy::Empty => Self(&EMPTY_BINDING_WORD),
            BufferUploadOccupancy::Populated => self,
        }
    }

    pub(super) fn write_to(
        self,
        context: &WgpuContext,
        buffer: &wgpu::Buffer,
        at: BufferByteOffset,
    ) {
        context.queue.write_buffer(buffer, u64::from(at), self.0);
    }
}
