use super::*;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};

#[wasm_bindgen(module = "/web/bridge.js")]
extern "C" {
    fn create_bake_worker() -> web_sys::Worker;
}

pub(super) struct Backend {
    worker: web_sys::Worker,
    result: Rc<RefCell<Option<Result<BakedRecipe, String>>>>,
    _message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _error: Closure<dyn FnMut(web_sys::ErrorEvent)>,
}
impl Backend {
    pub fn new() -> Self {
        let worker = create_bake_worker();
        let result = Rc::new(RefCell::new(None));
        let inbox = result.clone();
        let message = Closure::new(move |event: web_sys::MessageEvent| {
            let data = event.data();
            let value = if let Some(error) = data.as_string() {
                Err(error)
            } else {
                BakedRecipe::from_bytes(&js_sys::Uint8Array::new(&data).to_vec())
            };
            *inbox.borrow_mut() = Some(value);
        });
        let inbox = result.clone();
        let error = Closure::new(move |event: web_sys::ErrorEvent| {
            *inbox.borrow_mut() = Some(Err(event.message()));
        });
        worker.set_onmessage(Some(message.as_ref().unchecked_ref()));
        worker.set_onerror(Some(error.as_ref().unchecked_ref()));
        Self {
            worker,
            result,
            _message: message,
            _error: error,
        }
    }
    pub fn start(&mut self, document: Document) {
        if let Err(error) = self
            .worker
            .post_message(&JsValue::from_str(&document.to_json()))
        {
            *self.result.borrow_mut() = Some(Err(format!("Worker request failed: {error:?}")));
        }
    }
    pub fn poll(&mut self) -> Option<Result<BakedRecipe, String>> {
        self.result.borrow_mut().take()
    }
    pub fn cancel(&mut self) -> bool {
        *self = Self::new();
        true
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        self.worker.terminate();
    }
}
