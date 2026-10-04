//! Per-constraint record admission, upload, and binding ownership.
use crate::ConstraintCount;
use fabelgeist_gpu::prelude::{
    Buffer, BufferCreationError, BufferDefinition, BufferUpload, PassParameterName, PassParameters,
    WgpuContext,
};

/// Human-facing set label, independent of shader parameter identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintName(String);
impl From<&str> for ConstraintName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}
impl From<String> for ConstraintName {
    fn from(name: String) -> Self {
        Self(name)
    }
}
impl std::fmt::Display for ConstraintName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Native serialization captures the record count together with its payload.
/// Handwritten consumers retain both until the owning set admits and uploads it.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{ConstraintAttachment, ConstraintName};
/// ConstraintAttachment::from_records(ConstraintName::from("weights"), &[1u32]);
/// ```
pub struct ConstraintAttachment<'records> {
    parameter: PassParameterName,
    payload: BufferUpload<'records>,
    records: ConstraintCount,
}
impl<'records> ConstraintAttachment<'records> {
    pub fn from_records<T: bytemuck::NoUninit>(
        parameter: PassParameterName,
        records: &'records [T],
    ) -> Self {
        Self {
            parameter,
            payload: BufferUpload::from_elements(records),
            records: records.len().into(),
        }
    }
    pub(super) fn upload(
        self,
        context: &WgpuContext,
        set: &ConstraintName,
        expected: ConstraintCount,
    ) -> Result<(PassParameterName, Buffer), ConstraintAttachmentError> {
        if self.records != expected {
            return Err(ConstraintAttachmentError::RecordCount {
                set: set.clone(),
                parameter: self.parameter,
                actual: self.records,
                expected,
            });
        }
        let definition = BufferDefinition::storage().with_label(self.parameter.to_string().into());
        // Empty sets keep the original single zero-word binding, regardless
        // of record width. A populated record remains an exact native upload.
        let payload = if self.records == ConstraintCount::from(0) {
            self.payload.with_empty_word()
        } else {
            self.payload
        };
        let buffer = Buffer::from_upload(context, payload, definition).map_err(
            |source: BufferCreationError| -> ConstraintAttachmentError {
                ConstraintAttachmentError::Allocation {
                    set: set.clone(),
                    parameter: self.parameter.clone(),
                    source,
                }
            },
        )?;
        Ok((self.parameter, buffer))
    }
}

#[derive(Debug)]
pub enum ConstraintAttachmentError {
    RecordCount {
        set: ConstraintName,
        parameter: PassParameterName,
        actual: ConstraintCount,
        expected: ConstraintCount,
    },
    Allocation {
        set: ConstraintName,
        parameter: PassParameterName,
        source: BufferCreationError,
    },
}
impl std::fmt::Display for ConstraintAttachmentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecordCount {
                set,
                parameter,
                actual,
                expected,
            } => write!(
                formatter,
                "ConstraintSet `{set}`: attachment `{parameter}` has {actual} values for {expected} constraints"
            ),
            Self::Allocation { source, .. } => source.fmt(formatter),
        }
    }
}
impl std::error::Error for ConstraintAttachmentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RecordCount { .. } => None,
            Self::Allocation { source, .. } => Some(source),
        }
    }
}

#[derive(Default)]
pub(super) struct ConstraintAttachments {
    entries: Vec<(PassParameterName, Buffer)>,
}
impl ConstraintAttachments {
    pub(super) fn insert(&mut self, name: PassParameterName, buffer: Buffer) {
        self.entries
            .retain(|(existing, _): &(PassParameterName, Buffer)| -> bool { existing != &name });
        self.entries.push((name, buffer));
    }
    pub(super) fn get(&self, name: &PassParameterName) -> Option<&Buffer> {
        for (existing, buffer) in &self.entries {
            if existing == name {
                return Some(buffer);
            }
        }
        None
    }
    pub(super) fn bind(&self, parameters: &mut PassParameters) {
        for (name, buffer) in &self.entries {
            parameters.insert(name.clone(), buffer.clone().into());
        }
    }
}

#[cfg(test)]
mod tests;
