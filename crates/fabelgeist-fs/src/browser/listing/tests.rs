use super::*;
use crate::DirectoryEntry;
use crate::browser::test_handles::MockHandleConstructors;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_test::wasm_bindgen_test;

#[derive(Clone)]
enum ListingFixture {
    Results(js_sys::Array),
    ValuesThrow(JsValue),
    InvalidIterator(JsValue),
    NextThrow(JsValue),
    NextReject(JsValue),
    InvalidPromise(JsValue),
}

struct MockListing {
    directory: WebDirectory,
    _values: Closure<dyn FnMut() -> JsValue>,
    _next: Closure<dyn FnMut() -> JsValue>,
}

impl MockListing {
    fn new(fixture: ListingFixture) -> Self {
        let next_fixture = fixture.clone();
        let next = Closure::wrap(Box::new(move || -> JsValue {
            match &next_fixture {
                ListingFixture::Results(results) => {
                    js_sys::Promise::resolve(&results.shift()).into()
                }
                ListingFixture::NextThrow(cause) => wasm_bindgen::throw_val(cause.clone()),
                ListingFixture::NextReject(cause) => js_sys::Promise::reject(cause).into(),
                ListingFixture::InvalidPromise(value) => value.clone(),
                _ => panic!("fixture cannot call next"),
            }
        }) as Box<dyn FnMut() -> JsValue>);
        let iterator = js_sys::Object::new();
        js_sys::Reflect::set(&iterator, &"next".into(), next.as_ref()).unwrap();
        let values = Closure::wrap(Box::new(move || -> JsValue {
            match &fixture {
                ListingFixture::ValuesThrow(cause) => wasm_bindgen::throw_val(cause.clone()),
                ListingFixture::InvalidIterator(value) => value.clone(),
                _ => iterator.clone().into(),
            }
        }) as Box<dyn FnMut() -> JsValue>);
        let directory = js_sys::Object::new();
        js_sys::Reflect::set(&directory, &"name".into(), &"fixture-root".into()).unwrap();
        js_sys::Reflect::set(&directory, &"values".into(), values.as_ref()).unwrap();
        Self {
            directory: WebDirectory::from(
                directory.unchecked_into::<web_sys::FileSystemDirectoryHandle>(),
            ),
            _values: values,
            _next: next,
        }
    }
}

fn finished_result() -> js_sys::Object {
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &"done".into(), &JsValue::TRUE).unwrap();
    result
}

#[wasm_bindgen_test]
async fn completed_empty_enumeration_never_inspects_the_absent_value() {
    let results = js_sys::Array::new();
    results.push(&finished_result());
    let mock = MockListing::new(ListingFixture::Results(results));
    assert!(
        mock.directory
            .list_entries()
            .await
            .unwrap()
            .into_iter()
            .next()
            .is_none()
    );
}

#[wasm_bindgen_test]
async fn enumeration_retains_file_directory_order_and_missing_done_is_false() {
    let _constructors = MockHandleConstructors::new();
    let results = js_sys::Array::new();
    for (kind, name) in [("file", "payload.bin"), ("directory", "nested")] {
        let handle = js_sys::Object::new();
        js_sys::Reflect::set(&handle, &"kind".into(), &kind.into()).unwrap();
        js_sys::Reflect::set(&handle, &"name".into(), &name.into()).unwrap();
        let result = js_sys::Object::new();
        js_sys::Reflect::set(&result, &"value".into(), &handle).unwrap();
        results.push(&result);
    }
    results.push(&finished_result());
    let mock = MockListing::new(ListingFixture::Results(results));
    let mut entries = mock.directory.list_entries().await.unwrap().into_iter();
    match entries.next().unwrap() {
        Entry::File(file) => assert_eq!(
            file.name(),
            crate::EntryLabel::from(String::from("payload.bin"))
        ),
        Entry::Directory(_) => panic!("file must remain first"),
    }
    match entries.next().unwrap() {
        Entry::Directory(directory) => assert_eq!(
            directory.name(),
            crate::EntryLabel::from(String::from("nested"))
        ),
        Entry::File(_) => panic!("directory must remain second"),
    }
    assert!(entries.next().is_none());
}

