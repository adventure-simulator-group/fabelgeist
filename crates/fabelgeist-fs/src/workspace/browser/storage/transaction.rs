use super::{
    StoredWorkspaceRoot, WorkspaceStorageError, WorkspaceStorageOperation, WorkspaceStorageStage,
    WorkspaceStore, open::WorkspaceDatabase,
};
use crate::WebDirectory;
use futures_channel::oneshot;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkspaceTransactionPhase {
    Pending,
    Committed,
    Aborted,
}
enum WorkspaceReadState {
    Pending,
    Read(StoredWorkspaceRoot),
}
struct WorkspaceTransactionCompletion {
    sender: RefCell<Option<oneshot::Sender<Result<(), WorkspaceStorageError>>>>,
    phase: Cell<WorkspaceTransactionPhase>,
}
impl WorkspaceTransactionCompletion {
    fn finish(&self, result: Result<(), WorkspaceStorageError>) {
        if let Some(sender) = self.sender.borrow_mut().take() {
            let _ = sender.send(result);
        }
    }
}
struct WorkspaceTransaction {
    transaction: web_sys::IdbTransaction,
    operation: WorkspaceStorageOperation,
    completion: Rc<WorkspaceTransactionCompletion>,
    receiver: oneshot::Receiver<Result<(), WorkspaceStorageError>>,
    _complete: Closure<dyn FnMut()>,
    _abort: Closure<dyn FnMut()>,
}
impl WorkspaceTransaction {
    fn new(
        database: &WorkspaceDatabase,
        operation: WorkspaceStorageOperation,
    ) -> Result<Self, WorkspaceStorageError> {
        let mode = match operation {
            WorkspaceStorageOperation::Load => web_sys::IdbTransactionMode::Readonly,
            WorkspaceStorageOperation::Save { .. } => web_sys::IdbTransactionMode::Readwrite,
        };
        let transaction = database
            .database
            .transaction_with_str_and_mode(WorkspaceStore::HANDLE_STORE, mode)
            .map_err(|source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    &operation,
                    WorkspaceStorageStage::BeginTransaction,
                    source,
                )
            })?;
        let (sender, receiver) = oneshot::channel();
        let completion = Rc::new(WorkspaceTransactionCompletion {
            sender: RefCell::new(Some(sender)),
            phase: Cell::new(WorkspaceTransactionPhase::Pending),
        });
        let complete_completion = completion.clone();
        let complete = Closure::wrap(Box::new(move || -> () {
            complete_completion
                .phase
                .set(WorkspaceTransactionPhase::Committed);
            complete_completion.finish(Ok(()));
        }) as Box<dyn FnMut()>);
        let abort_transaction = transaction.clone();
        let abort_operation = operation.clone();
        let abort_completion = completion.clone();
        let abort = Closure::wrap(Box::new(move || -> () {
            abort_completion
                .phase
                .set(WorkspaceTransactionPhase::Aborted);
            let cause = js_sys::Reflect::get(&abort_transaction, &"error".into());
            let error = match cause {
                Ok(value) if value.is_null() || value.is_undefined() => {
                    WorkspaceStorageError::missing_error(
                        &abort_operation,
                        WorkspaceStorageStage::TransactionAbort,
                    )
                }
                Ok(source) | Err(source) => WorkspaceStorageError::from_browser(
                    &abort_operation,
                    WorkspaceStorageStage::TransactionAbort,
                    source,
                ),
            };
            abort_completion.finish(Err(error));
        }) as Box<dyn FnMut()>);
        transaction.set_oncomplete(Some(complete.as_ref().unchecked_ref()));
        transaction.set_onabort(Some(abort.as_ref().unchecked_ref()));
        Ok(Self {
            transaction,
            operation,
            completion,
            receiver,
            _complete: complete,
            _abort: abort,
        })
    }

    fn object_store(&self) -> Result<web_sys::IdbObjectStore, WorkspaceStorageError> {
        self.transaction
            .object_store(WorkspaceStore::HANDLE_STORE)
            .map_err(|source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    &self.operation,
                    WorkspaceStorageStage::SelectStore,
                    source,
                )
            })
    }
    async fn committed(&mut self) -> Result<(), WorkspaceStorageError> {
        (&mut self.receiver).await.map_err(
            |_source: oneshot::Canceled| -> WorkspaceStorageError {
                WorkspaceStorageError::interrupted(&self.operation)
            },
        )?
    }
}
impl Drop for WorkspaceTransaction {
    fn drop(&mut self) {
        self.transaction.set_oncomplete(None);
        self.transaction.set_onabort(None);
        if self.completion.phase.get() == WorkspaceTransactionPhase::Pending {
            // Keep the first operation error; an already-aborted transaction
            // may reject this cleanup attempt with InvalidStateError.
            let _ = self.transaction.abort();
        }
    }
}

