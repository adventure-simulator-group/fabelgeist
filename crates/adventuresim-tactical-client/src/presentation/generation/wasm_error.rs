//! Convert typed preparation failures at the JavaScript exception boundary.
use super::PreparationError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Error)]
    type JavaScriptError;

    #[wasm_bindgen(constructor, js_class = Error)]
    fn new(message: &str) -> JavaScriptError;

    #[wasm_bindgen(method, setter, js_name = name)]
    fn set_name(this: &JavaScriptError, name: &str);
}

impl From<PreparationError> for JsValue {
    fn from(error: PreparationError) -> Self {
        let exception = JavaScriptError::new(&error.to_string());
        exception.set_name(error.code());
        exception.into()
    }
}
