use super::*;
use crate::browser::test_handles::MockHandleConstructors;
use crate::{DirectoryEntry, ProjectRoot, WorkspaceRestoration};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::closure::Closure;
use wasm_bindgen_test::wasm_bindgen_test;

#[derive(Clone)]
enum PermissionFixtureMode {
    Status(WorkspacePermission),
    MalformedStatus(JsValue),
    Throw(JsValue),
    Reject(JsValue),
    GetterThrow(JsValue),
    InvalidPromise(JsValue),
    MissingMethod,
}

struct PermissionFixture {
    directory: WebDirectory,
    calls: Rc<RefCell<Vec<WorkspacePermissionOperation>>>,
    _methods: Vec<Closure<dyn FnMut(JsValue) -> JsValue>>,
}
impl PermissionFixture {
    fn new(query: PermissionFixtureMode, request: PermissionFixtureMode) -> Self {
        let receiver = js_sys::Object::new();
        js_sys::Reflect::set(&receiver, &"name".into(), &"restored-root".into()).unwrap();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut methods = Vec::new();
        for (operation, mode) in [
            (WorkspacePermissionOperation::Query, query),
            (WorkspacePermissionOperation::Request, request),
        ] {
            let getter = matches!(mode, PermissionFixtureMode::GetterThrow(_));
            let missing = matches!(mode, PermissionFixtureMode::MissingMethod);
            let recorded = calls.clone();
            let method = Closure::wrap(Box::new(move |options: JsValue| -> JsValue {
                recorded.borrow_mut().push(operation);
                if let PermissionFixtureMode::GetterThrow(cause) = &mode {
                    wasm_bindgen::throw_val(cause.clone());
                }
                assert_eq!(
                    js_sys::Reflect::get(&options, &"mode".into()).unwrap(),
                    JsValue::from("read"),
                );
                match &mode {
                    PermissionFixtureMode::Status(permission) => {
                        let status = match permission {
                            WorkspacePermission::Granted => "granted",
                            WorkspacePermission::Restricted(WorkspaceAccessRestriction::Prompt) => {
                                "prompt"
                            }
                            WorkspacePermission::Restricted(WorkspaceAccessRestriction::Denied) => {
                                "denied"
                            }
                        };
                        js_sys::Promise::resolve(&status.into()).into()
                    }
                    PermissionFixtureMode::MalformedStatus(value) => {
                        js_sys::Promise::resolve(value).into()
                    }
                    PermissionFixtureMode::Throw(cause) => wasm_bindgen::throw_val(cause.clone()),
                    PermissionFixtureMode::Reject(cause) => js_sys::Promise::reject(cause).into(),
                    PermissionFixtureMode::InvalidPromise(value) => value.clone(),
                    PermissionFixtureMode::GetterThrow(_)
                    | PermissionFixtureMode::MissingMethod => {
                        panic!("fixture cannot invoke this method")
                    }
                }
            }) as Box<dyn FnMut(JsValue) -> JsValue>);
            let name = match operation {
                WorkspacePermissionOperation::Query => "queryPermission",
                WorkspacePermissionOperation::Request => "requestPermission",
            };
            if getter {
                let descriptor = js_sys::Object::new();
                js_sys::Reflect::set(&descriptor, &"get".into(), method.as_ref()).unwrap();
                js_sys::Object::define_property(&receiver, &name.into(), &descriptor);
            } else if !missing {
                js_sys::Reflect::set(&receiver, &name.into(), method.as_ref()).unwrap();
            }
            methods.push(method);
        }
        Self {
            directory: WebDirectory::from(
                receiver.unchecked_into::<web_sys::FileSystemDirectoryHandle>(),
            ),
            calls,
            _methods: methods,
        }
    }
}

