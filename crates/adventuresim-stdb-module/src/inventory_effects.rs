//! Shared durable effects; entry points own host or non-live authorization.

pub(crate) use transfer::PersonalInventoryTransfer;

mod transfer;
