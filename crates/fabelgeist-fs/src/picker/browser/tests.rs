use super::*;
use crate::browser::test_handles::MockHandleConstructors;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_test::wasm_bindgen_test;

enum PickerFixture {
    Selection(JsValue),
    Throw(JsValue),
    Reject(JsValue),
    GetterThrow(JsValue),
    InvalidPromise(JsValue),
}

struct MockPicker {
    picker: BrowserPicker,
    kind: PickerKind,
    _method: Closure<dyn FnMut(JsValue) -> JsValue>,
}
impl MockPicker {
    fn new(kind: PickerKind, fixture: PickerFixture) -> Self {
        let getter = matches!(fixture, PickerFixture::GetterThrow(_));
        let method = Closure::wrap(Box::new(move |_options: JsValue| -> JsValue {
            match &fixture {
                PickerFixture::Selection(value) => js_sys::Promise::resolve(value).into(),
                PickerFixture::Reject(cause) => js_sys::Promise::reject(cause).into(),
                PickerFixture::Throw(cause) | PickerFixture::GetterThrow(cause) => {
                    wasm_bindgen::throw_val(cause.clone())
                }
                PickerFixture::InvalidPromise(value) => value.clone(),
            }
        }) as Box<dyn FnMut(JsValue) -> JsValue>);
        let receiver = js_sys::Object::new();
        let name = match kind {
            PickerKind::File => "showOpenFilePicker",
            PickerKind::Directory => "showDirectoryPicker",
        };
        if getter {
            let descriptor = js_sys::Object::new();
            js_sys::Reflect::set(&descriptor, &"get".into(), method.as_ref()).unwrap();
            js_sys::Object::define_property(&receiver, &name.into(), &descriptor);
        } else {
            js_sys::Reflect::set(&receiver, &name.into(), method.as_ref()).unwrap();
        }
        Self {
            picker: BrowserPicker(receiver),
            kind,
            _method: method,
        }
    }

    async fn error(&self) -> PickerError {
        match self.kind {
            PickerKind::File => self
                .picker
                .file(&FilePickerFilter::AnyFile)
                .await
                .unwrap_err(),
            PickerKind::Directory => self.picker.directory().await.unwrap_err(),
        }
    }
}

#[wasm_bindgen_test]
async fn node_without_a_window_reports_the_requested_picker_kind() {
    let error = FilePickerFilter::AnyFile.pick().await.unwrap_err();
    assert!(matches!(
        error,
        PickerError::MissingWindow {
            kind: PickerKind::File
        }
    ));
    let error = super::super::pick_folder_entry().await.unwrap_err();
    assert!(matches!(
        error,
        PickerError::MissingWindow {
            kind: PickerKind::Directory
        }
    ));
}

#[wasm_bindgen_test]
async fn synchronous_and_asynchronous_picker_failures_retain_kind_stage_and_cause() {
    let cause = js_sys::Error::new("selection rejected");
    for kind in [PickerKind::File, PickerKind::Directory] {
        for (fixture, expected) in [
            (
                PickerFixture::GetterThrow(cause.clone().into()),
                BrowserPickerStage::ReadMethod,
            ),
            (
                PickerFixture::Throw(cause.clone().into()),
                BrowserPickerStage::InvokeMethod,
            ),
            (
                PickerFixture::Reject(cause.clone().into()),
                BrowserPickerStage::AwaitPicker,
            ),
        ] {
            let mock = MockPicker::new(kind, fixture);
            let error = mock.error().await;
            assert!(std::error::Error::source(&error).is_some());
            match error {
                PickerError::Browser {
                    kind: actual,
                    stage,
                    source,
                } => {
                    assert_eq!(actual, kind);
                    assert_eq!(stage, expected);
                    assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
                }
                PickerError::MissingWindow { .. } => panic!("fixture has a picker receiver"),
            }
        }
    }
}

#[wasm_bindgen_test]
async fn absent_picker_methods_and_non_promises_are_structured_failures() {
    for kind in [PickerKind::File, PickerKind::Directory] {
        let mock = MockPicker::new(kind, PickerFixture::InvalidPromise(JsValue::NULL));
        match mock.error().await {
            PickerError::Browser {
                kind: actual,
                stage,
                source,
            } => {
                assert_eq!(actual, kind);
                assert_eq!(stage, BrowserPickerStage::DecodePromise);
                assert!(source.as_ref().is_null());
            }
            PickerError::MissingWindow { .. } => panic!("fixture has a picker receiver"),
        }
        let mock = MockPicker {
            picker: BrowserPicker(js_sys::Object::new()),
            ..mock
        };
        match mock.error().await {
            PickerError::Browser {
                kind: actual,
                stage,
                source,
            } => {
                assert_eq!(actual, kind);
                assert_eq!(stage, BrowserPickerStage::DecodeMethod);
                assert!(source.as_ref().is_undefined());
            }
            PickerError::MissingWindow { .. } => panic!("fixture has a picker receiver"),
        }
    }
}

#[wasm_bindgen_test]
async fn empty_file_results_remain_no_selection_and_extra_handles_are_rejected() {
    let handles = js_sys::Array::new();
    let mock = MockPicker::new(
        PickerKind::File,
        PickerFixture::Selection(handles.clone().into()),
    );
    assert!(
        mock.picker
            .file(&FilePickerFilter::AnyFile)
            .await
            .unwrap()
            .is_none()
    );
    handles.push(&JsValue::NULL);
    handles.push(&JsValue::NULL);
    match mock.error().await {
        PickerError::Browser {
            kind,
            stage,
            source,
        } => {
            assert_eq!(kind, PickerKind::File);
            assert_eq!(stage, BrowserPickerStage::FileCardinality);
            assert!(js_sys::Object::is(source.as_ref(), handles.as_ref()));
        }
        PickerError::MissingWindow { .. } => panic!("fixture has a picker receiver"),
    }
}

