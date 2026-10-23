//! Capacity selection at the native equipment-occupancy table boundary.

/// A transaction-local reservation is distinct from every stored item ID.
pub(super) enum AttachmentCapacity {
    Free,
    Reserved,
    /// The table's native inventory_item_id; all u64 values remain valid.
    Occupied(u64),
}

pub(super) fn first_free_attachment_capacity(
    capacity: u16,
    inventory_item_id: u64,
    mut occupant_at: impl FnMut(u16) -> AttachmentCapacity,
) -> Option<u16> {
    (0..capacity).find(|index| match occupant_at(*index) {
        AttachmentCapacity::Free => true,
        AttachmentCapacity::Reserved => false,
        AttachmentCapacity::Occupied(occupant) => occupant == inventory_item_id,
    })
}