#[wasm_bindgen_test]
fn permission_admission_retains_closed_states_operation_and_directory_context() {
    let fixture = PermissionFixture::new(
        PermissionFixtureMode::Status(WorkspacePermission::Granted),
        PermissionFixtureMode::Status(WorkspacePermission::Granted),
    );
    for operation in [
        WorkspacePermissionOperation::Query,
        WorkspacePermissionOperation::Request,
    ] {
        for (wire, permission) in [
            ("granted", WorkspacePermission::Granted),
            (
                "prompt",
                WorkspacePermission::Restricted(WorkspaceAccessRestriction::Prompt),
            ),
            (
                "denied",
                WorkspacePermission::Restricted(WorkspaceAccessRestriction::Denied),
            ),
        ] {
            assert_eq!(
                WorkspacePermission::from_browser(&fixture.directory, operation, wire.into())
                    .unwrap(),
                permission,
            );
        }
        let invalid = js_sys::Object::new();
        match WorkspacePermission::from_browser(
            &fixture.directory,
            operation,
            invalid.clone().into(),
        )
        .unwrap_err()
        {
            WorkspaceError::Permission {
                directory,
                operation: actual,
                stage,
                source,
            } => {
                assert_eq!(actual, operation);
                assert_eq!(stage, WorkspacePermissionStage::DecodeResult);
                assert!(js_sys::Object::is(
                    directory.as_ref(),
                    fixture.directory.as_ref()
                ));
                assert!(js_sys::Object::is(source.as_ref(), invalid.as_ref()));
            }
            _ => panic!("expected malformed permission admission"),
        }
    }
}

#[wasm_bindgen_test]
async fn granted_queries_skip_requests_and_restricted_queries_admit_the_request_status() {
    let fixture = PermissionFixture::new(
        PermissionFixtureMode::Status(WorkspacePermission::Granted),
        PermissionFixtureMode::Throw(js_sys::Error::new("must not request").into()),
    );
    assert_eq!(
        fixture.directory.restore_permission().await.unwrap(),
        WorkspacePermission::Granted
    );
    assert_eq!(
        fixture.calls.borrow().as_slice(),
        [WorkspacePermissionOperation::Query]
    );

    for restriction in [
        WorkspaceAccessRestriction::Prompt,
        WorkspaceAccessRestriction::Denied,
    ] {
        for result in [
            WorkspacePermission::Granted,
            WorkspacePermission::Restricted(WorkspaceAccessRestriction::Prompt),
            WorkspacePermission::Restricted(WorkspaceAccessRestriction::Denied),
        ] {
            let fixture = PermissionFixture::new(
                PermissionFixtureMode::Status(WorkspacePermission::Restricted(restriction)),
                PermissionFixtureMode::Status(result),
            );
            assert_eq!(
                fixture.directory.restore_permission().await.unwrap(),
                result
            );
            assert_eq!(
                fixture.calls.borrow().as_slice(),
                [
                    WorkspacePermissionOperation::Query,
                    WorkspacePermissionOperation::Request,
                ]
            );
        }
    }
}

#[wasm_bindgen_test]
async fn malformed_query_results_fail_before_requesting_permission() {
    for value in [
        JsValue::NULL,
        JsValue::FALSE,
        JsValue::from(0),
        JsValue::from("unknown"),
        JsValue::from(js_sys::Object::new()),
    ] {
        let fixture = PermissionFixture::new(
            PermissionFixtureMode::MalformedStatus(value.clone()),
            PermissionFixtureMode::Status(WorkspacePermission::Granted),
        );
        match fixture.directory.restore_permission().await.unwrap_err() {
            WorkspaceError::Permission {
                operation,
                stage,
                source,
                ..
            } => {
                assert_eq!(operation, WorkspacePermissionOperation::Query);
                assert_eq!(stage, WorkspacePermissionStage::DecodeResult);
                assert!(js_sys::Object::is(source.as_ref(), &value));
            }
            _ => panic!("expected query admission failure"),
        }
        assert_eq!(
            fixture.calls.borrow().as_slice(),
            [WorkspacePermissionOperation::Query]
        );
    }
}