#[wasm_bindgen_test]
async fn malformed_arrays_and_handles_retain_the_provider_values() {
    let handles = js_sys::Array::new();
    handles.push(&JsValue::NULL);
    for (kind, value, expected) in [
        (
            PickerKind::File,
            JsValue::NULL,
            BrowserPickerStage::DecodeArray,
        ),
        (
            PickerKind::File,
            handles.into(),
            BrowserPickerStage::DecodeFileHandle,
        ),
        (
            PickerKind::Directory,
            JsValue::NULL,
            BrowserPickerStage::DecodeDirectoryHandle,
        ),
    ] {
        let mock = MockPicker::new(kind, PickerFixture::Selection(value.clone()));
        match mock.error().await {
            PickerError::Browser {
                kind: actual,
                stage,
                source,
            } => {
                assert_eq!(actual, kind);
                assert_eq!(stage, expected);
                assert!(source.as_ref().is_null());
            }
            PickerError::MissingWindow { .. } => panic!("fixture has a picker receiver"),
        }
    }
}

#[wasm_bindgen_test]
async fn selected_provider_handles_become_entry_authority() {
    let _constructors = MockHandleConstructors::new();
    let handle = js_sys::Object::new();
    js_sys::Reflect::set(&handle, &"name".into(), &"chosen".into()).unwrap();
    let handles = js_sys::Array::new();
    handles.push(&handle);
    let file = MockPicker::new(PickerKind::File, PickerFixture::Selection(handles.into()));
    assert_eq!(
        file.picker
            .file(&FilePickerFilter::AnyFile)
            .await
            .unwrap()
            .unwrap()
            .name(),
        crate::EntryLabel::from(String::from("chosen")),
    );
    let directory = MockPicker::new(
        PickerKind::Directory,
        PickerFixture::Selection(handle.into()),
    );
    assert_eq!(
        directory.picker.directory().await.unwrap().unwrap().name(),
        crate::EntryLabel::from(String::from("chosen")),
    );
}

#[wasm_bindgen_test]
async fn file_handle_property_exceptions_are_caught_after_array_admission() {
    let cause = js_sys::Error::new("handle getter failed");
    let getter_cause = cause.clone();
    let getter = Closure::wrap(Box::new(move || -> JsValue {
        wasm_bindgen::throw_val(getter_cause.clone().into())
    }) as Box<dyn FnMut() -> JsValue>);
    let descriptor = js_sys::Object::new();
    js_sys::Reflect::set(&descriptor, &"get".into(), getter.as_ref()).unwrap();
    let handles = js_sys::Array::new();
    js_sys::Object::define_property(&handles, &"0".into(), &descriptor);
    let mock = MockPicker::new(PickerKind::File, PickerFixture::Selection(handles.into()));
    match mock.error().await {
        PickerError::Browser { stage, source, .. } => {
            assert_eq!(stage, BrowserPickerStage::ReadFileHandle);
            assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
        }
        PickerError::MissingWindow { .. } => panic!("fixture has a picker receiver"),
    }
}

enum ArrayLengthFixture {
    Throw(JsValue),
    Malformed(JsValue),
}

#[wasm_bindgen_test]
async fn array_proxy_length_failures_retain_the_cause_or_malformed_length() {
    let cause = js_sys::Error::new("array length failed");
    for (fixture, expected, original) in [
        (
            ArrayLengthFixture::Throw(cause.clone().into()),
            BrowserPickerStage::ReadArrayLength,
            JsValue::from(cause),
        ),
        (
            ArrayLengthFixture::Malformed(JsValue::from("1")),
            BrowserPickerStage::DecodeArrayLength,
            JsValue::from("1"),
        ),
    ] {
        let length_key = JsValue::from("length");
        let getter = Closure::wrap(Box::new(
            move |target: JsValue, key: JsValue, _receiver: JsValue| -> JsValue {
                if key == length_key {
                    match &fixture {
                        ArrayLengthFixture::Throw(cause) => wasm_bindgen::throw_val(cause.clone()),
                        ArrayLengthFixture::Malformed(value) => value.clone(),
                    }
                } else {
                    js_sys::Reflect::get(&target, &key).unwrap()
                }
            },
        )
            as Box<dyn FnMut(JsValue, JsValue, JsValue) -> JsValue>);
        let handler = js_sys::Object::new();
        js_sys::Reflect::set(&handler, &"get".into(), getter.as_ref()).unwrap();
        let value = js_sys::Proxy::new(&js_sys::Array::new(), &handler);
        let mock = MockPicker::new(PickerKind::File, PickerFixture::Selection(value.into()));
        match mock.error().await {
            PickerError::Browser { stage, source, .. } => {
                assert_eq!(stage, expected);
                assert!(js_sys::Object::is(source.as_ref(), &original));
            }
            PickerError::MissingWindow { .. } => panic!("fixture has a picker receiver"),
        }
    }
}

#[wasm_bindgen_test]
async fn abort_rejection_is_preserved_without_claiming_a_successful_empty_selection() {
    let cause = js_sys::Error::new("provider aborted selection");
    js_sys::Reflect::set(&cause, &"name".into(), &"AbortError".into()).unwrap();
    let mock = MockPicker::new(
        PickerKind::File,
        PickerFixture::Reject(cause.clone().into()),
    );
    match mock.error().await {
        PickerError::Browser { stage, source, .. } => {
            assert_eq!(stage, BrowserPickerStage::AwaitPicker);
            assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
        }
        PickerError::MissingWindow { .. } => panic!("fixture has a picker receiver"),
    }
}
