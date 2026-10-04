mod content;
mod listing;
#[cfg(test)]
pub(crate) mod test_handles;
use crate::entry_access::BrowserEntryStage;
use crate::{
    DirectoryContents, DirectoryEntry, DirectoryListingError, EntryAccessError, EntryLookupIntent,
    EntryName, EntryOperation, FileEntry,
};
use async_trait::async_trait;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct WebFile {
    handle: SendWrapper<web_sys::FileSystemFileHandle>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct WebDirectory {
    handle: SendWrapper<web_sys::FileSystemDirectoryHandle>,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl DirectoryEntry for WebDirectory {
    fn name(&self) -> crate::EntryLabel {
        crate::EntryLabel::from(self.handle.name())
    }
    async fn list_entries(&self) -> Result<DirectoryContents, DirectoryListingError> {
        let mut iterator = listing::DirectoryListingIterator::new(self)?;
        let mut entries = DirectoryContents::default();
        loop {
            match iterator.next().await? {
                listing::ListingStep::Finished => return Ok(entries),
                listing::ListingStep::Child(entry) => entries.push(entry),
            }
        }
    }
    async fn get_file(
        &self,
        name: &EntryName,
        intent: EntryLookupIntent,
    ) -> std::result::Result<Box<dyn FileEntry>, EntryAccessError> {
        let operation = EntryOperation::LookupFile;
        let promise = self.request(
            name,
            EntryRequest::Lookup {
                intent,
                kind: EntryLookupKind::File,
            },
        )?;
        let value = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|source: JsValue| -> EntryAccessError {
                EntryAccessError::from_browser(
                    name,
                    operation,
                    BrowserEntryStage::AwaitHandle,
                    source,
                )
            })?;
        let handle = value.dyn_into::<web_sys::FileSystemFileHandle>().map_err(
            |source: JsValue| -> EntryAccessError {
                EntryAccessError::from_browser(
                    name,
                    operation,
                    BrowserEntryStage::DecodeHandle,
                    source,
                )
            },
        )?;
        Ok(Box::new(WebFile::from(handle)))
    }
    async fn get_directory(
        &self,
        name: &EntryName,
        intent: EntryLookupIntent,
    ) -> std::result::Result<Box<dyn DirectoryEntry>, EntryAccessError> {
        let operation = EntryOperation::LookupDirectory;
        let promise = self.request(
            name,
            EntryRequest::Lookup {
                intent,
                kind: EntryLookupKind::Directory,
            },
        )?;
        let value = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|source: JsValue| -> EntryAccessError {
                EntryAccessError::from_browser(
                    name,
                    operation,
                    BrowserEntryStage::AwaitHandle,
                    source,
                )
            })?;
        let handle = value
            .dyn_into::<web_sys::FileSystemDirectoryHandle>()
            .map_err(|source: JsValue| -> EntryAccessError {
                EntryAccessError::from_browser(
                    name,
                    operation,
                    BrowserEntryStage::DecodeHandle,
                    source,
                )
            })?;
        Ok(Box::new(WebDirectory::from(handle)))
    }
    async fn delete_entry(&self, name: &EntryName) -> std::result::Result<(), EntryAccessError> {
        let promise = self.request(name, EntryRequest::Delete)?;
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|source: JsValue| -> EntryAccessError {
                EntryAccessError::from_browser(
                    name,
                    EntryOperation::Delete,
                    BrowserEntryStage::Remove,
                    source,
                )
            })?;
        Ok(())
    }
    fn namespace(&self) -> crate::DirectoryNamespace {
        crate::DirectoryNamespace::Browser(self.clone())
    }
}

/// A rejected or malformed provider value retained on its originating thread.
/// Crossing a thread is checked by SendWrapper rather than an unsafe promise.
#[derive(Debug)]
pub struct BrowserIoCause(SendWrapper<JsValue>);

impl From<JsValue> for BrowserIoCause {
    fn from(source: JsValue) -> Self {
        Self(SendWrapper::new(source))
    }
}
impl AsRef<JsValue> for BrowserIoCause {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}
impl std::fmt::Display for BrowserIoCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}
impl std::error::Error for BrowserIoCause {}

