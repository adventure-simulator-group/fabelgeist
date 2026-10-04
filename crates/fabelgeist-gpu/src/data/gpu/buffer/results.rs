//! Ordered readback buffers and their shared slot identity.

use super::{Buffer, MappedBufferBytes, ReadbackError};

/// An ordinal in one ordered readback, distinct from a byte address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadbackSlot(usize);

/// One device status protocol word. Individual protocols validate their flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadbackStatusWord(u32);

impl ReadbackStatusWord {
    pub const CLEAR: Self = Self(0);
}

impl From<u32> for ReadbackStatusWord {
    fn from(word: u32) -> Self {
        Self(word)
    }
}

impl From<ReadbackStatusWord> for u32 {
    fn from(word: ReadbackStatusWord) -> Self {
        word.0
    }
}

impl From<usize> for ReadbackSlot {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

/// The buffers copied together, in their result order.
pub struct ReadbackSources<'a>(&'a [&'a Buffer]);

impl<'a> From<&'a [&'a Buffer]> for ReadbackSources<'a> {
    fn from(buffers: &'a [&'a Buffer]) -> Self {
        Self(buffers)
    }
}

impl<'a> IntoIterator for ReadbackSources<'a> {
    type Item = &'a Buffer;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, &'a Buffer>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

/// Logical bytes of each copied buffer; alignment padding is excluded.
pub struct ReadbackBlocks {
    buffers: Vec<Vec<u8>>,
}

impl ReadbackBlocks {
    pub fn new(buffers: Vec<Vec<u8>>) -> Self {
        Self { buffers }
    }

    pub fn get(&self, slot: ReadbackSlot) -> Result<MappedBufferBytes<'_>, ReadbackError> {
        match self.buffers.get(slot.0) {
            Some(bytes) => Ok(MappedBufferBytes::from(bytes.as_slice())),
            None => Err(ReadbackError::MissingSlot(slot)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_are_checked_and_element_admission_is_reused() {
        let blocks = ReadbackBlocks::new(vec![vec![1, 2, 3]]);
        assert!(matches!(
            blocks.get(ReadbackSlot::from(1)),
            Err(ReadbackError::MissingSlot(_))
        ));
        assert!(matches!(
            blocks.get(ReadbackSlot::from(0)).unwrap().decode::<u32>(),
            Err(ReadbackError::PartialElement { .. })
        ));
    }
}
