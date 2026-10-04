use wasm_bindgen::JsValue;

/// Node lacks File System Access constructors. Scope their instanceof fixtures
/// so the real admission decoder runs and both globals are restored afterwards.
pub(crate) struct MockHandleConstructors {
    file: JsValue,
    directory: JsValue,
}
impl MockHandleConstructors {
    pub(crate) fn new() -> Self {
        let global = js_sys::global();
        let file = js_sys::Reflect::get(&global, &"FileSystemFileHandle".into()).unwrap();
        let directory = js_sys::Reflect::get(&global, &"FileSystemDirectoryHandle".into()).unwrap();
        let constructor = js_sys::Reflect::get(&global, &"Object".into()).unwrap();
        js_sys::Reflect::set(&global, &"FileSystemFileHandle".into(), &constructor).unwrap();
        js_sys::Reflect::set(&global, &"FileSystemDirectoryHandle".into(), &constructor).unwrap();
        Self { file, directory }
    }
}
impl Drop for MockHandleConstructors {
    fn drop(&mut self) {
        let global = js_sys::global();
        js_sys::Reflect::set(&global, &"FileSystemFileHandle".into(), &self.file).unwrap();
        js_sys::Reflect::set(
            &global,
            &"FileSystemDirectoryHandle".into(),
            &self.directory,
        )
        .unwrap();
    }
}
