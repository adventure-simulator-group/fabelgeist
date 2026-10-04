use super::{StorageEvent, StorageFixtureMode};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsValue, closure::Closure};

pub(super) struct StorageRecordFixture {
    pub(super) request: js_sys::Object,
    pub(super) transaction: js_sys::Object,
    _get: Closure<dyn FnMut(JsValue) -> JsValue>,
    _put: Closure<dyn FnMut(JsValue, JsValue) -> JsValue>,
    _store: Closure<dyn FnMut(JsValue) -> JsValue>,
    _abort: Closure<dyn FnMut()>,
}
impl StorageRecordFixture {
    pub(super) fn new(mode: &StorageFixtureMode, events: &Rc<RefCell<Vec<StorageEvent>>>) -> Self {
        let read_request = js_sys::Object::new();
        if let StorageFixtureMode::LoadedRecord(value) = &mode {
            js_sys::Reflect::set(&read_request, &"result".into(), value).unwrap();
        }
        let get_request = read_request.clone();
        let get_mode = mode.clone();
        let get_events = events.clone();
        let get = Closure::wrap(Box::new(move |key: JsValue| -> JsValue {
            assert_eq!(key, JsValue::from("project_root"));
            get_events.borrow_mut().push(StorageEvent::Read);
            if let StorageFixtureMode::ReadFailure(cause) = &get_mode {
                wasm_bindgen::throw_val(cause.clone());
            }
            get_request.clone().into()
        }) as Box<dyn FnMut(JsValue) -> JsValue>);
        let store = js_sys::Object::new();
        js_sys::Reflect::set(&store, &"get".into(), get.as_ref()).unwrap();
        let put_request = read_request.clone();
        let put_mode = mode.clone();
        let put_events = events.clone();
        let put = Closure::wrap(Box::new(move |_handle: JsValue, key: JsValue| -> JsValue {
            assert_eq!(key, JsValue::from("project_root"));
            put_events.borrow_mut().push(StorageEvent::Write);
            if let StorageFixtureMode::WriteFailure(cause) = &put_mode {
                wasm_bindgen::throw_val(cause.clone());
            }
            put_request.clone().into()
        }) as Box<dyn FnMut(JsValue, JsValue) -> JsValue>);
        js_sys::Reflect::set(&store, &"put".into(), put.as_ref()).unwrap();
        let store_mode = mode.clone();
        let object_store = Closure::wrap(Box::new(move |name: JsValue| -> JsValue {
            assert_eq!(name, JsValue::from("handles"));
            if let StorageFixtureMode::StoreFailure(cause) = &store_mode {
                wasm_bindgen::throw_val(cause.clone());
            }
            store.clone().into()
        }) as Box<dyn FnMut(JsValue) -> JsValue>);
        let transaction = js_sys::Object::new();
        js_sys::Reflect::set(&transaction, &"objectStore".into(), object_store.as_ref()).unwrap();
        let abort_events = events.clone();
        let abort = Closure::wrap(Box::new(move || -> () {
            abort_events.borrow_mut().push(StorageEvent::Aborted);
        }) as Box<dyn FnMut()>);
        js_sys::Reflect::set(&transaction, &"abort".into(), abort.as_ref()).unwrap();
        Self {
            request: read_request,
            transaction,
            _get: get,
            _put: put,
            _store: object_store,
            _abort: abort,
        }
    }
}
