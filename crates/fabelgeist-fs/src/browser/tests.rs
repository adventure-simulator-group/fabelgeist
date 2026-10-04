use super::*;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::closure::Closure;
use wasm_bindgen_test::wasm_bindgen_test;

enum MockLookupBehavior {
    Resolve,
    Reject(JsValue),
    Throw(JsValue),
    InvalidPromise,
}

struct MockDirectory {
    directory: WebDirectory,
    _method: Closure<dyn FnMut(JsValue, JsValue) -> JsValue>,
    observed: Rc<RefCell<Option<(JsValue, JsValue)>>>,
}

impl MockDirectory {
    fn new(behavior: MockLookupBehavior) -> Self {
        let observed = Rc::new(RefCell::new(None));
        let capture = observed.clone();
        let method = Closure::wrap(Box::new(move |name: JsValue, options: JsValue| -> JsValue {
            *capture.borrow_mut() = Some((name, options));
            match &behavior {
                MockLookupBehavior::Resolve => js_sys::Promise::resolve(&JsValue::UNDEFINED).into(),
                MockLookupBehavior::Reject(cause) => js_sys::Promise::reject(cause).into(),
                MockLookupBehavior::Throw(cause) => wasm_bindgen::throw_val(cause.clone()),
                MockLookupBehavior::InvalidPromise => JsValue::NULL,
            }
        }) as Box<dyn FnMut(JsValue, JsValue) -> JsValue>);
        let handle = js_sys::Object::new();
        js_sys::Reflect::set(&handle, &"removeEntry".into(), method.as_ref()).unwrap();
        js_sys::Reflect::set(&handle, &"getFileHandle".into(), method.as_ref()).unwrap();
        js_sys::Reflect::set(&handle, &"getDirectoryHandle".into(), method.as_ref()).unwrap();
        Self {
            directory: WebDirectory::from(
                handle.unchecked_into::<web_sys::FileSystemDirectoryHandle>(),
            ),
            _method: method,
            observed,
        }
    }
}

#[wasm_bindgen_test]
async fn browser_protocol_receives_the_admitted_name_and_creation_intent() {
    let name = EntryName::try_from("résumé.glb").unwrap();
    for kind in [EntryLookupKind::File, EntryLookupKind::Directory] {
        for intent in [
            EntryLookupIntent::Existing,
            EntryLookupIntent::CreateIfMissing,
        ] {
            let mock = MockDirectory::new(MockLookupBehavior::Resolve);
            let promise = mock
                .directory
                .request(&name, EntryRequest::Lookup { intent, kind })
                .unwrap();
            wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
            let observed = mock.observed.borrow();
            let (received_name, options) = observed.as_ref().unwrap();
            assert_eq!(received_name.as_string().unwrap(), name.as_ref());
            assert_eq!(
                js_sys::Reflect::get(options, &"create".into())
                    .unwrap()
                    .as_bool(),
                Some(intent == EntryLookupIntent::CreateIfMissing)
            );
        }
    }
}

#[wasm_bindgen_test]
async fn asynchronous_rejection_retains_the_original_exception_and_operation() {
    let cause = js_sys::Error::new("lookup rejected");
    let name = EntryName::try_from("file").unwrap();
    let mock = MockDirectory::new(MockLookupBehavior::Reject(cause.clone().into()));
    let failure = mock
        .directory
        .get_file(&name, EntryLookupIntent::Existing)
        .await
        .unwrap_err();
    match failure {
        EntryAccessError::Browser {
            operation,
            entry,
            stage,
            source,
        } => {
            assert_eq!(operation, EntryOperation::LookupFile);
            assert_eq!(entry, name);
            assert_eq!(stage, BrowserEntryStage::AwaitHandle);
            assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
        }
    }
}

#[wasm_bindgen_test]
fn synchronous_rejection_and_invalid_promise_are_distinct() {
    let cause = js_sys::Error::new("synchronous rejection");
    let name = EntryName::try_from("directory").unwrap();
    let mock = MockDirectory::new(MockLookupBehavior::Throw(cause.clone().into()));
    let failure = mock
        .directory
        .request(
            &name,
            EntryRequest::Lookup {
                intent: EntryLookupIntent::Existing,
                kind: EntryLookupKind::Directory,
            },
        )
        .unwrap_err();
    match failure {
        EntryAccessError::Browser {
            operation,
            entry,
            stage,
            source,
        } => {
            assert_eq!(operation, EntryOperation::LookupDirectory);
            assert_eq!(entry, name);
            assert_eq!(stage, BrowserEntryStage::InvokeMethod);
            assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
        }
    }
    let mock = MockDirectory::new(MockLookupBehavior::InvalidPromise);
    let failure = mock
        .directory
        .request(
            &name,
            EntryRequest::Lookup {
                intent: EntryLookupIntent::Existing,
                kind: EntryLookupKind::Directory,
            },
        )
        .unwrap_err();
    assert!(matches!(
        failure,
        EntryAccessError::Browser {
            stage: BrowserEntryStage::DecodePromise,
            ..
        }
    ));
}

#[wasm_bindgen_test]
async fn deletion_preserves_synchronous_and_asynchronous_provider_causes() {
    let cause = js_sys::Error::new("delete rejected");
    let name = EntryName::try_from("child").unwrap();
    for (behavior, expected) in [
        (
            MockLookupBehavior::Throw(cause.clone().into()),
            BrowserEntryStage::InvokeMethod,
        ),
        (
            MockLookupBehavior::Reject(cause.clone().into()),
            BrowserEntryStage::Remove,
        ),
    ] {
        let mock = MockDirectory::new(behavior);
        let failure = mock.directory.delete_entry(&name).await.unwrap_err();
        match failure {
            EntryAccessError::Browser {
                operation,
                entry,
                stage,
                source,
            } => {
                assert_eq!(operation, EntryOperation::Delete);
                assert_eq!(entry, name);
                assert_eq!(stage, expected);
                assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
            }
        }
    }
}
