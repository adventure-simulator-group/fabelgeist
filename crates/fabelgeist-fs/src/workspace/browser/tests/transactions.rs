use super::{fixture::StorageEvent, *};
use crate::BrowserIoCause;
use futures_util::FutureExt;

enum StorageNotification {
    Opened,
    Upgraded,
    OpenFailed,
    RecordRead,
    RecordFailed,
    Committed,
    Aborted,
}
impl StorageFixture {
    fn notify(&self, notification: StorageNotification) {
        let (object, property) = match notification {
            StorageNotification::Opened => (&self.request, "onsuccess"),
            StorageNotification::Upgraded => (&self.request, "onupgradeneeded"),
            StorageNotification::OpenFailed => (&self.request, "onerror"),
            StorageNotification::RecordRead => (&self.read_request, "onsuccess"),
            StorageNotification::RecordFailed => (&self.read_request, "onerror"),
            StorageNotification::Committed => (&self.transaction, "oncomplete"),
            StorageNotification::Aborted => (&self.transaction, "onabort"),
        };
        js_sys::Reflect::get(object, &property.into())
            .unwrap()
            .dyn_into::<js_sys::Function>()
            .unwrap()
            .call0(object)
            .unwrap();
    }
    fn assert_detached(&self) {
        for (object, property) in [
            (&self.request, "onsuccess"),
            (&self.request, "onerror"),
            (&self.request, "onupgradeneeded"),
            (&self.read_request, "onsuccess"),
            (&self.read_request, "onerror"),
            (&self.transaction, "oncomplete"),
            (&self.transaction, "onabort"),
        ] {
            let value = js_sys::Reflect::get(object, &property.into()).unwrap();
            assert!(value.is_null() || value.is_undefined());
        }
    }
}
async fn settle_callbacks() {
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
        .await
        .unwrap();
}
impl WorkspaceStorageError {
    fn assert_provider(&self, expected: WorkspaceStorageStage, original: &BrowserIoCause) {
        assert_eq!(self.stage, expected);
        assert!(std::error::Error::source(self).is_some());
        match &self.failure {
            WorkspaceStorageFailure::Provider(source) => {
                assert!(js_sys::Object::is(source.as_ref(), original.as_ref()));
            }
            _ => panic!("expected original provider cause"),
        }
    }
}

#[wasm_bindgen_test]
async fn read_success_waits_for_commit_and_late_abort_remains_observable() {
    let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(JsValue::UNDEFINED));
    let mut load = Box::pin(storage::WorkspaceStore::load());
    assert!(load.as_mut().now_or_never().is_none());
    settle_callbacks().await;
    fixture.notify(StorageNotification::Opened);
    settle_callbacks().await;
    fixture.notify(StorageNotification::RecordRead);
    settle_callbacks().await;
    assert!(load.as_mut().now_or_never().is_none());
    assert_eq!(*fixture.events.borrow(), [StorageEvent::Read]);
    let cause = js_sys::Error::new("read transaction aborted after request success");
    js_sys::Reflect::set(&fixture.transaction, &"error".into(), &cause).unwrap();
    fixture.notify(StorageNotification::Aborted);
    let error = load.await.unwrap_err();
    error.assert_provider(
        WorkspaceStorageStage::TransactionAbort,
        &BrowserIoCause::from(JsValue::from(cause.clone())),
    );
    assert_eq!(
        *fixture.events.borrow(),
        [StorageEvent::Read, StorageEvent::Closed]
    );
    fixture.assert_detached();
}