#[wasm_bindgen_test]
async fn synchronous_and_asynchronous_listing_failures_retain_the_original_cause() {
    let cause = js_sys::Error::new("listing rejected");
    for (fixture, expected) in [
        (
            ListingFixture::ValuesThrow(cause.clone().into()),
            BrowserListingStage::InvokeValuesMethod,
        ),
        (
            ListingFixture::NextThrow(cause.clone().into()),
            BrowserListingStage::InvokeNextMethod,
        ),
        (
            ListingFixture::NextReject(cause.clone().into()),
            BrowserListingStage::AwaitNext,
        ),
    ] {
        let mock = MockListing::new(fixture);
        match mock.directory.list_entries().await.unwrap_err() {
            DirectoryListingError::Browser {
                directory,
                stage,
                source,
            } => {
                assert_eq!(
                    directory.name(),
                    crate::EntryLabel::from(String::from("fixture-root"))
                );
                assert_eq!(stage, expected);
                assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
            }
        }
    }
}

#[wasm_bindgen_test]
async fn malformed_iterator_promises_and_results_are_observable() {
    let results = js_sys::Array::new();
    results.push(&JsValue::NULL);
    for (fixture, expected) in [
        (
            ListingFixture::InvalidIterator(JsValue::NULL),
            BrowserListingStage::DecodeIterator,
        ),
        (
            ListingFixture::InvalidPromise(JsValue::NULL),
            BrowserListingStage::DecodeNextPromise,
        ),
        (
            ListingFixture::Results(results),
            BrowserListingStage::DecodeResult,
        ),
    ] {
        let mock = MockListing::new(fixture);
        match mock.directory.list_entries().await.unwrap_err() {
            DirectoryListingError::Browser { stage, source, .. } => {
                assert_eq!(stage, expected);
                assert!(source.as_ref().is_null());
            }
        }
    }
}

#[wasm_bindgen_test]
async fn an_unknown_child_kind_fails_the_whole_listing() {
    let _constructors = MockHandleConstructors::new();
    let results = js_sys::Array::new();
    for kind in ["file", "unknown"] {
        let handle = js_sys::Object::new();
        js_sys::Reflect::set(&handle, &"kind".into(), &kind.into()).unwrap();
        let result = js_sys::Object::new();
        js_sys::Reflect::set(&result, &"value".into(), &handle).unwrap();
        results.push(&result);
    }
    let mock = MockListing::new(ListingFixture::Results(results));
    match mock.directory.list_entries().await.unwrap_err() {
        DirectoryListingError::Browser { stage, source, .. } => {
            assert_eq!(stage, BrowserListingStage::DecodeKind);
            assert_eq!(source.as_ref().as_string().as_deref(), Some("unknown"));
        }
    }
}

#[wasm_bindgen_test]
async fn iterator_property_exceptions_retain_the_original_provider_value() {
    let cause = js_sys::Error::new("done getter rejected");
    let thrown = cause.clone();
    let getter = Closure::wrap(Box::new(move || -> JsValue {
        wasm_bindgen::throw_val(thrown.clone().into())
    }) as Box<dyn FnMut() -> JsValue>);
    let descriptor = js_sys::Object::new();
    js_sys::Reflect::set(&descriptor, &"get".into(), getter.as_ref()).unwrap();
    let result = js_sys::Object::new();
    js_sys::Object::define_property(&result, &"done".into(), &descriptor);
    let results = js_sys::Array::new();
    results.push(&result);
    let mock = MockListing::new(ListingFixture::Results(results));
    match mock.directory.list_entries().await.unwrap_err() {
        DirectoryListingError::Browser { stage, source, .. } => {
            assert_eq!(stage, BrowserListingStage::ReadDone);
            assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
        }
    }
}

#[wasm_bindgen_test]
async fn a_known_kind_does_not_admit_a_malformed_provider_handle() {
    for (kind, expected) in [
        ("file", BrowserListingStage::DecodeFileHandle),
        ("directory", BrowserListingStage::DecodeDirectoryHandle),
    ] {
        let handle = js_sys::Object::new();
        js_sys::Reflect::set(&handle, &"kind".into(), &kind.into()).unwrap();
        let result = js_sys::Object::new();
        js_sys::Reflect::set(&result, &"value".into(), &handle).unwrap();
        let results = js_sys::Array::new();
        results.push(&result);
        let mock = MockListing::new(ListingFixture::Results(results));
        match mock.directory.list_entries().await.unwrap_err() {
            DirectoryListingError::Browser { stage, source, .. } => {
                assert_eq!(stage, expected);
                assert!(js_sys::Object::is(source.as_ref(), handle.as_ref()));
            }
        }
    }
}
