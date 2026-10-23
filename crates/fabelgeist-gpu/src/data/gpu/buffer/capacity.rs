//! Outcome of ensuring capacity for owned GPU buffer storage.

/// Whether a successful capacity request retained or replaced the storage.
///
/// Growth describes allocation only; it does not certify that a build or
/// dispatch has populated the replacement buffers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferCapacityChange {
    Unchanged,
    Grown,
}