#[wasm_bindgen_test]
async fn successful_reads_and_writes_close_once_after_transaction_completion() {
    let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(JsValue::UNDEFINED));
    let mut load = Box::pin(storage::WorkspaceStore::load());
    assert!(load.as_mut().now_or_never().is_none());
    settle_callbacks().await;
    fixture.notify(StorageNotification::Upgraded);
    assert_eq!(*fixture.events.borrow(), [StorageEvent::StoreCreated]);
    fixture.notify(StorageNotification::Opened);
    settle_callbacks().await;
    fixture.notify(StorageNotification::RecordRead);
    fixture.notify(StorageNotification::Committed);
    assert!(matches!(
        load.await.unwrap(),
        storage::StoredWorkspaceRoot::Absent
    ));
    assert_eq!(
        *fixture.events.borrow(),
        [
            StorageEvent::StoreCreated,
            StorageEvent::Read,
            StorageEvent::Closed
        ]
    );
    fixture.assert_detached();
    drop(fixture);

    let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(JsValue::UNDEFINED));
    let directory = WebDirectory::from(
        js_sys::Object::new().unchecked_into::<web_sys::FileSystemDirectoryHandle>(),
    );
    let mut save = Box::pin(storage::WorkspaceStore::save(directory));
    assert!(save.as_mut().now_or_never().is_none());
    settle_callbacks().await;
    fixture.notify(StorageNotification::Opened);
    settle_callbacks().await;
    assert!(save.as_mut().now_or_never().is_none());
    assert_eq!(*fixture.events.borrow(), [StorageEvent::Write]);
    fixture.notify(StorageNotification::Committed);
    save.await.unwrap();
    assert_eq!(
        *fixture.events.borrow(),
        [StorageEvent::Write, StorageEvent::Closed]
    );
    fixture.assert_detached();
}

#[wasm_bindgen_test]
async fn store_and_record_exceptions_abort_and_preserve_the_first_cause() {
    let cause = js_sys::Error::new("record SDK invocation failed");
    for (mode, expected) in [
        (
            StorageFixtureMode::StoreFailure(cause.clone().into()),
            WorkspaceStorageStage::SelectStore,
        ),
        (
            StorageFixtureMode::ReadFailure(cause.clone().into()),
            WorkspaceStorageStage::ReadRecord,
        ),
    ] {
        let fixture = StorageFixture::new(mode);
        let mut load = Box::pin(storage::WorkspaceStore::load());
        assert!(load.as_mut().now_or_never().is_none());
        settle_callbacks().await;
        fixture.notify(StorageNotification::Opened);
        load.await.unwrap_err().assert_provider(
            expected,
            &BrowserIoCause::from(JsValue::from(cause.clone())),
        );
        assert!(
            fixture
                .events
                .borrow()
                .ends_with(&[StorageEvent::Aborted, StorageEvent::Closed])
        );
        fixture.assert_detached();
    }
    let fixture = StorageFixture::new(StorageFixtureMode::WriteFailure(cause.clone().into()));
    let directory = WebDirectory::from(
        js_sys::Object::new().unchecked_into::<web_sys::FileSystemDirectoryHandle>(),
    );
    let mut save = Box::pin(storage::WorkspaceStore::save(directory.clone()));
    assert!(save.as_mut().now_or_never().is_none());
    settle_callbacks().await;
    fixture.notify(StorageNotification::Opened);
    let error = save.await.unwrap_err();
    error.assert_provider(
        WorkspaceStorageStage::WriteRecord,
        &BrowserIoCause::from(JsValue::from(cause.clone())),
    );
    match error.operation {
        WorkspaceStorageOperation::Save {
            directory: original,
        } => {
            assert!(js_sys::Object::is(original.as_ref(), directory.as_ref()));
        }
        _ => panic!("save errors retain the selected directory"),
    }
    assert_eq!(
        *fixture.events.borrow(),
        [
            StorageEvent::Write,
            StorageEvent::Aborted,
            StorageEvent::Closed
        ]
    );
    fixture.assert_detached();
}

#[wasm_bindgen_test]
async fn upgrade_and_async_request_failures_keep_causes_and_release_callbacks() {
    let cause = js_sys::Error::new("schema initialization failed");
    let fixture = StorageFixture::new(StorageFixtureMode::UpgradeFailure(cause.clone().into()));
    let mut load = Box::pin(storage::WorkspaceStore::load());
    assert!(load.as_mut().now_or_never().is_none());
    settle_callbacks().await;
    fixture.notify(StorageNotification::Upgraded);
    fixture.notify(StorageNotification::OpenFailed);
    load.await.unwrap_err().assert_provider(
        WorkspaceStorageStage::InitializeStore,
        &BrowserIoCause::from(JsValue::from(cause.clone())),
    );
    assert_eq!(
        *fixture.events.borrow(),
        [
            StorageEvent::StoreCreated,
            StorageEvent::Closed,
            StorageEvent::Aborted
        ]
    );
    fixture.assert_detached();
    drop(fixture);

    let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(JsValue::UNDEFINED));
    let mut load = Box::pin(storage::WorkspaceStore::load());
    assert!(load.as_mut().now_or_never().is_none());
    settle_callbacks().await;
    fixture.notify(StorageNotification::Opened);
    settle_callbacks().await;
    js_sys::Reflect::set(&fixture.read_request, &"error".into(), &cause).unwrap();
    fixture.notify(StorageNotification::RecordFailed);
    load.await.unwrap_err().assert_provider(
        WorkspaceStorageStage::RecordRequest,
        &BrowserIoCause::from(JsValue::from(cause.clone())),
    );
    assert_eq!(
        *fixture.events.borrow(),
        [
            StorageEvent::Read,
            StorageEvent::Aborted,
            StorageEvent::Closed
        ]
    );
    fixture.assert_detached();
}

