use super::*;
use crate::FileOperation;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsValue, closure::Closure};
use wasm_bindgen_test::wasm_bindgen_test;

enum FileResponse {
    Resolve(JsValue),
    Reject(JsValue),
    Throw(JsValue),
    InvalidPromise,
}
struct MockFile {
    file: WebFile,
    _method: Closure<dyn FnMut() -> JsValue>,
}
impl MockFile {
    fn new(response: FileResponse) -> Self {
        let method = Closure::wrap(Box::new(move || -> JsValue {
            match &response {
                FileResponse::Resolve(value) => js_sys::Promise::resolve(value).into(),
                FileResponse::Reject(cause) => js_sys::Promise::reject(cause).into(),
                FileResponse::Throw(cause) => wasm_bindgen::throw_val(cause.clone()),
                FileResponse::InvalidPromise => JsValue::NULL,
            }
        }) as Box<dyn FnMut() -> JsValue>);
        let handle = js_sys::Object::new();
        js_sys::Reflect::set(&handle, &"getFile".into(), method.as_ref()).unwrap();
        js_sys::Reflect::set(&handle, &"createWritable".into(), method.as_ref()).unwrap();
        Self {
            file: WebFile::from(handle.unchecked_into::<web_sys::FileSystemFileHandle>()),
            _method: method,
        }
    }
}

#[wasm_bindgen_test]
async fn file_read_preserves_all_binary_values() {
    let contents = FileContents::from(vec![0, 255, 128, 1]);
    let sequence = js_sys::Array::new();
    sequence.push(&js_sys::Uint8Array::from(contents.as_ref()));
    let file = web_sys::File::new_with_u8_array_sequence(&sequence, "payload.bin").unwrap();
    let mock = MockFile::new(FileResponse::Resolve(file.into()));
    assert_eq!(mock.file.read().await.unwrap(), contents);
}

#[wasm_bindgen_test]
async fn file_read_distinguishes_synchronous_rejection_and_async_rejection() {
    let cause = js_sys::Error::new("file rejected");
    for (response, expected) in [
        (
            FileResponse::Throw(cause.clone().into()),
            BrowserContentStage::InvokeMethod,
        ),
        (
            FileResponse::Reject(cause.clone().into()),
            BrowserContentStage::AwaitFile,
        ),
    ] {
        let mock = MockFile::new(response);
        let failure = mock.file.read().await.unwrap_err();
        assert_eq!(failure.operation(), FileOperation::Read);
        match failure {
            FileContentError::Browser { stage, source, .. } => {
                assert_eq!(stage, expected);
                assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
            }
        }
    }
    let failure = MockFile::new(FileResponse::InvalidPromise)
        .file
        .read()
        .await
        .unwrap_err();
    assert!(matches!(
        failure,
        FileContentError::Browser {
            stage: BrowserContentStage::DecodePromise,
            ..
        }
    ));
}

#[derive(Default)]
struct WriteReceipt {
    contents: Option<FileContents>,
    closed: bool,
}
struct MockStream {
    stream: js_sys::Object,
    receipt: Rc<RefCell<WriteReceipt>>,
    _write: Closure<dyn FnMut(JsValue) -> JsValue>,
    _close: Closure<dyn FnMut() -> JsValue>,
}
enum CloseResponse {
    Resolve,
    Reject(JsValue),
    Throw(JsValue),
}
impl MockStream {
    fn new(response: CloseResponse) -> Self {
        let receipt = Rc::new(RefCell::new(WriteReceipt::default()));
        let observe = receipt.clone();
        let write = Closure::wrap(Box::new(move |packet: JsValue| -> JsValue {
            observe.borrow_mut().contents = Some(FileContents::from(
                js_sys::Uint8Array::new(&packet).to_vec(),
            ));
            js_sys::Promise::resolve(&JsValue::UNDEFINED).into()
        }) as Box<dyn FnMut(JsValue) -> JsValue>);
        let observe = receipt.clone();
        let close = Closure::wrap(Box::new(move || -> JsValue {
            assert!(observe.borrow().contents.is_some());
            observe.borrow_mut().closed = true;
            match &response {
                CloseResponse::Resolve => js_sys::Promise::resolve(&JsValue::UNDEFINED).into(),
                CloseResponse::Reject(cause) => js_sys::Promise::reject(cause).into(),
                CloseResponse::Throw(cause) => wasm_bindgen::throw_val(cause.clone()),
            }
        }) as Box<dyn FnMut() -> JsValue>);
        let stream = js_sys::Object::new();
        js_sys::Reflect::set(&stream, &"write".into(), write.as_ref()).unwrap();
        js_sys::Reflect::set(&stream, &"close".into(), close.as_ref()).unwrap();
        Self {
            stream,
            receipt,
            _write: write,
            _close: close,
        }
    }
}

#[wasm_bindgen_test]
async fn file_write_serializes_exact_bytes_and_closes_after_writing() {
    let stream = MockStream::new(CloseResponse::Resolve);
    let file = MockFile::new(FileResponse::Resolve(stream.stream.clone().into()));
    let contents = FileContents::from(vec![0, 255, 128, 1]);
    file.file.write(&contents).await.unwrap();
    let receipt = stream.receipt.borrow();
    assert_eq!(receipt.contents.as_ref(), Some(&contents));
    assert!(receipt.closed);
}

#[wasm_bindgen_test]
async fn file_write_retains_the_acquisition_failure_and_operation() {
    let cause = js_sys::Error::new("write permission denied");
    let file = MockFile::new(FileResponse::Reject(cause.clone().into()));
    let failure = file.file.write(&FileContents::default()).await.unwrap_err();
    assert_eq!(failure.operation(), FileOperation::Write);
    match failure {
        FileContentError::Browser { stage, source, .. } => {
            assert_eq!(stage, BrowserContentStage::AwaitWritable);
            assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
        }
    }
}

#[wasm_bindgen_test]
async fn close_failures_retain_the_exact_method_stage_and_cause() {
    let cause = js_sys::Error::new("close rejected");
    for (response, expected) in [
        (
            CloseResponse::Reject(cause.clone().into()),
            BrowserContentStage::Close,
        ),
        (
            CloseResponse::Throw(cause.clone().into()),
            BrowserContentStage::InvokeMethod,
        ),
    ] {
        let stream = MockStream::new(response);
        let file = MockFile::new(FileResponse::Resolve(stream.stream.clone().into()));
        let failure = file.file.write(&FileContents::default()).await.unwrap_err();
        assert_eq!(failure.operation(), FileOperation::Write);
        match failure {
            FileContentError::Browser {
                method,
                stage,
                source,
                ..
            } => {
                assert_eq!(method, BrowserContentMethod::Close);
                assert_eq!(stage, expected);
                assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
            }
        }
    }
}