struct WorkspaceRecordListeners {
    request: web_sys::IdbRequest,
    _success: Option<Closure<dyn FnMut()>>,
    _error: Closure<dyn FnMut()>,
}
impl WorkspaceRecordListeners {
    fn new(request: web_sys::IdbRequest, transaction: &WorkspaceTransaction) -> Self {
        let error_request = request.clone();
        let operation = transaction.operation.clone();
        let completion = transaction.completion.clone();
        let error = Closure::wrap(Box::new(move || -> () {
            let error = match error_request.error() {
                Ok(Some(source)) => WorkspaceStorageError::from_browser(
                    &operation,
                    WorkspaceStorageStage::RecordRequest,
                    source.into(),
                ),
                Err(source) => WorkspaceStorageError::from_browser(
                    &operation,
                    WorkspaceStorageStage::RecordRequest,
                    source,
                ),
                Ok(None) => WorkspaceStorageError::missing_error(
                    &operation,
                    WorkspaceStorageStage::RecordRequest,
                ),
            };
            completion.finish(Err(error));
        }) as Box<dyn FnMut()>);
        request.set_onerror(Some(error.as_ref().unchecked_ref()));
        Self {
            request,
            _success: None,
            _error: error,
        }
    }
    fn read(
        request: web_sys::IdbRequest,
        transaction: &WorkspaceTransaction,
        state: &Rc<RefCell<WorkspaceReadState>>,
    ) -> Self {
        let mut listeners = Self::new(request.clone(), transaction);
        let state = state.clone();
        let completion = transaction.completion.clone();
        let success = Closure::wrap(Box::new(move || -> () {
            let result = request
                .result()
                .map_err(|source: JsValue| -> WorkspaceStorageError {
                    WorkspaceStorageError::from_browser(
                        &WorkspaceStorageOperation::Load,
                        WorkspaceStorageStage::ReadResult,
                        source,
                    )
                })
                .and_then(StoredWorkspaceRoot::from_browser);
            match result {
                Ok(root) => *state.borrow_mut() = WorkspaceReadState::Read(root),
                Err(error) => completion.finish(Err(error)),
            }
        }) as Box<dyn FnMut()>);
        listeners
            .request
            .set_onsuccess(Some(success.as_ref().unchecked_ref()));
        listeners._success = Some(success);
        listeners
    }
}
impl Drop for WorkspaceRecordListeners {
    fn drop(&mut self) {
        self.request.set_onsuccess(None);
        self.request.set_onerror(None);
    }
}

impl WorkspaceDatabase {
    pub(super) async fn load(&self) -> Result<StoredWorkspaceRoot, WorkspaceStorageError> {
        let mut transaction = WorkspaceTransaction::new(self, WorkspaceStorageOperation::Load)?;
        let request = transaction
            .object_store()?
            .get(&WorkspaceStore::ROOT_KEY.into())
            .map_err(|source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    &WorkspaceStorageOperation::Load,
                    WorkspaceStorageStage::ReadRecord,
                    source,
                )
            })?;
        let state = Rc::new(RefCell::new(WorkspaceReadState::Pending));
        let _listeners = WorkspaceRecordListeners::read(request, &transaction, &state);
        transaction.committed().await?;
        match state.replace(WorkspaceReadState::Pending) {
            WorkspaceReadState::Read(root) => Ok(root),
            WorkspaceReadState::Pending => Err(WorkspaceStorageError::missing_read()),
        }
    }
    pub(super) async fn save(&self, directory: WebDirectory) -> Result<(), WorkspaceStorageError> {
        let operation = WorkspaceStorageOperation::Save {
            directory: directory.clone(),
        };
        let mut transaction = WorkspaceTransaction::new(self, operation)?;
        let request = transaction
            .object_store()?
            .put_with_key(directory.as_ref(), &WorkspaceStore::ROOT_KEY.into())
            .map_err(|source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    &transaction.operation,
                    WorkspaceStorageStage::WriteRecord,
                    source,
                )
            })?;
        let _listeners = WorkspaceRecordListeners::new(request, &transaction);
        transaction.committed().await
    }
}