impl From<web_sys::FileSystemFileHandle> for WebFile {
    fn from(handle: web_sys::FileSystemFileHandle) -> Self {
        Self {
            handle: SendWrapper::new(handle),
        }
    }
}
impl From<web_sys::FileSystemDirectoryHandle> for WebDirectory {
    fn from(handle: web_sys::FileSystemDirectoryHandle) -> Self {
        Self {
            handle: SendWrapper::new(handle),
        }
    }
}
impl AsRef<web_sys::FileSystemDirectoryHandle> for WebDirectory {
    fn as_ref(&self) -> &web_sys::FileSystemDirectoryHandle {
        &self.handle
    }
}
impl WebDirectory {
    fn request(
        &self,
        name: &EntryName,
        request: EntryRequest,
    ) -> std::result::Result<js_sys::Promise, EntryAccessError> {
        let operation = request.operation();
        let options = request.options(name)?;
        let method = match request {
            EntryRequest::Lookup {
                kind: EntryLookupKind::File,
                ..
            } => "getFileHandle",
            EntryRequest::Lookup {
                kind: EntryLookupKind::Directory,
                ..
            } => "getDirectoryHandle",
            EntryRequest::Delete => "removeEntry",
        };
        let method = js_sys::Reflect::get(&self.handle, &method.into()).map_err(
            |source: JsValue| -> EntryAccessError {
                EntryAccessError::from_browser(
                    name,
                    operation,
                    BrowserEntryStage::ReadMethod,
                    source,
                )
            },
        )?;
        let method = method.dyn_into::<js_sys::Function>().map_err(
            |source: JsValue| -> EntryAccessError {
                EntryAccessError::from_browser(
                    name,
                    operation,
                    BrowserEntryStage::DecodeMethod,
                    source,
                )
            },
        )?;
        let result = match options {
            Some(options) => method.call2(&self.handle, &name.as_ref().into(), &options),
            None => method.call1(&self.handle, &name.as_ref().into()),
        }
        .map_err(|source: JsValue| -> EntryAccessError {
            EntryAccessError::from_browser(name, operation, BrowserEntryStage::InvokeMethod, source)
        })?;
        result
            .dyn_into::<js_sys::Promise>()
            .map_err(|source: JsValue| -> EntryAccessError {
                EntryAccessError::from_browser(
                    name,
                    operation,
                    BrowserEntryStage::DecodePromise,
                    source,
                )
            })
    }
}

#[derive(Clone, Copy)]
enum EntryRequest {
    Lookup {
        intent: EntryLookupIntent,
        kind: EntryLookupKind,
    },
    Delete,
}
impl EntryRequest {
    fn operation(self) -> EntryOperation {
        match self {
            Self::Lookup { kind, .. } => kind.operation(),
            Self::Delete => EntryOperation::Delete,
        }
    }
    fn options(
        self,
        name: &EntryName,
    ) -> std::result::Result<Option<js_sys::Object>, EntryAccessError> {
        let Self::Lookup { intent, .. } = self else {
            return Ok(None);
        };
        let options = js_sys::Object::new();
        let create = intent == EntryLookupIntent::CreateIfMissing;
        let encoded = js_sys::Reflect::set(&options, &"create".into(), &create.into()).map_err(
            |source: JsValue| -> EntryAccessError {
                EntryAccessError::from_browser(
                    name,
                    self.operation(),
                    BrowserEntryStage::EncodeOptions,
                    source,
                )
            },
        )?;
        if !encoded {
            return Err(EntryAccessError::from_browser(
                name,
                self.operation(),
                BrowserEntryStage::EncodeOptions,
                JsValue::FALSE,
            ));
        }
        Ok(Some(options))
    }
}

#[derive(Clone, Copy)]
enum EntryLookupKind {
    File,
    Directory,
}
impl EntryLookupKind {
    fn operation(self) -> EntryOperation {
        match self {
            Self::File => EntryOperation::LookupFile,
            Self::Directory => EntryOperation::LookupDirectory,
        }
    }
}
impl EntryAccessError {
    fn from_browser(
        name: &EntryName,
        operation: EntryOperation,
        stage: BrowserEntryStage,
        source: JsValue,
    ) -> Self {
        Self::Browser {
            operation,
            entry: name.clone(),
            stage,
            source: BrowserIoCause::from(source),
        }
    }
}

#[cfg(test)]
mod tests;
