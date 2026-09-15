use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};
#[wasm_bindgen(module = "/web/bridge.js")]
extern "C" {
    fn create_bake_worker() -> web_sys::Worker;
}
type Inbox = Rc<RefCell<Option<Result<Vec<u8>, String>>>>;
pub(super) struct Backend {
    worker: web_sys::Worker,
    result: Inbox,
    _message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _error: Closure<dyn FnMut(web_sys::ErrorEvent)>,
}
impl Backend {
    pub fn new() -> Self {
        let worker = create_bake_worker();
        let result: Inbox = Rc::new(RefCell::new(None));
        let inbox = result.clone();
        let message = Closure::new(move |event: web_sys::MessageEvent| {
            let data = event.data();
            *inbox.borrow_mut() = Some(if let Some(error) = data.as_string() {
                Err(error)
            } else {
                Ok(js_sys::Uint8Array::new(&data).to_vec())
            });
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
    pub fn start(&mut self, source: String) {
        if let Err(error) = self.worker.post_message(&JsValue::from_str(&source)) {
            *self.result.borrow_mut() = Some(Err(format!("Worker could not start: {error:?}")));
        }
    }
    pub fn poll(&mut self) -> Option<Result<Vec<u8>, String>> {
        let result = self.result.borrow_mut().take();
        if matches!(&result, Some(Err(_))) {
            *self = Self::new();
        }
        result
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
