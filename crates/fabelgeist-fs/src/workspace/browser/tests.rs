use super::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::wasm_bindgen_test;

mod fixture;
mod transactions;
pub(in crate::workspace::browser) use fixture::{StorageFixture, StorageFixtureMode};

#[wasm_bindgen_test]
async fn database_open_and_transaction_exceptions_reach_structured_workspace_errors() {
    let cause = js_sys::Error::new("database provider rejected");
    for mode in [
        StorageFixtureMode::OpenFailure(cause.clone().into()),
        StorageFixtureMode::TransactionFailure(cause.clone().into()),
    ] {
        let fixture = StorageFixture::new(mode.clone());
        if matches!(mode, StorageFixtureMode::TransactionFailure(_)) {
            fixture.dispatch_read();
        }
        match ProjectRoot::restore().await.unwrap_err() {
            WorkspaceError::Storage(WorkspaceStorageError {
                stage,
                failure: WorkspaceStorageFailure::Provider(source),
                ..
            }) => {
                let expected = match mode {
                    StorageFixtureMode::OpenFailure(_) => WorkspaceStorageStage::OpenDatabase,
                    _ => WorkspaceStorageStage::BeginTransaction,
                };
                assert_eq!(stage, expected);
                assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
            }
            _ => panic!("expected original database cause"),
        }
    }
    let _fixture = StorageFixture::new(StorageFixtureMode::OpenFailure(cause.clone().into()));
    let directory = WebDirectory::from(
        js_sys::Object::new().unchecked_into::<web_sys::FileSystemDirectoryHandle>(),
    );
    match ProjectRoot::from(directory).persist().await.unwrap_err() {
        WorkspaceError::Storage(WorkspaceStorageError {
            operation,
            stage,
            failure: WorkspaceStorageFailure::Provider(source),
        }) => {
            assert!(matches!(operation, WorkspaceStorageOperation::Save { .. }));
            assert_eq!(stage, WorkspaceStorageStage::OpenDatabase);
            assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
        }
        _ => panic!("expected original save cause"),
    }
}

#[wasm_bindgen_test]
async fn missing_records_and_malformed_falsy_records_remain_distinct() {
    let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(JsValue::UNDEFINED));
    fixture.dispatch_read();
    assert!(matches!(
        ProjectRoot::restore().await.unwrap(),
        WorkspaceRestoration::NoSavedRoot
    ));
    drop(fixture);
    for invalid in [
        JsValue::NULL,
        JsValue::FALSE,
        JsValue::from(0),
        JsValue::from(""),
    ] {
        let fixture = StorageFixture::new(StorageFixtureMode::LoadedRecord(invalid.clone()));
        fixture.dispatch_read();
        match ProjectRoot::restore().await.unwrap_err() {
            WorkspaceError::Storage(WorkspaceStorageError {
                stage,
                failure: WorkspaceStorageFailure::Provider(source),
                ..
            }) => {
                assert_eq!(stage, WorkspaceStorageStage::DecodeRecord);
                assert!(js_sys::Object::is(source.as_ref(), &invalid));
            }
            _ => panic!("expected malformed saved handle"),
        }
    }
}
