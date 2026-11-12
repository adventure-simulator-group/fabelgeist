//! Allocate before taking a mapped view; initialize before exposing typed data.
use anyhow::{Result, ensure};
use bytemuck::AnyBitPattern;

pub(super) struct Readback<T> {
    values: Vec<T>,
    byte_len: usize,
}

impl<T: AnyBitPattern> Readback<T> {
    pub(super) fn new(byte_len: u64) -> Result<Self> {
        let byte_len = usize::try_from(byte_len)?;
        let element_size = std::mem::size_of::<T>();
        ensure!(
            element_size != 0,
            "GPU readback requires nonzero-sized elements"
        );
        ensure!(
            byte_len.is_multiple_of(element_size),
            "GPU readback byte length is not a whole number of elements"
        );
        Ok(Self {
            values: Vec::with_capacity(byte_len / element_size),
            byte_len,
        })
    }

    pub(super) fn copy_from(mut self, bytes: &[u8]) -> Result<Vec<T>> {
        // GPU allocations may include trailing alignment padding.
        ensure!(bytes.len() >= self.byte_len, "GPU readback is truncated");
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
        assert!(Readback::<u32>::new(3).is_err());
        assert!(Readback::<()>::new(0).is_err());
        assert!(Readback::<u32>::new(4).unwrap().copy_from(&[0; 3]).is_err());
    }

    #[test]
    fn readback_copies_unaligned_bytes_and_preserves_bits() {
        let bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        let values = Readback::<u32>::new(8)
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
            Readback::<u32>::new(0)
                .unwrap()
                .copy_from(&[])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn readback_excludes_gpu_allocation_padding() {
        let values = Readback::<u8>::new(3)
            .unwrap()
            .copy_from(&[1, 2, 3, 0])
            .unwrap();
        assert_eq!(values, [1, 2, 3]);
    }
}
