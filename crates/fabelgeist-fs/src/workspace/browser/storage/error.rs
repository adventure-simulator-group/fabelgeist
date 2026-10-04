use crate::{BrowserIoCause, WebDirectory};
use wasm_bindgen::JsValue;

#[derive(Clone, Debug)]
pub enum WorkspaceStorageOperation {
    Load,
    Save { directory: WebDirectory },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceStorageStage {
    ReadFactory,
    DecodeFactory,
    OpenDatabase,
    ReadDatabase,
    DecodeDatabase,
    InitializeStore,
    OpenRequest,
    BeginTransaction,
    SelectStore,
    ReadRecord,
    WriteRecord,
    ReadResult,
    DecodeRecord,
    RecordRequest,
    TransactionAbort,
    TransactionCompletion,
    Driver,
}

#[derive(Debug)]
pub enum WorkspaceStorageFailure {
    Provider(BrowserIoCause),
    MissingProviderError,
    MissingReadResult,
    Interrupted,
}
#[derive(Debug)]
pub struct WorkspaceStorageError {
    pub operation: WorkspaceStorageOperation,
    pub stage: WorkspaceStorageStage,
    pub failure: WorkspaceStorageFailure,
}
impl WorkspaceStorageError {
    pub(super) fn from_browser(
        operation: &WorkspaceStorageOperation,
        stage: WorkspaceStorageStage,
        source: JsValue,
    ) -> Self {
        Self {
            operation: operation.clone(),
            stage,
            failure: WorkspaceStorageFailure::Provider(BrowserIoCause::from(source)),
        }
    }
    pub(super) fn missing_error(
        operation: &WorkspaceStorageOperation,
        stage: WorkspaceStorageStage,
    ) -> Self {
        Self {
            operation: operation.clone(),
            stage,
            failure: WorkspaceStorageFailure::MissingProviderError,
        }
    }
    pub(super) fn interrupted(operation: &WorkspaceStorageOperation) -> Self {
        Self {
            operation: operation.clone(),
            stage: WorkspaceStorageStage::Driver,
            failure: WorkspaceStorageFailure::Interrupted,
        }
    }
    pub(super) fn missing_read() -> Self {
        Self {
            operation: WorkspaceStorageOperation::Load,
            stage: WorkspaceStorageStage::TransactionCompletion,
            failure: WorkspaceStorageFailure::MissingReadResult,
        }
    }
}
impl std::fmt::Display for WorkspaceStorageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "workspace record storage failed: {self:?}")
    }
}
impl std::error::Error for WorkspaceStorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.failure {
            WorkspaceStorageFailure::Provider(source) => Some(source),
            _ => None,
        }
    }
}
