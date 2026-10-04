//! Allocate before taking a mapped view; initialize before exposing typed data.

use bytemuck::AnyBitPattern;

use super::{BufferByteLength, ReadbackError, ReadbackRange};

/// Borrowed device-layout bytes, admitted at the mapped-view boundary.
#[derive(Clone, Copy)]
pub struct MappedBufferBytes<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for MappedBufferBytes<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
}

impl<'a> MappedBufferBytes<'a> {
    pub fn segment(self, range: ReadbackRange) -> Result<Self, ReadbackError> {
        let start = usize::try_from(range.start)?;
        let end = usize::try_from(range.end)?;
        self.0
            .get(start..end)
            .map(Self)
            .ok_or(ReadbackError::Truncated {
                expected: BufferByteLength::from(u64::from(range.end)),
                actual: BufferByteLength::from(self.0.len()),
            })
    }

    /// Decode an owned host representation of a device element layout.
    pub fn decode<T: AnyBitPattern>(self) -> Result<Vec<T>, ReadbackError> {
        BufferReadback::<T>::new(BufferByteLength::from(self.0.len()))?.copy_from(self)
    }
}

/// A checked element layout and preallocated host destination for one buffer.
pub struct BufferReadback<T> {
    values: Vec<T>,
    byte_len: usize,
}

impl<T: AnyBitPattern> BufferReadback<T> {
    pub fn new(length: BufferByteLength) -> Result<Self, ReadbackError> {
        let byte_len = usize::try_from(length)?;
        let element_size = std::mem::size_of::<T>();
        if element_size == 0 {
            return Err(ReadbackError::ZeroSizedElement);
        }
        if !byte_len.is_multiple_of(element_size) {
            return Err(ReadbackError::PartialElement {
                length,
                element_bytes: BufferByteLength::from(element_size),
            });
        }
        let mut values = Vec::new();
        if let Err(source) = values.try_reserve_exact(byte_len / element_size) {
            return Err(ReadbackError::Allocation { length, source });
        }
        Ok(Self { values, byte_len })
    }

    /// Convert mapped bytes to initialized values without allocating under the view.
    pub fn copy_from(mut self, bytes: MappedBufferBytes<'_>) -> Result<Vec<T>, ReadbackError> {
        if bytes.0.len() < self.byte_len {
            return Err(ReadbackError::Truncated {
                expected: BufferByteLength::from(self.byte_len),
                actual: BufferByteLength::from(bytes.0.len()),
            });
        }
        // SAFETY: the allocation fits every byte and is aligned for T. The
        // buffers do not overlap, and AnyBitPattern permits all copied values.
        // Set the length only after every element has been initialized.
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.0.as_ptr(),
                self.values.as_mut_ptr().cast::<u8>(),
                self.byte_len,
            );
            self.values
                .set_len(self.byte_len / std::mem::size_of::<T>());
        }
        Ok(self.values)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{BufferUpload, BufferUse};
    use super::*;
    use std::error::Error;

    #[test]
    fn invalid_layouts_and_truncation_are_classifiable() {
        assert!(matches!(
            BufferReadback::<u32>::new(BufferByteLength::from(3u64)),
            Err(ReadbackError::PartialElement { length, element_bytes })
                if length == BufferByteLength::from(3u64)
                    && element_bytes == BufferByteLength::from(4u64)
        ));
        assert!(matches!(
            BufferReadback::<()>::new(BufferByteLength::from(0u64)),
            Err(ReadbackError::ZeroSizedElement)
        ));
        assert!(matches!(
            BufferReadback::<u32>::new(BufferByteLength::from(4u64)).unwrap()
                .copy_from(MappedBufferBytes::from(&[0u8; 3][..])),
            Err(ReadbackError::Truncated { expected, actual })
                if expected == BufferByteLength::from(4u64)
                    && actual == BufferByteLength::from(3u64)
        ));
    }

    #[test]
    fn readback_copies_unaligned_bytes_and_preserves_bits() {
        let bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        let values = BufferReadback::<u32>::new(BufferByteLength::from(8u64))
            .unwrap()
            .copy_from(MappedBufferBytes::from(&bytes[1..]))
            .unwrap();
        assert_eq!(
            values,
            [
                u32::from_ne_bytes([1, 2, 3, 4]),
                u32::from_ne_bytes([5, 6, 7, 8])
            ]
        );
        assert!(
            BufferReadback::<u32>::new(BufferByteLength::from(0u64))
                .unwrap()
                .copy_from(MappedBufferBytes::from(&[][..]))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn readback_excludes_gpu_allocation_padding() {
        let values = BufferReadback::<u8>::new(BufferByteLength::from(3u64))
            .unwrap()
            .copy_from(MappedBufferBytes::from(&[1, 2, 3, 0][..]))
            .unwrap();
        assert_eq!(values, [1, 2, 3]);
    }

    #[test]
    fn unallocatable_lengths_retain_the_host_failure() {
        let error = BufferReadback::<u8>::new(BufferByteLength::from(u64::MAX))
            .err()
            .unwrap();
        assert!(matches!(
            error,
            ReadbackError::Allocation { .. } | ReadbackError::HostLength { .. }
        ));
        assert!(error.source().is_some());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn successful_mappings_are_released_for_reuse() {
        use super::super::{Buffer, BufferDefinition};
        let context = pollster::block_on(crate::globals::WgpuContext::new()).unwrap();
        for definition in [
            BufferDefinition::storage(),
            BufferDefinition::new()
                .with_usage(BufferUse::HostRead)
                .with_usage(BufferUse::CopyDestination),
        ] {
            let buffer = Buffer::from_upload(
                &context,
                BufferUpload::from_elements(&[7u32, 11]),
                definition,
            )
            .unwrap();
            for _ in 0..2 {
                assert_eq!(
                    pollster::block_on(buffer.read::<u32>(&context)).unwrap(),
                    [7, 11]
                );
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn logical_padding_empty_results_and_physical_bounds_are_checked() {
        use super::super::{Buffer, BufferDefinition};
        let context = pollster::block_on(crate::globals::WgpuContext::new()).unwrap();
        let mut buffer = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[1u8, 2, 3, 4]),
            BufferDefinition::storage(),
        )
        .unwrap();
        buffer.length = 3u64.into();
        assert_eq!(
            pollster::block_on(buffer.read::<u8>(&context)).unwrap(),
            [1, 2, 3]
        );
        buffer.length = BufferByteLength::default();
        assert!(
            pollster::block_on(buffer.read::<u32>(&context))
                .unwrap()
                .is_empty()
        );
        assert!(matches!(
            pollster::block_on(buffer.read::<()>(&context)),
            Err(ReadbackError::ZeroSizedElement)
        ));
        buffer.length = 5u64.into();
        assert!(matches!(pollster::block_on(buffer.read::<u8>(&context)),
            Err(ReadbackError::CopyStorage { required, available })
                if required == BufferByteLength::from(8u64) && available == BufferByteLength::from(4u64)));
    }
}