#[wasm_bindgen_test]
async fn query_and_request_failures_retain_their_original_provider_cause_and_stage() {
    let cause = js_sys::Error::new("permission provider failed");
    for operation in [
        WorkspacePermissionOperation::Query,
        WorkspacePermissionOperation::Request,
    ] {
        for (mode, expected, original) in [
            (
                PermissionFixtureMode::GetterThrow(cause.clone().into()),
                WorkspacePermissionStage::ReadMethod,
                JsValue::from(cause.clone()),
            ),
            (
                PermissionFixtureMode::Throw(cause.clone().into()),
                WorkspacePermissionStage::InvokeMethod,
                JsValue::from(cause.clone()),
            ),
            (
                PermissionFixtureMode::Reject(cause.clone().into()),
                WorkspacePermissionStage::AwaitResult,
                JsValue::from(cause.clone()),
            ),
            (
                PermissionFixtureMode::InvalidPromise(JsValue::NULL),
                WorkspacePermissionStage::DecodePromise,
                JsValue::NULL,
            ),
            (
                PermissionFixtureMode::MissingMethod,
                WorkspacePermissionStage::DecodeMethod,
                JsValue::UNDEFINED,
            ),
        ] {
            let (query, request) = match operation {
                WorkspacePermissionOperation::Query => (
                    mode,
                    PermissionFixtureMode::Status(WorkspacePermission::Granted),
                ),
                WorkspacePermissionOperation::Request => (
                    PermissionFixtureMode::Status(WorkspacePermission::Restricted(
                        WorkspaceAccessRestriction::Prompt,
                    )),
                    mode,
                ),
            };
            let fixture = PermissionFixture::new(query, request);
            let error = fixture.directory.restore_permission().await.unwrap_err();
            assert!(std::error::Error::source(&error).is_some());
            match error {
                WorkspaceError::Permission {
                    directory,
                    operation: actual,
                    stage,
                    source,
                } => {
                    assert_eq!(actual, operation);
                    assert_eq!(stage, expected);
                    assert!(js_sys::Object::is(
                        directory.as_ref(),
                        fixture.directory.as_ref()
                    ));
                    assert!(js_sys::Object::is(source.as_ref(), &original));
                }
                _ => panic!("expected caught permission failure"),
            }
        }
    }
}

#[wasm_bindgen_test]
async fn malformed_requested_statuses_do_not_become_restored_roots() {
    let invalid = JsValue::from("unexpected");
    let fixture = PermissionFixture::new(
        PermissionFixtureMode::Status(WorkspacePermission::Restricted(
            WorkspaceAccessRestriction::Prompt,
        )),
        PermissionFixtureMode::MalformedStatus(invalid.clone()),
    );
    match fixture.directory.restore_permission().await.unwrap_err() {
        WorkspaceError::Permission {
            operation,
            stage,
            source,
            ..
        } => {
            assert_eq!(operation, WorkspacePermissionOperation::Request);
            assert_eq!(stage, WorkspacePermissionStage::DecodeResult);
            assert_eq!(source.as_ref(), &invalid);
        }
        _ => panic!("expected request admission failure"),
    }
}

#[wasm_bindgen_test]
async fn persisted_root_restoration_preserves_only_real_access_restrictions() {
    use super::super::tests::{StorageFixture, StorageFixtureMode};
    let _constructors = MockHandleConstructors::new();
    for result in [
        WorkspacePermission::Granted,
        WorkspacePermission::Restricted(WorkspaceAccessRestriction::Prompt),
        WorkspacePermission::Restricted(WorkspaceAccessRestriction::Denied),
    ] {
        let permissions = PermissionFixture::new(
            PermissionFixtureMode::Status(WorkspacePermission::Restricted(
                WorkspaceAccessRestriction::Prompt,
            )),
            PermissionFixtureMode::Status(result),
        );
        let storage = StorageFixture::new(StorageFixtureMode::LoadedRecord(
            permissions.directory.as_ref().clone().into(),
        ));
        storage.dispatch_read();
        match (result, ProjectRoot::restore().await.unwrap()) {
            (WorkspacePermission::Granted, WorkspaceRestoration::Restored(root)) => {
                assert_eq!(root.directory().name(), permissions.directory.name());
            }
            (
                WorkspacePermission::Restricted(expected),
                WorkspaceRestoration::PermissionNotGranted {
                    directory,
                    restriction,
                },
            ) => {
                assert_eq!(restriction, expected);
                assert!(js_sys::Object::is(
                    directory.as_ref(),
                    permissions.directory.as_ref()
                ));
            }
            _ => panic!("restoration must retain the admitted permission state"),
        }
    }
}
