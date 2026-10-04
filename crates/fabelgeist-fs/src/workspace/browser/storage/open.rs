use super::{
    WorkspaceStorageError, WorkspaceStorageOperation, WorkspaceStorageStage, WorkspaceStore,
};
use futures_channel::oneshot;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

pub(super) struct WorkspaceDatabase {
    pub(super) database: web_sys::IdbDatabase,
}
struct WorkspaceOpenCompletion {
    sender: RefCell<Option<oneshot::Sender<Result<WorkspaceDatabase, WorkspaceStorageError>>>>,
    upgrade_error: RefCell<Option<WorkspaceStorageError>>,
    upgrade_database: RefCell<Option<WorkspaceDatabase>>,
}
impl WorkspaceOpenCompletion {
    fn finish(&self, result: Result<WorkspaceDatabase, WorkspaceStorageError>) {
        if let Some(sender) = self.sender.borrow_mut().take() {
            let _ = sender.send(result);
        }
    }
}
struct WorkspaceOpenListeners {
    request: web_sys::IdbOpenDbRequest,
    _upgrade: Closure<dyn FnMut()>,
    _success: Closure<dyn FnMut()>,
    _error: Closure<dyn FnMut()>,
}
impl WorkspaceOpenListeners {
    fn new(
        request: web_sys::IdbOpenDbRequest,
        operation: &WorkspaceStorageOperation,
        completion: &Rc<WorkspaceOpenCompletion>,
    ) -> Self {
        let upgrade_request = request.clone();
        let upgrade_operation = operation.clone();
        let upgrade_completion = completion.clone();
        let upgrade = Closure::wrap(Box::new(move || -> () {
            match WorkspaceDatabase::initialize_store(&upgrade_request, &upgrade_operation) {
                Ok(database) => {
                    *upgrade_completion.upgrade_database.borrow_mut() = Some(database);
                }
                Err(error) => {
                    *upgrade_completion.upgrade_error.borrow_mut() = Some(error);
                    if let Some(transaction) = upgrade_request.transaction() {
                        let _ = transaction.abort();
                    }
                }
            }
        }) as Box<dyn FnMut()>);
        let success_request = request.clone();
        let success_operation = operation.clone();
        let success_completion = completion.clone();
        let success = Closure::wrap(Box::new(move || -> () {
            let database = match success_completion.upgrade_database.borrow_mut().take() {
                Some(database) => Ok(database),
                None => WorkspaceDatabase::from_request(&success_request, &success_operation),
            };
            let result = match success_completion.upgrade_error.borrow_mut().take() {
                Some(error) => Err(error),
                None => database,
            };
            success_completion.finish(result);
        }) as Box<dyn FnMut()>);
        let error_request = request.clone();
        let error_operation = operation.clone();
        let error_completion = completion.clone();
        let error = Closure::wrap(Box::new(move || -> () {
            error_completion.upgrade_database.borrow_mut().take();
            let error = error_completion
                .upgrade_error
                .borrow_mut()
                .take()
                .unwrap_or_else(|| -> WorkspaceStorageError {
                    match error_request.error() {
                        Ok(Some(source)) => WorkspaceStorageError::from_browser(
                            &error_operation,
                            WorkspaceStorageStage::OpenRequest,
                            source.into(),
                        ),
                        Err(source) => WorkspaceStorageError::from_browser(
                            &error_operation,
                            WorkspaceStorageStage::OpenRequest,
                            source,
                        ),
                        Ok(None) => WorkspaceStorageError::missing_error(
                            &error_operation,
                            WorkspaceStorageStage::OpenRequest,
                        ),
                    }
                });
            error_completion.finish(Err(error));
        }) as Box<dyn FnMut()>);
        request.set_onupgradeneeded(Some(upgrade.as_ref().unchecked_ref()));
        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
        request.set_onerror(Some(error.as_ref().unchecked_ref()));
        Self {
            request,
            _upgrade: upgrade,
            _success: success,
            _error: error,
        }
    }
}
impl Drop for WorkspaceOpenListeners {
    fn drop(&mut self) {
        self.request.set_onupgradeneeded(None);
        self.request.set_onsuccess(None);
        self.request.set_onerror(None);
    }
}
impl WorkspaceDatabase {
    pub(super) async fn open(
        operation: &WorkspaceStorageOperation,
    ) -> Result<Self, WorkspaceStorageError> {
        let factory = js_sys::Reflect::get(&js_sys::global(), &"indexedDB".into()).map_err(
            |source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    operation,
                    WorkspaceStorageStage::ReadFactory,
                    source,
                )
            },
        )?;
        let factory = factory.dyn_into::<web_sys::IdbFactory>().map_err(
            |source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    operation,
                    WorkspaceStorageStage::DecodeFactory,
                    source,
                )
            },
        )?;
        let request = factory
            .open_with_u32(
                WorkspaceStore::DATABASE_NAME,
                WorkspaceStore::DATABASE_VERSION,
            )
            .map_err(|source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    operation,
                    WorkspaceStorageStage::OpenDatabase,
                    source,
                )
            })?;
        let (sender, receiver) = oneshot::channel();
        let completion = Rc::new(WorkspaceOpenCompletion {
            sender: RefCell::new(Some(sender)),
            upgrade_error: RefCell::default(),
            upgrade_database: RefCell::default(),
        });
        let _listeners = WorkspaceOpenListeners::new(request, operation, &completion);
        receiver
            .await
            .map_err(|_source: oneshot::Canceled| -> WorkspaceStorageError {
                WorkspaceStorageError::interrupted(operation)
            })?
    }

    fn from_request(
        request: &web_sys::IdbOpenDbRequest,
        operation: &WorkspaceStorageOperation,
    ) -> Result<Self, WorkspaceStorageError> {
        let value = request
            .result()
            .map_err(|source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    operation,
                    WorkspaceStorageStage::ReadDatabase,
                    source,
                )
            })?;
        let database = value.dyn_into::<web_sys::IdbDatabase>().map_err(
            |source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    operation,
                    WorkspaceStorageStage::DecodeDatabase,
                    source,
                )
            },
        )?;
        Ok(Self { database })
    }

    fn initialize_store(
        request: &web_sys::IdbOpenDbRequest,
        operation: &WorkspaceStorageOperation,
    ) -> Result<Self, WorkspaceStorageError> {
        let database = Self::from_request(request, operation)?;
        database
            .database
            .create_object_store(WorkspaceStore::HANDLE_STORE)
            .map_err(|source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    operation,
                    WorkspaceStorageStage::InitializeStore,
                    source,
                )
            })?;
        Ok(database)
    }
}
impl Drop for WorkspaceDatabase {
    fn drop(&mut self) {
        self.database.close();
    }
}
