/// The concrete authority behind one directory namespace. Browser handles have
/// no serialized filesystem path; their names are presentation labels only.
#[derive(Clone, Debug)]
pub enum DirectoryNamespace {
    #[cfg(not(target_arch = "wasm32"))]
    Native(crate::NativeDirectory),
    #[cfg(target_arch = "wasm32")]
    Browser(crate::WebDirectory),
}
