//! Allocate before taking a mapped view; initialize before exposing typed data.
use super::{BufferByteLength, BufferReadError, BufferReadResult};
use bytemuck::AnyBitPattern;

pub(super) struct Readback<T> {
    values: Vec<T>,
    byte_len: BufferByteLength,
}

impl<T: AnyBitPattern> Readback<T> {
    pub(super) fn new(byte_len: BufferByteLength) -> BufferReadResult<Self> {
        // The host allocation counts native bytes. Admit the logical extent
        // once before allocation, preserving conversion-before-element checks.
        let host_bytes = usize::try_from(u64::from(byte_len)).map_err(|cause| {
            BufferReadError::HostLengthConversion {
                bytes: byte_len,
                cause,
            }
        })?;
        let element_size = std::mem::size_of::<T>();
        if element_size == 0 {
            return Err(BufferReadError::ZeroSizedElement);
        }
        if !host_bytes.is_multiple_of(element_size) {
            return Err(BufferReadError::PartialElement {
                bytes: byte_len,
                element_bytes: element_size.into(),
            });
        }
        Ok(Self {
            values: Vec::with_capacity(host_bytes / element_size),
            byte_len,
        })
    }

    pub(super) fn copy_from(mut self, bytes: &[u8]) -> BufferReadResult<Vec<T>> {
        // GPU allocations may include trailing alignment padding.
        if BufferByteLength::from(bytes.len()) < self.byte_len {
            return Err(BufferReadError::TruncatedMappedView {
                expected: self.byte_len,
                available: bytes.len().into(),
            });
        }
        // Native pointer/vector operations count host bytes/elements. `new`
        // already proved this stored logical extent fits usize and whole T's.
        let host_bytes = u64::from(self.byte_len) as usize;
        // SAFETY: the allocation fits every byte and is aligned for T. The
        // buffers do not overlap, and AnyBitPattern permits all copied values.
        // Set the length only after every element has been initialized.
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                self.values.as_mut_ptr().cast::<u8>(),
                host_bytes,
            );
            self.values.set_len(host_bytes / std::mem::size_of::<T>());
        }
        Ok(self.values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readback_rejects_partial_and_zero_sized_elements() {
        assert!(matches!(
            Readback::<u32>::new(3u64.into()),
            Err(BufferReadError::PartialElement { bytes, element_bytes })
                if bytes == 3u64.into() && element_bytes == 4u64.into()
        ));
        assert!(matches!(
            Readback::<()>::new(0u64.into()),
            Err(BufferReadError::ZeroSizedElement)
        ));
        let error = Readback::<u32>::new(4u64.into())
            .unwrap()
            .copy_from(&[0; 3])
            .unwrap_err();
        assert!(matches!(
            error,
            BufferReadError::TruncatedMappedView { expected, available }
                if expected == 4u64.into() && available == 3u64.into()
        ));
        assert_eq!(error.to_string(), "GPU readback is truncated");
        assert!(std::error::Error::source(&error).is_none());
    }

    #[test]
    fn readback_copies_unaligned_bytes_and_preserves_bits() {
        let bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        let values = Readback::<u32>::new((8u64).into())
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
            Readback::<u32>::new((0u64).into())
                .unwrap()
                .copy_from(&[])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn readback_excludes_gpu_allocation_padding() {
        let values = Readback::<u8>::new((3u64).into())
            .unwrap()
            .copy_from(&[1, 2, 3, 0])
            .unwrap();
        assert_eq!(values, [1, 2, 3]);
    }
}
