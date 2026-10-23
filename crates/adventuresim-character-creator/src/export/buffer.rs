//! Packed glTF byte buffers and aligned views.
use serde_json::{Value, json};

#[derive(Default)]
pub(super) struct BufferBuilder {
    pub(super) bytes: Vec<u8>,
    pub(super) views: Vec<Value>,
}

impl BufferBuilder {
    pub(super) fn push(&mut self, bytes: &[u8], target: Option<u32>) -> usize {
        while !self.bytes.len().is_multiple_of(4) {
            self.bytes.push(0);
        }
        let offset = self.bytes.len();
        self.bytes.extend_from_slice(bytes);
        let mut view = json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": bytes.len(),
        });
        if let Some(target) = target {
            view["target"] = target.into();
        }
        self.views.push(view);
        self.views.len() - 1
    }
}

pub(super) fn f32_bytes(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
    values
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>()
}

pub(super) fn u16_bytes(values: impl IntoIterator<Item = u16>) -> Vec<u8> {
    values
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>()
}

pub(super) fn u32_bytes(values: impl IntoIterator<Item = u32>) -> Vec<u8> {
    values
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>()
}
