//! Allocate before taking a mapped view; initialize before exposing typed data.
use super::{BufferByteLength, BufferReadError, ReadResult};
use bytemuck::AnyBitPattern;

pub(in crate::data::gpu) struct Readback<T> {
    values: Vec<T>,
    byte_len: usize,
}

impl<T: AnyBitPattern> Readback<T> {
    pub(in crate::data::gpu) fn new(bytes: BufferByteLength) -> ReadResult<Self> {
        let byte_len = usize::try_from(u64::from(bytes)).map_err(BufferReadError::HostAddress)?;
        let element_size = std::mem::size_of::<T>();
        if element_size == 0 {
            return Err(BufferReadError::ZeroSizedElement);
        }
        if !byte_len.is_multiple_of(element_size) {
            return Err(BufferReadError::PartialElement {
                bytes,
                element_bytes: element_size,
            });
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(byte_len / element_size)
            .map_err(BufferReadError::HostAllocation)?;
        Ok(Self { values, byte_len })
    }

    pub(in crate::data::gpu) fn copy_from(mut self, bytes: &[u8]) -> ReadResult<Vec<T>> {
        // GPU allocations may include trailing alignment padding.
        if bytes.len() < self.byte_len {
            return Err(BufferReadError::Truncated {
                requested: BufferByteLength::from(self.byte_len),
                available: BufferByteLength::from(bytes.len()),
            });
        }
        // SAFETY: the allocation fits every byte and is aligned for T. The
        // buffers do not overlap, and AnyBitPattern permits all copied values.
        // Set the length only after every element has been initialized.
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
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
    use super::*;

    #[test]
    fn readback_rejects_partial_and_zero_sized_elements() {
        assert!(Readback::<u32>::new(BufferByteLength::from(3u64)).is_err());
        assert!(Readback::<()>::new(BufferByteLength::from(0u64)).is_err());
        assert!(
            Readback::<u32>::new(BufferByteLength::from(4u64))
                .unwrap()
                .copy_from(&[0; 3])
                .is_err()
        );
    }

    #[test]
    fn readback_copies_unaligned_bytes_and_preserves_bits() {
        let bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        let values = Readback::<u32>::new(BufferByteLength::from(8u64))
            .unwrap()
            .copy_from(&bytes[1..])
            .unwrap();
        assert_eq!(
            values,
            [
                u32::from_ne_bytes([1, 2, 3, 4]),
                u32::from_ne_bytes([5, 6, 7, 8])
            ]
        );
        assert!(
            Readback::<u32>::new(BufferByteLength::from(0u64))
                .unwrap()
                .copy_from(&[])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn readback_excludes_gpu_allocation_padding() {
        let values = Readback::<u8>::new(BufferByteLength::from(3u64))
            .unwrap()
            .copy_from(&[1, 2, 3, 0])
            .unwrap();
        assert_eq!(values, [1, 2, 3]);
    }
}
