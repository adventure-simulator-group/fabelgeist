use super::*;
use crate::FileSuffix;
use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn browser_filters_encode_the_owned_mime_suffixes_description_and_single_selection() {
    let suffixes = FileSuffixes::try_from(vec![
        FileSuffix::try_from("PNG").unwrap(),
        FileSuffix::try_from(".png").unwrap(),
    ])
    .unwrap();
    let filter = FilePickerFilter::AcceptedType(FileTypeFilter::new(
        "Images".into(),
        ResourceMime::Png,
        suffixes,
    ));
    let options = filter.browser_options().unwrap();
    assert_eq!(
        js_sys::Reflect::get(&options, &"multiple".into()).unwrap(),
        JsValue::FALSE
    );
    let types = js_sys::Reflect::get(&options, &"types".into())
        .unwrap()
        .dyn_into::<js_sys::Array>()
        .unwrap();
    assert_eq!(types.length(), 1);
    let group = types.get(0);
    assert_eq!(
        js_sys::Reflect::get(&group, &"description".into()).unwrap(),
        JsValue::from("Images")
    );
    let accept = js_sys::Reflect::get(&group, &"accept".into()).unwrap();
    let extensions = js_sys::Reflect::get(&accept, &"image/png".into())
        .unwrap()
        .dyn_into::<js_sys::Array>()
        .unwrap();
    assert_eq!(extensions.get(0), JsValue::from(".PNG"));
    assert_eq!(extensions.get(1), JsValue::from(".png"));
    assert!(
        js_sys::Reflect::get(&accept, &"*/*".into())
            .unwrap()
            .is_undefined()
    );
    let options = FilePickerFilter::AnyFile.browser_options().unwrap();
    assert!(
        js_sys::Reflect::get(&options, &"types".into())
            .unwrap()
            .is_undefined()
    );
    assert_eq!(
        js_sys::Reflect::get(&options, &"multiple".into()).unwrap(),
        JsValue::FALSE
    );
}

#[wasm_bindgen_test]
fn option_encoding_reports_rejected_property_writes() {
    let target = js_sys::Object::freeze(&js_sys::Object::new());
    match PickerOptionField::Multiple
        .encode(&target, &JsValue::FALSE)
        .unwrap_err()
    {
        PickerError::Browser {
            kind,
            stage,
            source,
        } => {
            assert_eq!(kind, PickerKind::File);
            assert_eq!(
                stage,
                BrowserPickerStage::EncodeOption(PickerOptionField::Multiple)
            );
            assert_eq!(source.as_ref(), &JsValue::FALSE);
        }
        PickerError::MissingWindow { .. } => panic!("encoding does not need a window"),
    }
}

#[wasm_bindgen_test]
fn option_encoding_retains_exceptions_from_the_provider_protocol() {
    let cause = js_sys::Error::new("property write failed");
    let setter_cause = cause.clone();
    let setter = Closure::wrap(Box::new(
        move |_target: JsValue, _name: JsValue, _value: JsValue| -> JsValue {
            wasm_bindgen::throw_val(setter_cause.clone().into())
        },
    )
        as Box<dyn FnMut(JsValue, JsValue, JsValue) -> JsValue>);
    let handler = js_sys::Object::new();
    js_sys::Reflect::set(&handler, &"set".into(), setter.as_ref()).unwrap();
    let target =
        js_sys::Proxy::new(&js_sys::Object::new(), &handler).unchecked_into::<js_sys::Object>();
    match PickerOptionField::Description
        .encode(&target, &"Images".into())
        .unwrap_err()
    {
        PickerError::Browser { stage, source, .. } => {
            assert_eq!(
                stage,
                BrowserPickerStage::EncodeOption(PickerOptionField::Description)
            );
            assert!(js_sys::Object::is(source.as_ref(), cause.as_ref()));
        }
        PickerError::MissingWindow { .. } => panic!("encoding does not need a window"),
    }
}
