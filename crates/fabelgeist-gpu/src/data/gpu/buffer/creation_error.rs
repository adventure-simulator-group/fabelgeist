//! Admission failure before allocating a physical GPU buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BufferCreationError {
    #[error("Buffer size must be greater than 0")]
    Empty,
}