#[wasm_bindgen_test]
async fn caller_cancellation_keeps_the_storage_driver_alive_for_cleanup() {
    let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(JsValue::UNDEFINED));
    assert!(storage::WorkspaceStore::load().now_or_never().is_none());
    settle_callbacks().await;
    fixture.notify(StorageNotification::Opened);
    settle_callbacks().await;
    fixture.notify(StorageNotification::RecordRead);
    fixture.notify(StorageNotification::Committed);
    settle_callbacks().await;
    assert_eq!(
        *fixture.events.borrow(),
        [StorageEvent::Read, StorageEvent::Closed]
    );
    fixture.assert_detached();
}

#[wasm_bindgen_test]
async fn incomplete_read_and_missing_abort_causes_are_structured_failures() {
    for notification in [StorageNotification::Committed, StorageNotification::Aborted] {
        let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(JsValue::UNDEFINED));
        let mut load = Box::pin(storage::WorkspaceStore::load());
        assert!(load.as_mut().now_or_never().is_none());
        settle_callbacks().await;
        fixture.notify(StorageNotification::Opened);
        settle_callbacks().await;
        fixture.notify(notification);
        let error = load.await.unwrap_err();
        assert!(matches!(
            error.failure,
            WorkspaceStorageFailure::MissingReadResult
                | WorkspaceStorageFailure::MissingProviderError
        ));
        assert!(std::error::Error::source(&error).is_none());
        assert_eq!(
            *fixture.events.borrow(),
            [StorageEvent::Read, StorageEvent::Closed]
        );
        fixture.assert_detached();
    }
}

#[wasm_bindgen_test]
async fn malformed_factories_database_results_and_open_failures_keep_original_values() {
    let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(JsValue::UNDEFINED));
    js_sys::Reflect::set(&js_sys::global(), &"indexedDB".into(), &JsValue::NULL).unwrap();
    storage::WorkspaceStore::load()
        .await
        .unwrap_err()
        .assert_provider(
            WorkspaceStorageStage::DecodeFactory,
            &BrowserIoCause::from(JsValue::NULL),
        );
    fixture.assert_detached();
    assert!(fixture.events.borrow().is_empty());
    drop(fixture);

    let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(JsValue::UNDEFINED));
    js_sys::Reflect::set(&fixture.request, &"result".into(), &JsValue::FALSE).unwrap();
    let mut load = Box::pin(storage::WorkspaceStore::load());
    assert!(load.as_mut().now_or_never().is_none());
    settle_callbacks().await;
    fixture.notify(StorageNotification::Opened);
    load.await.unwrap_err().assert_provider(
        WorkspaceStorageStage::DecodeDatabase,
        &BrowserIoCause::from(JsValue::FALSE),
    );
    fixture.assert_detached();
    assert!(fixture.events.borrow().is_empty());
    drop(fixture);

    let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(JsValue::UNDEFINED));
    let cause = js_sys::Error::new("asynchronous open failed");
    js_sys::Reflect::set(&fixture.request, &"error".into(), &cause).unwrap();
    let mut load = Box::pin(storage::WorkspaceStore::load());
    assert!(load.as_mut().now_or_never().is_none());
    settle_callbacks().await;
    fixture.notify(StorageNotification::OpenFailed);
    load.await.unwrap_err().assert_provider(
        WorkspaceStorageStage::OpenRequest,
        &BrowserIoCause::from(JsValue::from(cause.clone())),
    );
    fixture.assert_detached();
    assert!(fixture.events.borrow().is_empty());
}
