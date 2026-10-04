use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

mod record;
use record::StorageRecordFixture;

#[derive(Clone)]
pub(in crate::workspace::browser) enum StorageFixtureMode {
    OpenFailure(JsValue),
    TransactionFailure(JsValue),
    LoadedRecord(JsValue),
    StoreFailure(JsValue),
    ReadFailure(JsValue),
    WriteFailure(JsValue),
    UpgradeFailure(JsValue),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StorageEvent {
    Closed,
    Aborted,
    StoreCreated,
    Read,
    Write,
}
struct MockStorageConstructors {
    factory: JsValue,
    database: JsValue,
}
impl MockStorageConstructors {
    fn new() -> Self {
        let global = js_sys::global();
        let factory = js_sys::Reflect::get(&global, &"IDBFactory".into()).unwrap();
        let database = js_sys::Reflect::get(&global, &"IDBDatabase".into()).unwrap();
        let constructor = js_sys::Reflect::get(&global, &"Object".into()).unwrap();
        js_sys::Reflect::set(&global, &"IDBFactory".into(), &constructor).unwrap();
        js_sys::Reflect::set(&global, &"IDBDatabase".into(), &constructor).unwrap();
        Self { factory, database }
    }
}
impl Drop for MockStorageConstructors {
    fn drop(&mut self) {
        let global = js_sys::global();
        js_sys::Reflect::set(&global, &"IDBFactory".into(), &self.factory).unwrap();
        js_sys::Reflect::set(&global, &"IDBDatabase".into(), &self.database).unwrap();
    }
}
pub(in crate::workspace::browser) struct StorageFixture {
    previous: JsValue,
    _constructors: MockStorageConstructors,
    pub(super) events: Rc<RefCell<Vec<StorageEvent>>>,
    pub(super) request: js_sys::Object,
    pub(super) read_request: js_sys::Object,
    pub(super) transaction: js_sys::Object,
    _open: Closure<dyn FnMut(JsValue, JsValue) -> JsValue>,
    _transaction: Closure<dyn FnMut(JsValue, JsValue) -> JsValue>,
    _close: Closure<dyn FnMut()>,
    _create_store: Closure<dyn FnMut(JsValue) -> JsValue>,
    _records: StorageRecordFixture,
}
impl StorageFixture {
    pub(in crate::workspace::browser) fn new(mode: StorageFixtureMode) -> Self {
        let constructors = MockStorageConstructors::new();
        let events = Rc::new(RefCell::new(Vec::new()));
        let global = js_sys::global();
        let previous = js_sys::Reflect::get(&global, &"indexedDB".into()).unwrap();
        let request = js_sys::Object::new();
        let records = StorageRecordFixture::new(&mode, &events);
        let read_request = records.request.clone();
        let transaction = records.transaction.clone();
        let returned_transaction = transaction.clone();
        let transaction_mode = mode.clone();
        let transaction_method =
            Closure::wrap(Box::new(move |name: JsValue, access: JsValue| -> JsValue {
                assert_eq!(name, JsValue::from("handles"));
                assert!(matches!(
                    access.as_string().as_deref(),
                    Some("readonly" | "readwrite")
                ));
                match &transaction_mode {
                    StorageFixtureMode::TransactionFailure(cause) => {
                        wasm_bindgen::throw_val(cause.clone())
                    }
                    _ => returned_transaction.clone().into(),
                }
            })
                as Box<dyn FnMut(JsValue, JsValue) -> JsValue>);
        let database = js_sys::Object::new();
        js_sys::Reflect::set(
            &database,
            &"transaction".into(),
            transaction_method.as_ref(),
        )
        .unwrap();
        let close_events = events.clone();
        let close = Closure::wrap(Box::new(move || -> () {
            close_events.borrow_mut().push(StorageEvent::Closed);
        }) as Box<dyn FnMut()>);
        let create_mode = mode.clone();
        let create_events = events.clone();
        let create_store = Closure::wrap(Box::new(move |name: JsValue| -> JsValue {
            assert_eq!(name, JsValue::from("handles"));
            create_events.borrow_mut().push(StorageEvent::StoreCreated);
            if let StorageFixtureMode::UpgradeFailure(cause) = &create_mode {
                wasm_bindgen::throw_val(cause.clone());
            }
            js_sys::Object::new().into()
        }) as Box<dyn FnMut(JsValue) -> JsValue>);
        js_sys::Reflect::set(
            &database,
            &"createObjectStore".into(),
            create_store.as_ref(),
        )
        .unwrap();
        js_sys::Reflect::set(&request, &"result".into(), &database).unwrap();
        js_sys::Reflect::set(&request, &"transaction".into(), &transaction).unwrap();
        js_sys::Reflect::set(&database, &"close".into(), close.as_ref()).unwrap();
        let returned_request = request.clone();
        let open = Closure::wrap(Box::new(move |name: JsValue, version: JsValue| -> JsValue {
            assert_eq!(name, JsValue::from("fabelgeist_fs"));
            assert_eq!(version, JsValue::from(1));
            match &mode {
                StorageFixtureMode::OpenFailure(cause) => wasm_bindgen::throw_val(cause.clone()),
                _ => returned_request.clone().into(),
            }
        }) as Box<dyn FnMut(JsValue, JsValue) -> JsValue>);
        let indexed_db = js_sys::Object::new();
        js_sys::Reflect::set(&indexed_db, &"open".into(), open.as_ref()).unwrap();
        js_sys::Reflect::set(&global, &"indexedDB".into(), &indexed_db).unwrap();
        Self {
            previous,
            _constructors: constructors,
            events,
            request,
            read_request,
            transaction,
            _open: open,
            _transaction: transaction_method,
            _close: close,
            _create_store: create_store,
            _records: records,
        }
    }
    pub(in crate::workspace::browser) fn dispatch_read(&self) {
        let request = self.request.clone();
        let read_request = self.read_request.clone();
        let transaction = self.transaction.clone();
        wasm_bindgen_futures::spawn_local(async move {
            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
                .await
                .unwrap();
            let success = js_sys::Reflect::get(&request, &"onsuccess".into())
                .unwrap()
                .dyn_into::<js_sys::Function>()
                .unwrap();
            success.call0(&request).unwrap();
            // Opening wakes the Rust driver in a microtask. Start the request
            // after it has attached the transaction and record listeners.
            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
                .await
                .unwrap();
            let read = js_sys::Reflect::get(&read_request, &"onsuccess".into()).unwrap();
            if let Ok(read) = read.dyn_into::<js_sys::Function>() {
                read.call0(&read_request).unwrap();
                let complete = js_sys::Reflect::get(&transaction, &"oncomplete".into())
                    .unwrap()
                    .dyn_into::<js_sys::Function>()
                    .unwrap();
                complete.call0(&transaction).unwrap();
            }
        });
    }
}
impl Drop for StorageFixture {
    fn drop(&mut self) {
        js_sys::Reflect::set(&js_sys::global(), &"indexedDB".into(), &self.previous).unwrap();
    }
}
